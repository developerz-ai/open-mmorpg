//! **Engine asset pipeline** — the async asset server, hot-reload plumbing and
//! (render-gated) glTF import that feed the game engine, built on the
//! `omm_engine_core` headless substrate.
//!
//! # Open formats, one server
//! [`AssetsPlugin`] composes Bevy's [`AssetPlugin`] — the async, content-addressed
//! asset server — and wires a type-erased [hot-reload](crate::AssetHotReloaded)
//! channel on top. glTF/KTX2/EXR/Ogg load through the same server; nothing
//! proprietary enters the chain. → `docs/specs/game-engine/assets/README.md`.
//!
//! # Headless-first, watch is opt-in
//! The plugin pulls in no GPU or window: loading and hot-reload dispatch are pure
//! ECS, so an agent drives them headless exactly as the rendered client does.
//! File-watching is a **runtime toggle** ([`AssetsPlugin::watch_for_changes`]),
//! **off** by default so CI/headless runs stay deterministic (no background IO
//! threads); the watcher backend itself is the opt-in `watch` cargo feature.
//! glTF import is the `render` cargo feature — it produces `Mesh`/
//! `StandardMaterial` render assets that only exist in a rendered head.
//!
//! # The MMO-scale pipeline logic (pure, headless)
//! Three modules are plain deterministic logic — no GPU, no ECS, no IO — so a CI
//! agent validates content and reasons about detail/streaming exactly as the
//! client does:
//! - [`manifest`] — parse `manifest.json` and validate each slot spec
//!   (model + textures + rig) fail-loud, so AI-generated content only has to *fit
//!   the slot*.
//! - [`lod`] — pick a mesh tier from on-screen size, drop to an octahedral
//!   imposter for the far tier, cull the rest.
//! - [`streaming`] — load/unload world tiles by camera position under a hard
//!   memory budget, nearest-first.

#![forbid(unsafe_code)]

pub mod error;
pub mod lod;
pub mod manifest;
pub mod streaming;

mod hot_reload;

#[cfg(feature = "render")]
mod gltf;

pub use error::AssetError;
pub use hot_reload::{AssetHotReloadAppExt, AssetHotReloaded};
pub use lod::{ImposterAtlas, ImposterCell, LodChain, LodSelection};
pub use manifest::{AssetManifest, AssetSlot, SlotCandidate, SlotKind};
pub use streaming::{StreamingDelta, StreamingGrid, TileCoord};

use bevy_app::{App, Plugin};
use bevy_asset::AssetPlugin;

/// The engine's asset plugin: async asset server + hot-reload channel, plus the
/// glTF loader when the `render` feature is on.
///
/// Headless-safe — adds no rendering. Add it to any [`App`] (on top of
/// `omm_engine_core::EnginePlugins`) to load assets and, when a concrete asset
/// type is opted in via [`AssetHotReloadAppExt::forward_hot_reload`], receive
/// [`AssetHotReloaded`] messages on live reload.
#[derive(Debug, Default, Clone)]
pub struct AssetsPlugin {
    /// Watch the asset directory and live-reload on change (hot reload).
    ///
    /// **Off by default.** A watcher spawns background IO threads and makes load
    /// timing non-deterministic — fine for a dev/agent iterate loop, wrong for
    /// CI/headless. Turning it on additionally requires the `watch` cargo feature
    /// for the watcher backend; without that feature this logs a Bevy warning and
    /// no watcher starts (the server still loads assets on demand).
    pub watch_for_changes: bool,
}

impl AssetsPlugin {
    /// Build the [`AssetPlugin`] this plugin installs, mapping the runtime watch
    /// toggle onto Bevy's `watch_for_changes_override`. Split out so the mapping
    /// is unit-testable without starting a real asset server / watcher.
    fn asset_plugin(&self) -> AssetPlugin {
        AssetPlugin {
            watch_for_changes_override: Some(self.watch_for_changes),
            ..Default::default()
        }
    }
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        // The async asset server. Respect a pre-existing one (e.g. added by the
        // scene plugin) rather than double-adding, which would panic.
        if !app.is_plugin_added::<AssetPlugin>() {
            app.add_plugins(self.asset_plugin());
        }
        // The type-erased hot-reload channel. Concrete asset types opt in with
        // `App::forward_hot_reload::<A>()`.
        app.add_message::<AssetHotReloaded>();

        // glTF is the mesh/material/anim source of truth, but it pulls render
        // asset types — so the loader only exists in a rendered build.
        #[cfg(feature = "render")]
        gltf::register(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_asset::AssetPlugin;
    use bevy_ecs::message::Messages;
    use omm_engine_core::EnginePlugins;

    fn assets_app(plugin: AssetsPlugin) -> App {
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        app.add_plugins(plugin);
        app
    }

    #[test]
    fn plugin_builds_and_runs_headless() {
        // Composing AssetPlugin on the headless core must not touch a GPU/window:
        // build, finalize and tick once without panicking.
        let mut app = assets_app(AssetsPlugin::default());
        app.finish();
        app.cleanup();
        app.update();
    }

    #[test]
    fn asset_server_is_present() {
        let app = assets_app(AssetsPlugin::default());
        assert!(
            app.is_plugin_added::<AssetPlugin>(),
            "AssetPlugin (the asset server) must be composed"
        );
    }

    #[test]
    fn hot_reload_channel_is_registered() {
        // The type-erased channel must exist even before any concrete asset type
        // is forwarded, so listeners can be wired unconditionally.
        let app = assets_app(AssetsPlugin::default());
        assert!(
            app.world()
                .get_resource::<Messages<AssetHotReloaded>>()
                .is_some(),
            "AssetHotReloaded message must be registered by the plugin"
        );
    }

    #[test]
    fn watch_defaults_off_and_toggle_maps_to_override() {
        // Determinism guard: the default must not watch, and the runtime toggle
        // must map straight onto Bevy's override (proven without starting IO).
        assert!(!AssetsPlugin::default().watch_for_changes);
        assert_eq!(
            AssetsPlugin::default()
                .asset_plugin()
                .watch_for_changes_override,
            Some(false)
        );
        assert_eq!(
            AssetsPlugin {
                watch_for_changes: true
            }
            .asset_plugin()
            .watch_for_changes_override,
            Some(true)
        );
    }

    #[test]
    fn tolerates_a_preexisting_asset_plugin() {
        // If the host already added AssetPlugin (e.g. the scene plugin), the
        // assets plugin must not double-add it — it only fills the hot-reload wire.
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        app.add_plugins(AssetPlugin {
            watch_for_changes_override: Some(false),
            ..Default::default()
        });
        app.add_plugins(AssetsPlugin::default());
        assert!(app
            .world()
            .get_resource::<Messages<AssetHotReloaded>>()
            .is_some());
    }
}
