//! **Engine scene graph** — reflection-driven scene/prefab loading for the game
//! engine, built on the `omm_engine_core` substrate.
//!
//! # Content is data
//! A prefab is a *scene fragment* — entities and their reflected components —
//! authored as text (RON) by a person, the editor, or an AI agent, and spawned
//! into the world by [`Prefab::spawn`]. Behaviour is compiled; content is data.
//! Reflection is the entire contract: a type is (de)serializable, inspectable and
//! agent-visible exactly when it is in the registry, so [`EngineScenePlugin`]
//! `register_type`s the scene-graph components and every failure is loud
//! ([`SceneError`]). → `docs/specs/game-engine/scene/README.md`.
//!
//! # Headless-first
//! The plugin composes Bevy's [`AssetPlugin`] (with file-watching **off** — no
//! background IO, deterministic in CI) and [`ScenePlugin`] on top of the same
//! headless core the server runs. It pulls in no GPU, window or audio: scene
//! (de)serialization and spawning are pure ECS + reflection, so an agent drives
//! them in a headless harness identically to the rendered client.

mod error;
mod parse;
mod prefab;

pub use error::SceneError;
pub use prefab::{EntityBlueprint, Prefab};

use bevy_app::{App, Plugin};
use bevy_asset::AssetPlugin;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::name::Name;
use bevy_scene::ScenePlugin;
use bevy_transform::components::{GlobalTransform, Transform};

/// The engine's scene graph plugin: asset + scene runtime, plus reflection
/// registration of the scene-graph components.
///
/// Headless-safe — adds no rendering. Add it to any [`App`] (on top of
/// `omm_engine_core::EnginePlugins`) to enable data-driven scene/prefab spawning
/// and hierarchical entity management.
#[derive(Debug, Default, Clone, Copy)]
pub struct EngineScenePlugin;

impl Plugin for EngineScenePlugin {
    fn build(&self, app: &mut App) {
        // Asset backbone. File-watching stays off: hot reload is an explicit
        // opt-in for the assets batch, and a watcher would make CI non-deterministic.
        if !app.is_plugin_added::<AssetPlugin>() {
            app.add_plugins(AssetPlugin {
                watch_for_changes_override: Some(false),
                ..Default::default()
            });
        }
        // Bevy's scene runtime (asset-backed scene patches + spawner).
        if !app.is_plugin_added::<ScenePlugin>() {
            app.add_plugins(ScenePlugin);
        }
        // Scene-graph components: registered so scenes, the inspector and agents
        // can (de)serialize and enumerate them. Unregistered = invisible = a bug.
        app.register_type::<Transform>()
            .register_type::<GlobalTransform>()
            .register_type::<Name>()
            .register_type::<ChildOf>()
            .register_type::<Children>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;

    fn scene_app() -> App {
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        app.add_plugins(EngineScenePlugin);
        app
    }

    #[test]
    fn plugin_builds_and_runs_headless() {
        // Composing AssetPlugin + bevy ScenePlugin on the headless core must not
        // touch a GPU/window: build, finalize and tick once without panicking.
        let mut app = scene_app();
        app.finish();
        app.cleanup();
        app.update();
    }

    #[test]
    fn scene_components_are_registered() {
        let app = scene_app();
        let registry = app.world().resource::<AppTypeRegistry>().read();
        for type_id in [
            TypeId::of::<Transform>(),
            TypeId::of::<GlobalTransform>(),
            TypeId::of::<Name>(),
            TypeId::of::<ChildOf>(),
            TypeId::of::<Children>(),
        ] {
            assert!(
                registry.get(type_id).is_some(),
                "a scene-graph component is not registered"
            );
        }
    }

    #[test]
    fn asset_and_scene_runtimes_are_present() {
        let app = scene_app();
        assert!(
            app.is_plugin_added::<AssetPlugin>(),
            "AssetPlugin must be composed"
        );
        assert!(
            app.is_plugin_added::<ScenePlugin>(),
            "ScenePlugin must be composed"
        );
    }

    #[test]
    fn tolerates_a_preexisting_asset_plugin() {
        // If the host app already added AssetPlugin, the scene plugin must not
        // double-add it (which would panic) — it only fills the scene runtime.
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        app.add_plugins(AssetPlugin {
            watch_for_changes_override: Some(false),
            ..Default::default()
        });
        app.add_plugins(EngineScenePlugin);
        assert!(app.is_plugin_added::<ScenePlugin>());
    }
}
