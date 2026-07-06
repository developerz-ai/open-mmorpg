//! Integration tests — line-of-sight (thin and thick) and scene-query probes.
//!
//! Tests exercise the public API the server authoritative world-model uses for
//! targeting, ability ranges, and interaction clearance — all headless, no ECS.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy_math::Vec3;
use omm_engine_physics::{line_of_sight, Aabb3d, Capsule, Collider, Ray, SceneQuery, Sphere};
use proptest::prelude::*;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn ray(origin: Vec3, dir: Vec3) -> Ray {
    Ray::new(origin, dir).expect("valid ray")
}

/// A room used across several tests — same geometry as the `scene_queries`
/// integration test so the two suites cross-validate.
fn room() -> Vec<(u64, Collider)> {
    vec![
        // Wall left of the doorway.
        (
            10,
            Collider::Aabb(Aabb3d::new(
                Vec3::new(4.5, 0.0, -5.0),
                Vec3::new(5.5, 4.0, -1.0),
            )),
        ),
        // Wall right of the doorway.
        (
            11,
            Collider::Aabb(Aabb3d::new(
                Vec3::new(4.5, 0.0, 1.0),
                Vec3::new(5.5, 4.0, 5.0),
            )),
        ),
        // Round pillar.
        (
            20,
            Collider::Capsule(Capsule::new(
                Vec3::new(8.0, 0.0, 3.0),
                Vec3::new(8.0, 4.0, 3.0),
                0.5,
            )),
        ),
        // Target dummy on the far side of the wall.
        (
            30,
            Collider::Sphere(Sphere::new(Vec3::new(10.0, 1.0, 0.0), 0.5)),
        ),
    ]
}

// ─── basic blocked / clear ───────────────────────────────────────────────────

#[test]
fn los_clear_when_no_colliders() {
    let colliders: Vec<(u64, Collider)> = vec![];
    assert!(line_of_sight(
        Vec3::ZERO,
        Vec3::new(100.0, 0.0, 0.0),
        &colliders
    ));
}

#[test]
fn los_blocked_by_box_wall() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(0u64, wall)];
    assert!(!line_of_sight(
        Vec3::ZERO,
        Vec3::new(10.0, 0.0, 0.0),
        &colliders
    ));
}

#[test]
fn los_clear_over_box_wall() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(0u64, wall)];
    // Shoot at y = 5 — well above the unit box centred at y=0.
    assert!(line_of_sight(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(10.0, 5.0, 0.0),
        &colliders
    ));
}

#[test]
fn los_blocked_by_sphere() {
    let sphere = Collider::Sphere(Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0));
    let colliders = [(0u64, sphere)];
    assert!(!line_of_sight(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        &colliders
    ));
}

#[test]
fn los_blocked_by_capsule() {
    let cap = Collider::Capsule(Capsule::new(
        Vec3::new(5.0, -1.0, 0.0),
        Vec3::new(5.0, 1.0, 0.0),
        0.5,
    ));
    let colliders = [(0u64, cap)];
    assert!(!line_of_sight(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        &colliders
    ));
}

// ─── doorway room ────────────────────────────────────────────────────────────

#[test]
fn los_blocked_by_wall_in_room() {
    let colliders = room();
    // Aim across wall segment 11 (z ∈ [1,5]) at the dummy (z ≈ 0); the wall
    // at z=3 is in the way.
    assert!(!line_of_sight(
        Vec3::new(0.0, 1.0, 3.0),
        Vec3::new(10.0, 1.0, 0.0),
        &colliders
    ));
}

#[test]
fn los_clear_through_doorway_gap() {
    let colliders = room();
    // Thread the doorway (gap z ∈ (-1,1)) to a point past the wall.
    assert!(line_of_sight(
        Vec3::new(0.0, 1.0, 0.5),
        Vec3::new(7.0, 1.0, 0.5),
        &colliders
    ));
}

#[test]
fn thin_los_threads_doorway_that_wide_probe_cannot() {
    let colliders = room();
    let from = Vec3::new(0.0, 1.0, 0.0);
    let to = Vec3::new(8.0, 1.0, 0.0);
    let query = SceneQuery::new(&colliders);
    // Thin ray threads the 2-m wide doorway (gap z ∈ (-1,1)).
    assert!(query.line_of_sight(from, to), "thin LOS should pass");
    // A 1.5-radius sphere (3.0 wide) cannot fit the 2.0-wide gap.
    assert!(
        !query.thick_line_of_sight(from, to, 1.5),
        "3.0-wide probe should be blocked"
    );
    // A 0.5-radius sphere (1.0 wide) fits comfortably.
    assert!(
        query.thick_line_of_sight(from, to, 0.5),
        "1.0-wide probe should pass"
    );
}

