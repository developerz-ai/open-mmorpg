//! **Engine scene graph** — Bevy scene management, asset loading, and entity
//! hierarchy for the game engine.
//!
//! Builds on [`omm_engine_core`] to add scene graph traversal, prefab loading,
//! and hierarchical transform propagation. This is the substrate for level
//! streaming, entity spawning, and the scene editor.

use bevy_app::{App, Plugin};

/// The engine's scene graph plugin: scene loading, asset management, and
/// hierarchy traversal.
///
/// Headless-safe — pulls in no rendering. Add it to any `App` to enable
/// scene I/O and hierarchical entity management.
#[derive(Debug, Default, Clone, Copy)]
pub struct EngineScenePlugin;

impl Plugin for EngineScenePlugin {
    fn build(&self, app: &mut App) {
        // Scene plugin configuration will be added here as the track progresses.
        let _ = app;
    }
}
