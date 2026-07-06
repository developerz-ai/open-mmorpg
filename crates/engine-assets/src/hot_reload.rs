//! Hot-reload plumbing — turn Bevy's per-type [`AssetEvent`] reload signal into a
//! single, type-erased [`AssetHotReloaded`] message the rest of the engine can
//! react to without depending on any concrete (render-gated) asset type.
//!
//! # Why type-erased
//! Bevy emits `AssetEvent::<A>::Modified` once per asset kind `A`. Gameplay,
//! tooling and agents want one uniform "an asset just live-reloaded" signal, and
//! they must stay compilable in the headless build where render asset types
//! (`Mesh`, `StandardMaterial`, …) do not exist. So a caller opts a concrete type
//! into the shared channel with [`AssetHotReloadAppExt::forward_hot_reload`] and
//! then listens only for [`AssetHotReloaded`]. → `docs/specs/game-engine/assets/README.md`.

use bevy_app::prelude::*;
use bevy_asset::{Asset, AssetEvent, AssetEventSystems};
use bevy_ecs::prelude::*;

/// A watched asset was (re)loaded — the engine-level "hot reload happened"
/// signal, decoupled from the concrete asset type.
///
/// Emitted whenever an asset that was registered via
/// [`AssetHotReloadAppExt::forward_hot_reload`] is modified: the file-watcher
/// reloading it from disk, or any in-place `Assets::<A>::get_mut`. Listen with
/// `MessageReader<AssetHotReloaded>`.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct AssetHotReloaded {
    /// Rust type name of the asset kind that reloaded (e.g. the glTF type). A
    /// stable string, so headless tools and agents can match on it without
    /// linking the render types themselves.
    pub asset_type: &'static str,
}

/// Bridge `AssetEvent::<A>::Modified` into the type-erased [`AssetHotReloaded`]
/// channel. Registered per asset type by [`AssetHotReloadAppExt::forward_hot_reload`].
fn forward_hot_reload<A: Asset>(
    mut incoming: MessageReader<AssetEvent<A>>,
    mut outgoing: MessageWriter<AssetHotReloaded>,
) {
    for event in incoming.read() {
        // Only `Modified` is a reload; `Added`/`Removed`/`Unused`/`Loaded…` are
        // lifecycle noise for this signal.
        if matches!(event, AssetEvent::Modified { .. }) {
            outgoing.write(AssetHotReloaded {
                asset_type: core::any::type_name::<A>(),
            });
        }
    }
}

/// [`App`] extension for opting a concrete asset type into the hot-reload channel.
pub trait AssetHotReloadAppExt {
    /// Emit an [`AssetHotReloaded`] whenever an asset of type `A` is modified.
    ///
    /// Requires the [`AssetHotReloaded`] message to be registered first — the
    /// [`AssetsPlugin`](crate::AssetsPlugin) does that in its `build`. Idempotent
    /// per type: adding the forwarder twice just runs it once more per frame.
    fn forward_hot_reload<A: Asset>(&mut self) -> &mut Self;
}

impl AssetHotReloadAppExt for App {
    fn forward_hot_reload<A: Asset>(&mut self) -> &mut Self {
        // `PostUpdate` after `AssetEventSystems`: Bevy flushes `AssetEvent`s there,
        // so forwarding in the same set-ordered slot delivers the signal without a
        // frame of latency. Hot reload is a frame-level dev-loop signal, never part
        // of the deterministic fixed simulation.
        self.add_systems(PostUpdate, forward_hot_reload::<A>.after(AssetEventSystems))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_asset::{AssetApp, Assets};
    use bevy_reflect::TypePath;
    use omm_engine_core::EnginePlugins;

    use crate::AssetsPlugin;

    /// Throwaway asset used only to drive the forwarder headlessly — no GPU, no IO.
    #[derive(Asset, TypePath, Default)]
    struct FakeAsset {
        version: u32,
    }

    /// Captures forwarded reloads so a test can assert on them.
    #[derive(Resource, Default)]
    struct Captured(Vec<AssetHotReloaded>);

    fn capture(mut sink: ResMut<Captured>, mut reloads: MessageReader<AssetHotReloaded>) {
        for reload in reloads.read() {
            sink.0.push(reload.clone());
        }
    }

    fn hot_reload_app() -> App {
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        app.add_plugins(AssetsPlugin::default());
        app.init_asset::<FakeAsset>();
        app.forward_hot_reload::<FakeAsset>();
        app.init_resource::<Captured>();
        // `Last` runs strictly after the `PostUpdate` forwarder, so the capture
        // sees a same-frame reload without ordering guesswork.
        app.add_systems(Last, capture);
        app
    }

    #[test]
    fn modifying_an_asset_emits_a_hot_reload() {
        let mut app = hot_reload_app();
        let handle = app
            .world_mut()
            .resource_mut::<Assets<FakeAsset>>()
            .add(FakeAsset::default());
        // Frame 1 flushes the `Added` event — which must NOT be forwarded.
        app.update();
        assert!(
            app.world().resource::<Captured>().0.is_empty(),
            "an added asset is not a hot reload"
        );

        // Mutating in place emits `AssetEvent::Modified`.
        {
            let mut assets = app.world_mut().resource_mut::<Assets<FakeAsset>>();
            let Some(mut asset) = assets.get_mut(&handle) else {
                panic!("asset missing after add");
            };
            asset.version += 1;
        }
        app.update();

        let captured = &app.world().resource::<Captured>().0;
        assert_eq!(captured.len(), 1, "one modify => one hot-reload signal");
        let Some(reload) = captured.first() else {
            panic!("no hot-reload captured");
        };
        assert_eq!(reload.asset_type, core::any::type_name::<FakeAsset>());
    }

    #[test]
    fn added_and_removed_are_not_hot_reloads() {
        let mut app = hot_reload_app();
        let handle = app
            .world_mut()
            .resource_mut::<Assets<FakeAsset>>()
            .add(FakeAsset::default());
        app.update();
        app.world_mut()
            .resource_mut::<Assets<FakeAsset>>()
            .remove(&handle);
        app.update();
        assert!(
            app.world().resource::<Captured>().0.is_empty(),
            "add/remove must not surface as hot reloads"
        );
    }
}