// ─── symmetry ────────────────────────────────────────────────────────────────

#[test]
fn los_symmetry_with_multiple_shapes() {
    let colliders = room();
    let pairs = [
        (Vec3::new(0.0, 1.0, 0.0), Vec3::new(10.0, 1.0, 0.0)),
        (Vec3::new(0.0, 1.0, 3.0), Vec3::new(10.0, 1.0, 3.0)),
        (Vec3::new(0.0, 5.0, 0.0), Vec3::new(10.0, 5.0, 0.0)),
    ];
    for (a, b) in pairs {
        assert_eq!(
            line_of_sight(a, b, &colliders),
            line_of_sight(b, a, &colliders),
            "LOS must be symmetric for A={a:?} B={b:?}"
        );
    }
}

// ─── edge cases ──────────────────────────────────────────────────────────────

#[test]
fn los_zero_length_segment_is_clear() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(10.0)));
    let colliders = [(0u64, wall)];
    assert!(
        line_of_sight(Vec3::ZERO, Vec3::ZERO, &colliders),
        "zero-length segment must be trivially clear"
    );
}

#[test]
fn los_segment_endpoint_on_box_surface_is_clear() {
    // End-point touches the box face but does not pass through it.
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(0u64, wall)];
    // Segment stops exactly at x=4 (the entry face of the box).
    assert!(
        line_of_sight(Vec3::ZERO, Vec3::new(4.0, 0.0, 0.0), &colliders),
        "segment stopping at the box face must be clear"
    );
}

// ─── SceneQuery thin + thick LOS order-invariance ────────────────────────────

#[test]
fn scene_query_los_result_is_independent_of_collider_order() {
    let mut colliders = room();
    let from = Vec3::new(0.0, 1.0, 3.0);
    let to = Vec3::new(10.0, 1.0, 0.0);

    let q1 = SceneQuery::new(&colliders);
    let first = q1.line_of_sight(from, to);

    colliders.reverse();
    let q2 = SceneQuery::new(&colliders);
    let reversed = q2.line_of_sight(from, to);

    assert_eq!(
        first, reversed,
        "LOS result must not depend on collider order"
    );
}

#[test]
fn scene_query_raycast_order_invariant() {
    let mut colliders = room();
    let r = ray(Vec3::new(0.0, 1.0, 3.0), Vec3::X);

    let a = SceneQuery::new(&colliders).raycast(&r, 100.0);
    colliders.reverse();
    let b = SceneQuery::new(&colliders).raycast(&r, 100.0);

    assert_eq!(a, b, "raycast must be replay-stable regardless of order");
}

// ─── proptest: LOS symmetry over arbitrary scenes ────────────────────────────

proptest! {
    /// LOS(A,B) == LOS(B,A) for any two points in a deterministic scene — the
    /// Boolean property that makes targeting, AoI, and interaction range checks
    /// safe to evaluate from either endpoint.
    #[test]
    fn los_symmetry_proptest(
        ax in -20.0f32..20.0, ay in 0.1f32..5.0, az in -20.0f32..20.0,
        bx in -20.0f32..20.0, by in 0.1f32..5.0, bz in -20.0f32..20.0,
        wx in -5.0f32..5.0,   wz in -5.0f32..5.0,
    ) {
        let wall = Collider::Aabb(Aabb3d::from_center_half(
            Vec3::new(wx, 1.0, wz),
            Vec3::splat(1.0),
        ));
        let colliders = [(0u64, wall)];
        let a = Vec3::new(ax, ay, az);
        let b = Vec3::new(bx, by, bz);
        prop_assert_eq!(
            line_of_sight(a, b, &colliders),
            line_of_sight(b, a, &colliders),
            "LOS must be symmetric"
        );
    }

    /// `line_of_sight` must be deterministic — calling it twice with identical
    /// inputs returns the same Boolean.
    #[test]
    fn los_is_deterministic(
        ax in -20.0f32..20.0, ay in 0.0f32..5.0, az in -20.0f32..20.0,
        bx in -20.0f32..20.0, by in 0.0f32..5.0, bz in -20.0f32..20.0,
    ) {
        let wall = Collider::Aabb(Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(1.0)));
        let colliders = [(0u64, wall)];
        let a = Vec3::new(ax, ay, az);
        let b = Vec3::new(bx, by, bz);
        let r1 = line_of_sight(a, b, &colliders);
        let r2 = line_of_sight(a, b, &colliders);
        prop_assert_eq!(r1, r2, "LOS must be deterministic");
    }
}
