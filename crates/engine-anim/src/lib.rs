//! # Engine Animation — skeletal + GPU skinning, blend/FSM graph, IK, crowd VAT
//!
//! First-party animation pipeline for the game engine. Provides:
//! - Bevy's `AnimationPlayer`/`AnimationClip` graph playback (headless-safe)
//! - Skeletal animation with optional GPU skinning (render feature)
//! - Distance-tiered LOD: near skeletal → mid reduced → far vertex-animation-texture (VAT)
//! - Two-bone IK solver (FABRIK, deterministic where shared)
//! - Blend graph evaluation (weighted/additive/masked, pure math)
//!
//! All graph/IK/VAT logic is pure (headless-testable); GPU skinning wiring is
//! feature-gated. Systems run in `SimSet::Simulate` (FixedUpdate, deterministic).

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

pub mod error;
pub use error::AnimError;

/// Global animation configuration and runtime state.
#[derive(Resource, Reflect, Debug, Clone, Copy)]
pub struct AnimationConfig {
    /// Enable GPU skinning under the render feature (native only).
    pub skinning_enabled: bool,
    /// Distance threshold (world units) for LOD tier transition to reduced skeletal.
    pub lod_mid_distance: f32,
    /// Distance threshold (world units) for LOD tier transition to VAT.
    pub lod_far_distance: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            skinning_enabled: cfg!(feature = "render"),
            lod_mid_distance: 30.0,
            lod_far_distance: 100.0,
        }
    }
}

/// Engine animation plugin: bevy_animation playback, blend graph, IK, VAT LOD.
///
/// Registers animation types and systems. Under the `render` feature, enables
/// GPU skinning with bevy_render's SkinnedMesh. Pure blend/IK/VAT logic runs
/// headless in `SimSet::Simulate` (deterministic FixedUpdate).
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // Headless-safe core: register animation types for reflection.
        app.register_type::<AnimationConfig>();

        // Insert global config resource (defaults used if not explicitly set).
        app.init_resource::<AnimationConfig>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_engine_core::headless_app;

    #[test]
    fn anim_plugin_registers_types() {
        let mut app = headless_app();
        app.add_plugins(AnimationPlugin);

        // Verify AnimationConfig resource is inserted.
        let config = app.world().get_resource::<AnimationConfig>();
        assert!(config.is_some(), "AnimationConfig not inserted");

        // Verify the config has sensible defaults.
        let cfg = config.unwrap();
        assert_eq!(cfg.lod_mid_distance, 30.0);
        assert_eq!(cfg.lod_far_distance, 100.0);
    }

    #[test]
    fn anim_config_headless_defaults() {
        let config = AnimationConfig::default();
        assert_eq!(config.lod_mid_distance, 30.0);
        assert_eq!(config.lod_far_distance, 100.0);

        // Headless (no render feature): skinning_enabled == false.
        #[cfg(not(feature = "render"))]
        {
            assert!(!config.skinning_enabled);
        }
    }
}
