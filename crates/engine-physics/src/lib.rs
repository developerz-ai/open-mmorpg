//! Engine physics: in-house collision, kinematic character controller, line-of-sight queries.
//!
//! **Headless-first design:** collision shapes, broadphase queries, kinematic controller,
//! and LOS checks are pure, deterministic logic with no GPU, no window, no external
//! physics solver. Runs in `SimSet::Simulate`, same timestep as server sim.
//!
//! **Broadphase:** reuses `omm_world`'s quadtree spatial index for interest management.
//! **Shapes:** AABB, sphere, capsule; composed into scene entities via components.
//! **Controller:** move-and-slide with auto step-climb and floor-snap; runs as a system.
//! **Queries:** raycasts and shape casts for targeting/LOS; stable contact ordering.
//!
//! Third-party physics solvers (Rapier, Avian) are deliberately avoided — the MMORPG's
//! server sim owns the source of truth; client and server determinism matter more than
//! solver fidelity. See `docs/architecture/decisions/0001-...md`.

mod error;
pub use error::PhysicsError;

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

/// Physics plugin: registers collision shapes, controller, and query systems.
///
/// Headless-safe: runs in `SimSet::Simulate` without GPU or device code.
/// All deterministic logic (shapes, broadphase, controller, LOS) is available
/// in headless builds; no optional features required.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
  fn build(&self, app: &mut App) {
    app
      // Register core physics components.
      .register_type::<PhysicsSettings>();

    // TODO(E7): add collision shape components (AABB, sphere, capsule).
    // TODO(E7): add kinematic controller system (move-and-slide, auto step/snap).
    // TODO(E7): add broadphase queries (raycast, shapecast).
    // TODO(E7): add LOS (line-of-sight) checks with stable ordering.
  }
}

/// Global physics settings: gravity, timestep, broadphase strategy.
#[derive(Debug, Clone, Copy, Resource, Reflect)]
pub struct PhysicsSettings {
  /// Gravity acceleration (m/s²), typically -9.8 on Y axis.
  pub gravity: bevy_math::Vec3,
  /// Fixed timestep (seconds), must match `SimSet` tick.
  pub timestep: f32,
  /// Enable kinematic controller auto-climb; step height in world units.
  pub step_height: f32,
}

impl Default for PhysicsSettings {
  fn default() -> Self {
    Self {
      gravity: bevy_math::Vec3::new(0.0, -9.8, 0.0),
      // TICK_DT must match omm_ecs_core: 1/60 Hz = ~0.01667s.
      // See crates/engine-core/src/lib.rs and crates/ecs-core/src/lib.rs.
      timestep: 1.0 / 60.0,
      step_height: 0.3,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn physics_settings_default() {
    let settings = PhysicsSettings::default();
    assert!(settings.gravity.y < 0.0);
    assert!(settings.timestep > 0.0);
    assert!(settings.step_height > 0.0);
  }

  #[test]
  fn physics_plugin_registers_types() {
    use bevy_ecs::prelude::AppTypeRegistry;

    let mut app = App::new();
    app.add_plugins(PhysicsPlugin);

    // Verify PhysicsSettings is registered.
    let registry = app.world().resource::<AppTypeRegistry>().read();
    assert!(registry.get(std::any::TypeId::of::<PhysicsSettings>()).is_some());
  }
}
