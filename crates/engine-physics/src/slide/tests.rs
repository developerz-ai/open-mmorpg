//! Behavioural tests for move-and-slide: fall/land, wall slide, step, snap.

use bevy_math::Vec3;

use super::*;

const RADIUS: f32 = 0.4;
const HEIGHT: f32 = 1.8;

/// An upright character capsule whose feet sit at `feet`.
fn capsule(feet: Vec3) -> Capsule {
    Capsule::new(
        feet + Vec3::Y * RADIUS,
        feet + Vec3::Y * (HEIGHT - RADIUS),
        RADIUS,
    )
}

/// The feet position of an upright capsule.
fn feet(cap: &Capsule) -> Vec3 {
    cap.a - Vec3::Y * RADIUS
}

/// A wide floor slab with its top face at `y = 0`.
fn floor() -> Aabb3d {
    Aabb3d::new(Vec3::new(-50.0, -5.0, -50.0), Vec3::new(50.0, 0.0, 50.0))
}

#[test]
fn falls_and_lands_flush_on_floor() {
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.5, 0.0)),
        Vec3::new(0.0, -1.0, 0.0),
        &[floor()],
        &SlideParams::default(),
    );
    assert!(res.grounded, "should land");
    assert!(
        feet(&res.capsule).y.abs() < 1e-2,
        "feet rest at floor top, got {}",
        feet(&res.capsule).y
    );
}

#[test]
fn stays_airborne_above_the_floor() {
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 5.0, 0.0)),
        Vec3::new(0.0, -0.5, 0.0),
        &[floor()],
        &SlideParams::default(),
    );
    assert!(!res.grounded);
    assert!((feet(&res.capsule).y - 4.5).abs() < 1e-4);
}

#[test]
fn slides_along_a_wall_instead_of_stopping() {
    // Wall occupying x in [1, 3]; character pushes diagonally into it (+x) and
    // along it (+z).
    let wall = Aabb3d::new(Vec3::new(1.0, -1.0, -10.0), Vec3::new(3.0, 3.0, 10.0));
    let res = move_and_slide(
        capsule(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 1.0),
        &[wall],
        &SlideParams::default(),
    );
    let p = feet(&res.capsule);
    assert!(p.x < 0.7, "blocked in x by the wall, got {}", p.x); // stopped at ~0.6 (1 - radius)
    assert!(p.z > 0.5, "slid along the wall in z, got {}", p.z); // tangential motion preserved
}

#[test]
fn climbs_a_small_step() {
    let step = Aabb3d::new(Vec3::new(1.0, 0.0, -10.0), Vec3::new(10.0, 0.2, 10.0)); // 0.2 < step_height
    let res = move_and_slide(
        capsule(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 0.0),
        &[floor(), step],
        &SlideParams::default(),
    );
    let p = feet(&res.capsule);
    assert!(res.grounded, "grounded on the step");
    assert!(p.y > 0.15, "climbed onto the 0.2 step, got y {}", p.y);
    assert!(p.x > 0.8, "advanced past the step edge, got x {}", p.x);
}

#[test]
fn blocked_by_a_tall_wall() {
    let wall = Aabb3d::new(Vec3::new(1.0, 0.0, -10.0), Vec3::new(10.0, 2.0, 10.0)); // 2.0 > step_height
    let res = move_and_slide(
        capsule(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 0.0),
        &[floor(), wall],
        &SlideParams::default(),
    );
    let p = feet(&res.capsule);
    assert!(p.x < 0.7, "did not climb the tall wall, got x {}", p.x);
    assert!(p.y < 0.1, "stayed at ground level, got y {}", p.y);
}

#[test]
fn snaps_down_onto_ground_when_left_just_above_it() {
    // Feet left 0.25 above the floor (within snap_distance 0.3) while descending.
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.25, 0.0)),
        Vec3::new(0.1, 0.0, 0.0),
        &[floor()],
        &SlideParams::default(),
    );
    assert!(res.grounded, "snapped to the floor");
    assert!(
        feet(&res.capsule).y.abs() < 1e-2,
        "pulled down to floor top, got {}",
        feet(&res.capsule).y
    );
}

#[test]
fn does_not_snap_beyond_snap_distance() {
    // 0.9 above the floor is farther than snap_distance 0.3 → stays airborne.
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.9, 0.0)),
        Vec3::new(0.1, 0.0, 0.0),
        &[floor()],
        &SlideParams::default(),
    );
    assert!(!res.grounded);
    assert!(feet(&res.capsule).y > 0.5);
}

#[test]
fn is_deterministic() {
    let cap = capsule(Vec3::new(0.2, 0.6, -0.3));
    let wall = Aabb3d::new(Vec3::new(1.0, -1.0, -10.0), Vec3::new(3.0, 3.0, 10.0));
    let colliders = [floor(), wall];
    let motion = Vec3::new(1.0, -0.5, 0.7);
    let a = move_and_slide(cap, motion, &colliders, &SlideParams::default());
    let b = move_and_slide(cap, motion, &colliders, &SlideParams::default());
    assert_eq!(a.capsule, b.capsule);
    assert_eq!(a.grounded, b.grounded);
}

#[test]
fn zero_up_axis_falls_back_to_y() {
    let params = SlideParams {
        up: Vec3::ZERO,
        ..SlideParams::default()
    };
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.5, 0.0)),
        Vec3::new(0.0, -1.0, 0.0),
        &[floor()],
        &params,
    );
    assert!(res.grounded);
}
