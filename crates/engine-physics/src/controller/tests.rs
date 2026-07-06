//! Unit tests for controller component math (ECS behaviour is covered by the
//! integration tests in `tests/`).

use bevy_math::Vec3;

use super::*;

#[test]
fn capsule_at_builds_upright_capsule_from_feet() {
    let cc = CharacterController::new(0.4, 1.8);
    let cap = cc.capsule_at(Vec3::new(1.0, 2.0, 3.0), Vec3::Y);
    assert_eq!(cap.radius, 0.4);
    assert_eq!(cap.a, Vec3::new(1.0, 2.4, 3.0)); // lower spine = feet + radius
    assert_eq!(cap.b, Vec3::new(1.0, 3.4, 3.0)); // upper spine = feet + height - radius
}

#[test]
fn feet_of_inverts_capsule_at() {
    let cc = CharacterController::default();
    let feet = Vec3::new(-2.0, 5.0, 7.0);
    let cap = cc.capsule_at(feet, Vec3::Y);
    let back = cc.feet_of(&cap, Vec3::Y);
    assert!((back - feet).length() < 1e-6);
}

#[test]
fn short_height_collapses_to_a_sphere_without_panicking() {
    // height < 2*radius must not produce a negative spine length.
    let cc = CharacterController::new(1.0, 0.5);
    let cap = cc.capsule_at(Vec3::ZERO, Vec3::Y);
    assert_eq!(cap.a, cap.b); // zero-length spine
}

#[test]
fn defaults_are_sane() {
    let cc = CharacterController::default();
    assert!(cc.radius > 0.0 && cc.height >= 2.0 * cc.radius);
    assert!(cc.step_height > 0.0 && cc.snap_distance > 0.0);
    assert!(cc.max_slope_dot > 0.0 && cc.max_slope_dot <= 1.0);
    assert!(!cc.grounded && cc.vertical_velocity == 0.0);
}

#[test]
fn move_intent_defaults_to_zero() {
    assert_eq!(MoveIntent::default().desired, Vec3::ZERO);
    assert_eq!(MoveIntent::new(Vec3::X).desired, Vec3::X);
}
