//! The kinematic character controller — ECS glue over the pure [`crate::slide`]
//! solver.
//!
//! On the client this is **cosmetic**: it predicts and smooths movement while
//! the server owns the authoritative position (the client never asserts state).
//! The same deterministic collision math is available server-side for movement
//! validation. Two systems, chained in [`SimSet::Simulate`](omm_engine_core::SimSet):
//!
//! * [`sync_broadphase`] mirrors static [`Collider`] entities into the
//!   [`Broadphase`] resource (insert on spawn/move, drop on removal).
//! * [`character_controller_system`] integrates gravity, gathers nearby world
//!   boxes from the broadphase, runs move-and-slide, and writes back the feet
//!   position and grounded state.
//!
//! Colliders are read from [`Transform`] translation (world space for unparented
//! static geometry); parenting, rotation, and scale are not applied to broadphase
//! boxes in this batch — an honest gap for axis-aligned world collision.

use bevy_ecs::prelude::*;
use bevy_math::Vec3;
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;

use crate::broadphase::Broadphase;
use crate::shapes::{Capsule, Collider};
use crate::slide::{move_and_slide, SlideParams};
use crate::PhysicsSettings;

/// A kinematic character: an upright capsule that slides, step-climbs, and snaps
/// to the floor. Holds both tuning and the small runtime state (vertical
/// velocity, grounded) the controller integrates each tick.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct CharacterController {
    /// Capsule radius (metres).
    pub radius: f32,
    /// Total capsule height, caps included (metres, `>= 2 * radius`).
    pub height: f32,
    /// Maximum ledge height auto-climbed.
    pub step_height: f32,
    /// Distance the controller snaps down onto ground when descending/idle.
    pub snap_distance: f32,
    /// Rest slop kept against surfaces.
    pub skin: f32,
    /// Minimum `normal · up` for a surface to count as walkable floor.
    pub max_slope_dot: f32,
    /// Terminal fall speed clamp (metres/second), also a tunnelling guard.
    pub max_fall_speed: f32,
    /// Current vertical velocity along up (metres/second); gravity accumulates
    /// here and is zeroed on landing.
    pub vertical_velocity: f32,
    /// Whether the controller rested on walkable floor last tick.
    pub grounded: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            radius: 0.4,
            height: 1.8,
            step_height: 0.3,
            snap_distance: 0.3,
            skin: 0.02,
            max_slope_dot: 0.7,
            max_fall_speed: 55.0,
            vertical_velocity: 0.0,
            grounded: false,
        }
    }
}

impl CharacterController {
    /// A controller with the given capsule size and default tuning.
    #[must_use]
    pub fn new(radius: f32, height: f32) -> Self {
        Self {
            radius,
            height,
            ..Self::default()
        }
    }

    /// The world-space capsule for a character whose feet are at `feet`.
    #[must_use]
    pub fn capsule_at(&self, feet: Vec3, up: Vec3) -> Capsule {
        let spine = (self.height - 2.0 * self.radius).max(0.0);
        let a = feet + up * self.radius;
        Capsule::new(a, a + up * spine, self.radius)
    }

    /// The feet position of a resolved capsule.
    #[must_use]
    pub fn feet_of(&self, capsule: &Capsule, up: Vec3) -> Vec3 {
        capsule.a - up * self.radius
    }
}

/// A per-tick horizontal move request, in metres/second. Gameplay/input writes
/// it; the controller consumes it (any vertical component is ignored — gravity
/// and jumping drive vertical velocity).
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq)]
#[reflect(Component)]
pub struct MoveIntent {
    /// Desired horizontal velocity (metres/second).
    pub desired: Vec3,
}

impl MoveIntent {
    /// A move intent with the given desired velocity.
    #[must_use]
    pub fn new(desired: Vec3) -> Self {
        Self { desired }
    }
}

/// Mirror static colliders into the broadphase: insert/move changed ones, drop
/// removed ones. Characters (which carry [`CharacterController`]) are excluded so
/// the controller never collides with itself.
#[allow(clippy::type_complexity)]
pub fn sync_broadphase(
    mut broadphase: ResMut<Broadphase>,
    changed: Query<
        (Entity, &Collider, &Transform),
        (
            Without<CharacterController>,
            Or<(Added<Collider>, Changed<Collider>, Changed<Transform>)>,
        ),
    >,
    mut removed: RemovedComponents<Collider>,
) {
    for entity in removed.read() {
        broadphase.remove(entity.to_bits());
    }
    for (entity, collider, transform) in &changed {
        let world = collider.translated(transform.translation);
        broadphase.insert(entity.to_bits(), world.bounding_aabb());
    }
}

/// Advance every character controller by one fixed tick: integrate gravity, run
/// move-and-slide against nearby static boxes, write back feet + grounded.
pub fn character_controller_system(
    settings: Res<PhysicsSettings>,
    broadphase: Res<Broadphase>,
    mut characters: Query<(
        &mut Transform,
        &mut CharacterController,
        Option<&MoveIntent>,
    )>,
) {
    let dt = settings.timestep;
    let up = Vec3::Y;
    let gravity_accel = settings.gravity.dot(up); // negative under normal gravity

    for (mut transform, mut controller, intent) in &mut characters {
        // Integrate vertical velocity (clamped to terminal speed).
        controller.vertical_velocity =
            (controller.vertical_velocity + gravity_accel * dt).max(-controller.max_fall_speed);

        let desired = intent.map_or(Vec3::ZERO, |m| m.desired);
        let horizontal = desired - up * desired.dot(up);
        let vertical = up * controller.vertical_velocity;
        let motion = (horizontal + vertical) * dt;

        let feet = transform.translation;
        let capsule = controller.capsule_at(feet, up);

        // Candidate static boxes over the swept region, padded for step/snap probes.
        let pad = Vec3::splat(controller.skin + controller.step_height + controller.snap_distance);
        let region = capsule
            .bounding_aabb()
            .union(&capsule.translated(motion).bounding_aabb())
            .expanded(pad);
        let colliders = broadphase.query_aabbs(&region);

        let params = SlideParams {
            up,
            step_height: controller.step_height,
            snap_distance: controller.snap_distance,
            skin: controller.skin,
            max_iterations: 4,
            floor_min_dot: controller.max_slope_dot,
        };
        let result = move_and_slide(capsule, motion, &colliders, &params);

        transform.translation = controller.feet_of(&result.capsule, up);
        controller.grounded = result.grounded;
        if result.grounded && controller.vertical_velocity < 0.0 {
            controller.vertical_velocity = 0.0;
        }
    }
}

#[cfg(test)]
mod tests;
