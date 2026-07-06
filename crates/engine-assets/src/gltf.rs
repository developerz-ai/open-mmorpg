//! glTF loader registration — **render-gated**.
//!
//! glTF 2.0/glB is the engine's mesh/material/animation source of truth: any
//! Blender/Houdini/AI-gen tool that emits glTF drops in with zero conversion
//! (→ `docs/specs/game-engine/assets/README.md`). But import produces render
//! asset types — `Mesh`, `StandardMaterial`, `Image` — that only exist in a
//! rendered head, so this whole module lives behind the `render` cargo feature
//! and never compiles into the headless server build.

use bevy_app::App;
use bevy_gltf::{Gltf, GltfPlugin};

use crate::AssetHotReloadAppExt;

/// Register Bevy's glTF 2.0 / glB asset loader on `app` and route glTF reloads
/// into the engine's [hot-reload channel](crate::AssetHotReloaded).
///
/// Called by [`AssetsPlugin`](crate::AssetsPlugin) when the `render` feature is
/// on. Idempotent: a pre-existing [`GltfPlugin`] is left untouched (double-adding
/// a Bevy plugin panics). The loader needs the render stack's `Mesh`/
/// `StandardMaterial`/`Image` asset types at *load* time — those arrive with the
/// render plugin; registering the loader here is safe headless.
pub(crate) fn register(app: &mut App) {
    if !app.is_plugin_added::<GltfPlugin>() {
        app.add_plugins(GltfPlugin::default());
    }
    // "edit .gltf → hot-reload in ms": surface glTF reloads on the shared channel.
    app.forward_hot_reload::<Gltf>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_asset::Assets;
    use omm_engine_core::EnginePlugins;

    use crate::AssetsPlugin;

    #[test]
    fn gltf_loader_registers_headless() {
        // Under the `render` feature the loader must register on the headless core
        // without a GPU/window: build + finalize prove no device init sneaks in.
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        app.add_plugins(AssetsPlugin::default());
        app.finish();
        app.cleanup();

        assert!(
            app.is_plugin_added::<GltfPlugin>(),
            "the glTF loader must be registered when `render` is enabled"
        );
        assert!(
            app.world().get_resource::<Assets<Gltf>>().is_some(),
            "GltfPlugin must register the `Gltf` asset type"
        );
    }
}
