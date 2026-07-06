//! # Engine physics — in-house collision, broadphase, kinematic controller, LOS
//!
//! A Rust-native physics layer for the game engine, built **headless-first** and
//! **deterministic** so the same collision math the client uses to predict and
//! smooth movement can be reused server-side for movement validation and
//! anti-cheat re-simulation. No GPU, no window, no third-party solver — the
//! MMORPG's server owns the source of truth, so client/server determinism
//! matters more than solver fidelity (ADR-0001).
//!
//! ## Modules
//! - [`shapes`] — [`Aabb3d`], [`Sphere`], [`Capsule`] and the reflected
//!   [`Collider`] component, plus exact sphere/box and capsule/box penetration.
//! - [`query`] — [`Ray`] casts against every shape, nearest-hit scene queries,
//!   and [`line_of_sight`] for targeting/interaction probes.
//! - [`broadphase`] — the [`Broadphase`] resource, which **reuses
//!   [`omm_world`]'s quadtree** (one index, no drift) to prune the collider set.
//! - [`slide`] — the pure [`move_and_slide`] solver: slide, step-climb, floor
//!   snap. Unit-testable with no ECS.
//! - [`controller`] — the [`CharacterController`] component and the two ECS
//!   systems, chained in [`SimSet::Simulate`].
//!
//! ## What CI verifies vs what it does not
//! **Verified (headless, every commit):** all shape/penetration math, ray casts
//! and LOS, broadphase pruning (margin expansion + `y` filtering + stable
//! ordering), the full move-and-slide behaviour (fall/land, wall slide, step,
//! snap, determinism), and that every authored type is reflected for the editor.
//!
//! **Not verified here:** continuous collision (fast motion may tunnel thin
//! geometry — per-tick motion is assumed smaller than collider thickness, and
//! [`CharacterController::max_fall_speed`] caps the worst case), rotated/scaled
//! collider transforms, and character-vs-character resolution (the server is
//! authoritative on contact between players).
//!
//! → `docs/specs/game-engine/physics/README.md`.

pub mod broadphase;
pub mod controller;
mod error;
pub mod penetration;
pub mod query;
pub mod shapes;
pub mod slide;

pub use broadphase::Broadphase;
pub use controller::{
    character_controller_system, sync_broadphase, CharacterController, MoveIntent,
};
pub use error::PhysicsError;
pub use penetration::{capsule_vs_aabb, closest_point_on_segment, sphere_vs_aabb, Penetration};
pub use query::{
    line_of_sight, ray_vs_aabb, ray_vs_capsule, ray_vs_sphere, raycast_nearest, Ray, RayHit,
};
pub use shapes::{Aabb3d, Capsule, Collider, Sphere};
pub use slide::{move_and_slide, SlideParams, SlideResult};

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_math::Vec3;
use bevy_reflect::Reflect;
use omm_engine_core::SimSet;

/// Physics plugin: registers collision types for reflection, installs the
/// [`Broadphase`], and runs the controller in [`SimSet::Simulate`].
///
/// Headless-safe: pure deterministic logic, no GPU or device code. Compose on
/// top of `omm_engine_core::EnginePlugins`.
#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        register_physics_types(app);
        app.init_resource::<PhysicsSettings>();
        app.init_resource::<Broadphase>();
        // Broadphase must reflect the latest collider set before the controller
        // reads it, so the two systems are chained within the deterministic sim set.
        app.add_systems(
            bevy_app::FixedUpdate,
            (
                controller::sync_broadphase,
                controller::character_controller_system,
            )
                .chain()
                .in_set(SimSet::Simulate),
        );
    }
}

/// Register every authored physics type with the app's reflection registry, so
/// the MCP editor and agents can enumerate and author them. An unregistered type
/// is invisible to tooling, which is a bug.
fn register_physics_types(app: &mut App) {
    app.register_type::<PhysicsSettings>()
        .register_type::<Aabb3d>()
        .register_type::<Sphere>()
        .register_type::<Capsule>()
        .register_type::<Collider>()
        .register_type::<CharacterController>()
        .register_type::<MoveIntent>();
}

/// Global physics settings: gravity and the fixed timestep the controller
/// integrates over.
#[derive(Debug, Clone, Copy, Resource, Reflect)]
#[reflect(Resource)]
pub struct PhysicsSettings {
    /// Gravity acceleration (metres/second²); `-Y` under normal gravity.
    pub gravity: Vec3,
    /// Fixed timestep (seconds). Must equal the sim tick, or headless re-simulation
    /// integrates over a different `dt` than the client — silent desync.
    pub timestep: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            // Sourced from the engine's fixed tick (30 Hz) so gravity integrates over
            // the same `dt` as the shared sim. The `timestep_matches_engine_tick` test
            // fails loud if these ever drift.
            timestep: omm_engine_core::TICK_DT as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::any::TypeId;

    #[test]
    fn physics_settings_default_is_downward_gravity_at_tick_rate() {
        let settings = PhysicsSettings::default();
        assert!(settings.gravity.y < 0.0);
        assert!(settings.timestep > 0.0);
    }

    /// The physics timestep must equal the engine's fixed tick, or gravity
    /// integrates over a different `dt` than the shared sim — a silent desync.
    #[test]
    fn timestep_matches_engine_tick() {
        let dt = PhysicsSettings::default().timestep;
        assert!((f64::from(dt) - omm_engine_core::TICK_DT).abs() < 1e-6);
    }

    #[test]
    fn plugin_registers_all_authored_types() {
        let mut app = App::new();
        app.add_plugins(PhysicsPlugin);
        let registry = app.world().resource::<AppTypeRegistry>().read();
        for type_id in [
            TypeId::of::<PhysicsSettings>(),
            TypeId::of::<Aabb3d>(),
            TypeId::of::<Sphere>(),
            TypeId::of::<Capsule>(),
            TypeId::of::<Collider>(),
            TypeId::of::<CharacterController>(),
            TypeId::of::<MoveIntent>(),
        ] {
            assert!(
                registry.get(type_id).is_some(),
                "unregistered physics type {type_id:?}"
            );
        }
    }

    #[test]
    fn plugin_installs_broadphase_resource() {
        let mut app = App::new();
        app.add_plugins(PhysicsPlugin);
        assert!(app.world().get_resource::<Broadphase>().is_some());
    }
}
