//! Integration tests — `CharacterController` component math exercised headless
//! (no ECS, no GPU). The ECS tick-loop behaviour is in `controller_sim.rs`.
//!
//! Tests here prove the pure component helpers: capsule construction from feet,
//! inverse (feet_of), slide tuning round-trip, and edge-case capsule shapes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy_math::Vec3;
use omm_engine_physics::{move_and_slide, Aabb3d, CharacterController, SlideParams};
use proptest::prelude::*;

// ─── capsule construction ────────────────────────────────────────────────────

#[test]
fn capsule_at_upright_geometry() {
    let cc = CharacterController::new(0.4, 1.8);
    let feet = Vec3::new(1.0, 2.0, 3.0);
    let cap = cc.capsule_at(feet, Vec3::Y);
    // Spine bottom = feet + radius * up.
    assert!(
        (cap.a - Vec3::new(1.0, 2.4, 3.0)).length() < 1e-5,
        "a = feet + radius*Y, got {:?}",
        cap.a
    );
    // Spine top = feet + (height - radius) * up.
    assert!(
        (cap.b - Vec3::new(1.0, 3.4, 3.0)).length() < 1e-5,
        "b = feet + (h-r)*Y, got {:?}",
        cap.b
    );
    assert_eq!(cap.radius, 0.4);
}

#[test]
fn feet_of_inverts_capsule_at_exactly() {
    let cc = CharacterController::default();
    let origin = Vec3::new(-2.0, 5.0, 7.0);
    let cap = cc.capsule_at(origin, Vec3::Y);
    let back = cc.feet_of(&cap, Vec3::Y);
    assert!(
        (back - origin).length() < 1e-5,
        "feet_of ∘ capsule_at == id, got {:?}",
        back
    );
}

#[test]
fn short_height_collapses_to_sphere_without_panic() {
    // height < 2*radius — spine length must clamp to 0 (a == b).
    let cc = CharacterController::new(1.0, 0.5);
    let cap = cc.capsule_at(Vec3::ZERO, Vec3::Y);
    assert_eq!(cap.a, cap.b, "zero-length spine for undersized capsule");
    assert_eq!(cap.radius, 1.0);
}

#[test]
fn default_controller_field_sanity() {
    let cc = CharacterController::default();
    assert!(cc.radius > 0.0, "radius must be positive");
    assert!(cc.height >= 2.0 * cc.radius, "height must fit both caps");
    assert!(cc.step_height > 0.0);
    assert!(cc.snap_distance > 0.0);
    assert!(
        cc.max_slope_dot > 0.0 && cc.max_slope_dot <= 1.0,
        "slope dot must be a valid cosine"
    );
    assert!(!cc.grounded, "fresh controller starts airborne");
    assert_eq!(cc.vertical_velocity, 0.0, "fresh controller starts at rest");
}

// ─── slide_params sourced from controller ────────────────────────────────────

/// The `SlideParams` built from a `CharacterController` must produce the same
/// behaviour as one constructed manually with the same values.
#[test]
fn slide_params_from_controller_match_manual() {
    let cc = CharacterController::default();
    let from_cc = SlideParams {
        up: Vec3::Y,
        step_height: cc.step_height,
        snap_distance: cc.snap_distance,
        skin: cc.skin,
        max_iterations: 4,
        floor_min_dot: cc.max_slope_dot,
    };
    let floor = Aabb3d::new(Vec3::new(-50.0, -5.0, -50.0), Vec3::new(50.0, 0.0, 50.0));
    let cap = cc.capsule_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y);
    let motion = Vec3::new(0.2, -0.5, 0.0);

    let a = move_and_slide(cap, motion, &[floor], &from_cc);
    let b = move_and_slide(cap, motion, &[floor], &SlideParams::default());

    // Both use the same default values — results must agree.
    assert_eq!(a.grounded, b.grounded, "grounded must agree");
    assert!(
        (a.capsule.a - b.capsule.a).length() < 1e-4,
        "resolved capsule must agree"
    );
}

// ─── controller with non-default capsule sizes ───────────────────────────────

#[test]
fn large_capsule_geometry_is_consistent() {
    let cc = CharacterController::new(0.6, 2.4);
    let feet = Vec3::new(3.0, 10.0, -2.0);
    let cap = cc.capsule_at(feet, Vec3::Y);
    let back = cc.feet_of(&cap, Vec3::Y);
    assert!(
        (back - feet).length() < 1e-4,
        "large capsule round-trip, got {back:?}"
    );
    assert!(cap.b.y > cap.a.y, "b is above a for upright capsule");
}

#[test]
fn tiny_radius_capsule_does_not_panic() {
    let cc = CharacterController::new(0.01, 0.5);
    let cap = cc.capsule_at(Vec3::ZERO, Vec3::Y);
    assert!(cap.radius > 0.0);
    // Ensure the bounding box is well-formed (no NaN/inf).
    let bb = cap.bounding_aabb();
    assert!(bb.min.is_finite() && bb.max.is_finite());
}

// ─── proptest: capsule_at / feet_of round-trip ───────────────────────────────

proptest! {
    /// For any valid (radius, height) pair, feet_of ∘ capsule_at must recover
    /// the original feet position to floating-point precision.
    #[test]
    fn capsule_at_feet_of_round_trip(
        fx in -100.0f32..100.0,
        fy in -100.0f32..100.0,
        fz in -100.0f32..100.0,
        radius in 0.01f32..2.0,
        height_extra in 0.0f32..3.0,  // height = 2*radius + extra
    ) {
        let height = 2.0 * radius + height_extra;
        let cc = CharacterController::new(radius, height);
        let feet = Vec3::new(fx, fy, fz);
        let cap = cc.capsule_at(feet, Vec3::Y);
        let back = cc.feet_of(&cap, Vec3::Y);
        prop_assert!(
            (back - feet).length() < 1e-3,
            "round-trip failed: feet={feet:?} back={back:?}"
        );
    }

    /// capsule_at must always produce a non-negative spine length — i.e., `a.y
    /// <= b.y` for upright (+Y) capsules — regardless of how height and radius
    /// relate.
    #[test]
    fn spine_always_non_negative(
        radius in 0.01f32..5.0,
        height in 0.0f32..10.0,
    ) {
        let cc = CharacterController::new(radius, height);
        let cap = cc.capsule_at(Vec3::ZERO, Vec3::Y);
        prop_assert!(
            cap.a.y <= cap.b.y + 1e-5,
            "a must not be above b: a={:?} b={:?}", cap.a, cap.b
        );
    }
}
