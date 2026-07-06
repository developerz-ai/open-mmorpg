//! Integration tests — pure collision math: ray casts, slide, step, floor-snap,
//! LOS symmetry, and proptest determinism.
//!
//! All tests are headless (no ECS, no GPU). They exercise the public API the
//! server world-model and the client prediction path share, proving the
//! invariants that make replays and anti-cheat re-simulation safe.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy_math::Vec3;
use omm_engine_physics::{
    capsule_vs_aabb, line_of_sight, move_and_slide, ray_vs_aabb, ray_vs_capsule, ray_vs_sphere,
    raycast_nearest, Aabb3d, Capsule, Collider, Penetration, Ray, SlideParams, Sphere,
};
use proptest::prelude::*;

// ─── helpers ────────────────────────────────────────────────────────────────

fn ray(origin: Vec3, dir: Vec3) -> Ray {
    Ray::new(origin, dir).expect("valid ray")
}

/// A wide floor slab with its top face at y = 0.
fn floor() -> Aabb3d {
    Aabb3d::new(Vec3::new(-50.0, -5.0, -50.0), Vec3::new(50.0, 0.0, 50.0))
}

/// An upright character capsule whose feet sit at `feet`.
fn capsule(feet: Vec3) -> Capsule {
    const RADIUS: f32 = 0.4;
    const HEIGHT: f32 = 1.8;
    Capsule::new(
        feet + Vec3::Y * RADIUS,
        feet + Vec3::Y * (HEIGHT - RADIUS),
        RADIUS,
    )
}

fn feet_of(cap: &Capsule) -> Vec3 {
    cap.a - Vec3::Y * 0.4 // matches RADIUS above
}

// ─── raycast hit / miss ──────────────────────────────────────────────────────

#[test]
fn aabb_hit_from_negative_x() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    let hit = ray_vs_aabb(&ray(Vec3::ZERO, Vec3::X), &b, 100.0).expect("should hit");
    assert!(
        (hit.toi - 4.0).abs() < 1e-4,
        "hit at x=4 (box starts at 4), got {}",
        hit.toi
    );
    // Normal must face the shooter (negative-X face of the box).
    assert!((hit.normal - Vec3::NEG_X).length() < 1e-5);
}

#[test]
fn aabb_miss_when_ray_clears_the_box() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    // Ray shoots along X but at y=5 — well above the unit box centred on y=0.
    assert!(ray_vs_aabb(&ray(Vec3::new(0.0, 5.0, 0.0), Vec3::X), &b, 100.0).is_none());
}

#[test]
fn aabb_hit_respects_max_toi() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    assert!(
        ray_vs_aabb(&ray(Vec3::ZERO, Vec3::X), &b, 3.0).is_none(),
        "max_toi 3 < 4: miss"
    );
    assert!(
        ray_vs_aabb(&ray(Vec3::ZERO, Vec3::X), &b, 5.0).is_some(),
        "max_toi 5 >= 4: hit"
    );
}

#[test]
fn sphere_hit_from_negative_z() {
    let s = Sphere::new(Vec3::new(0.0, 0.0, 10.0), 2.0);
    let hit = ray_vs_sphere(&ray(Vec3::ZERO, Vec3::Z), &s, 100.0).expect("sphere hit");
    assert!(
        (hit.toi - 8.0).abs() < 1e-4,
        "entry at z=8 (10-radius), got {}",
        hit.toi
    );
    assert!((hit.normal - Vec3::NEG_Z).length() < 1e-4);
}

#[test]
fn sphere_miss_when_ray_parallel_and_offset() {
    let s = Sphere::new(Vec3::ZERO, 1.0);
    assert!(ray_vs_sphere(&ray(Vec3::new(0.0, 2.0, -5.0), Vec3::Z), &s, 100.0).is_none());
}

#[test]
fn capsule_body_hit() {
    let cap = Capsule::new(Vec3::new(0.0, -1.0, 5.0), Vec3::new(0.0, 1.0, 5.0), 0.5);
    let hit = ray_vs_capsule(&ray(Vec3::ZERO, Vec3::Z), &cap, 100.0).expect("capsule body hit");
    assert!(
        (hit.toi - 4.5).abs() < 1e-3,
        "entry at z=4.5 (5-radius), got {}",
        hit.toi
    );
}

#[test]
fn capsule_miss_when_offset_beyond_radius() {
    let cap = Capsule::new(Vec3::new(0.0, -1.0, 5.0), Vec3::new(0.0, 1.0, 5.0), 0.5);
    // Ray at y=3 misses the capsule entirely (radius 0.5, spine at y in [-1,1]).
    assert!(ray_vs_capsule(&ray(Vec3::new(0.0, 3.0, 0.0), Vec3::Z), &cap, 100.0).is_none());
}

#[test]
fn raycast_nearest_returns_closest_and_breaks_ties_by_key() {
    // Two identical boxes; the ray hits both at the same toi.
    let b = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(7u64, b), (3u64, b)];
    let (key, hit) = raycast_nearest(&ray(Vec3::ZERO, Vec3::X), 100.0, &colliders).expect("hit");
    assert_eq!(key, 3, "lower key wins on tie");
    assert!((hit.toi - 4.0).abs() < 1e-4);
}

#[test]
fn raycast_nearest_picks_nearer_box_regardless_of_key_order() {
    let near = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::splat(0.5),
    ));
    let far = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(8.0, 0.0, 0.0),
        Vec3::splat(0.5),
    ));
    // Near box has higher key — the toi must still win.
    let colliders = [(99u64, near), (1u64, far)];
    let (key, _) = raycast_nearest(&ray(Vec3::ZERO, Vec3::X), 100.0, &colliders).expect("hit");
    assert_eq!(key, 99, "nearer box wins over lower key");
}

// ─── slide / step / floor-snap ──────────────────────────────────────────────

#[test]
fn slide_falls_and_lands_flush_on_floor() {
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.5, 0.0)),
        Vec3::new(0.0, -1.0, 0.0),
        &[floor()],
        &SlideParams::default(),
    );
    assert!(res.grounded, "should land on the floor");
    let y = feet_of(&res.capsule).y;
    assert!(y.abs() < 1e-2, "feet flush at y=0, got {y}");
}

#[test]
fn slide_stays_airborne_above_snap_distance() {
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 5.0, 0.0)),
        Vec3::new(0.0, -0.5, 0.0), // tiny downward motion, still far above floor
        &[floor()],
        &SlideParams::default(),
    );
    assert!(!res.grounded, "should not land yet");
    let y = feet_of(&res.capsule).y;
    assert!(y > 4.0, "should be above y=4, got {y}");
}

#[test]
fn slide_along_wall_preserves_tangential_motion() {
    let wall = Aabb3d::new(Vec3::new(1.0, -1.0, -10.0), Vec3::new(3.0, 3.0, 10.0));
    let res = move_and_slide(
        capsule(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 1.0), // diagonal into wall
        &[wall],
        &SlideParams::default(),
    );
    let p = feet_of(&res.capsule);
    assert!(p.x < 0.7, "blocked in x by the wall, got x {}", p.x);
    assert!(p.z > 0.5, "tangential +z motion preserved, got z {}", p.z);
}

#[test]
fn slide_step_climbs_low_ledge() {
    let step = Aabb3d::new(Vec3::new(1.0, 0.0, -10.0), Vec3::new(10.0, 0.2, 10.0));
    let res = move_and_slide(
        capsule(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 0.0),
        &[floor(), step],
        &SlideParams::default(),
    );
    let p = feet_of(&res.capsule);
    assert!(res.grounded, "grounded after climbing the step");
    assert!(p.y > 0.15, "climbed onto 0.2 step, got y {}", p.y);
    assert!(p.x > 0.8, "advanced past the step edge, got x {}", p.x);
}

#[test]
fn slide_blocked_by_tall_wall_cannot_step() {
    let wall = Aabb3d::new(Vec3::new(1.0, 0.0, -10.0), Vec3::new(10.0, 2.0, 10.0));
    let res = move_and_slide(
        capsule(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 0.0),
        &[floor(), wall],
        &SlideParams::default(),
    );
    let p = feet_of(&res.capsule);
    assert!(p.x < 0.7, "did not climb the tall wall, got x {}", p.x);
    assert!(p.y < 0.1, "stayed at ground level, got y {}", p.y);
}

#[test]
fn floor_snap_activates_within_snap_distance() {
    // Feet 0.25 above floor, within snap_distance 0.3.
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.25, 0.0)),
        Vec3::new(0.1, 0.0, 0.0), // horizontal motion triggers snap path
        &[floor()],
        &SlideParams::default(),
    );
    assert!(res.grounded, "snapped down to the floor");
    let y = feet_of(&res.capsule).y;
    assert!(y.abs() < 1e-2, "pulled down to y≈0, got {y}");
}

#[test]
fn floor_snap_does_not_fire_beyond_snap_distance() {
    // 0.9 above the floor is farther than snap_distance 0.3.
    let res = move_and_slide(
        capsule(Vec3::new(0.0, 0.9, 0.0)),
        Vec3::new(0.1, 0.0, 0.0),
        &[floor()],
        &SlideParams::default(),
    );
    assert!(!res.grounded, "too far above to snap");
    let y = feet_of(&res.capsule).y;
    assert!(y > 0.5, "stayed airborne, got {y}");
}

// ─── line-of-sight symmetry ──────────────────────────────────────────────────

#[test]
fn los_is_symmetric_clear_path() {
    let colliders: Vec<(u64, Collider)> = vec![];
    let a = Vec3::new(0.0, 1.0, 0.0);
    let b = Vec3::new(10.0, 1.0, 0.0);
    assert_eq!(
        line_of_sight(a, b, &colliders),
        line_of_sight(b, a, &colliders),
        "LOS(A,B) == LOS(B,A) with no obstacles"
    );
}

#[test]
fn los_is_symmetric_blocked_by_wall() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 1.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(0u64, wall)];
    let a = Vec3::new(0.0, 1.0, 0.0);
    let b = Vec3::new(10.0, 1.0, 0.0);
    let ab = line_of_sight(a, b, &colliders);
    let ba = line_of_sight(b, a, &colliders);
    assert!(!ab, "wall blocks A→B");
    assert_eq!(ab, ba, "LOS(A,B) == LOS(B,A)");
}

#[test]
fn los_stops_short_of_endpoint() {
    // Target at x=3; wall at x=5 — the segment never reaches the wall.
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(0u64, wall)];
    assert!(
        line_of_sight(Vec3::ZERO, Vec3::new(3.0, 0.0, 0.0), &colliders),
        "segment ends before the wall → clear"
    );
}

#[test]
fn los_zero_length_segment_is_always_clear() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(10.0)));
    let colliders = [(0u64, wall)];
    assert!(
        line_of_sight(Vec3::ZERO, Vec3::ZERO, &colliders),
        "zero-length segment is trivially clear"
    );
}

// ─── penetration depth sanity ────────────────────────────────────────────────

#[test]
fn capsule_vs_aabb_returns_none_when_separated() {
    let cap = Capsule::new(Vec3::new(0.0, 0.4, 0.0), Vec3::new(0.0, 1.4, 0.0), 0.4);
    let far_box = Aabb3d::new(Vec3::new(5.0, 0.0, 5.0), Vec3::new(6.0, 2.0, 6.0));
    assert!(capsule_vs_aabb(&cap, &far_box).is_none());
}

#[test]
fn capsule_vs_aabb_returns_some_with_positive_depth_when_overlapping() {
    let cap = Capsule::new(Vec3::new(0.0, 0.4, 0.0), Vec3::new(0.0, 1.4, 0.0), 0.4);
    let box_under_cap = Aabb3d::new(Vec3::new(-1.0, -0.1, -1.0), Vec3::new(1.0, 0.5, 1.0));
    let pen: Penetration = capsule_vs_aabb(&cap, &box_under_cap).expect("should overlap");
    assert!(pen.depth > 0.0, "positive penetration depth");
    assert!((pen.normal.length() - 1.0).abs() < 1e-5, "unit normal");
}

// ─── proptest: determinism over arbitrary inputs ─────────────────────────────

proptest! {
    /// `move_and_slide` must return bit-identical results when called twice
    /// with the same inputs — the invariant that makes replay and anti-cheat
    /// re-simulation safe.
    #[test]
    fn slide_is_deterministic(
        fx in -10.0f32..10.0,
        fy in 0.0f32..5.0,
        fz in -10.0f32..10.0,
        mx in -2.0f32..2.0,
        my in -2.0f32..2.0,
        mz in -2.0f32..2.0,
    ) {
        let cap = capsule(Vec3::new(fx, fy, fz));
        let motion = Vec3::new(mx, my, mz);
        let colliders = [floor()];
        let a = move_and_slide(cap, motion, &colliders, &SlideParams::default());
        let b = move_and_slide(cap, motion, &colliders, &SlideParams::default());
        prop_assert_eq!(a.capsule, b.capsule);
        prop_assert_eq!(a.grounded, b.grounded);
    }

    /// Raycast against an axis-aligned box must be deterministic: identical
    /// inputs produce bit-identical outputs (Option<RayHit> equality).
    #[test]
    fn raycast_aabb_is_deterministic(
        ox in -20.0f32..20.0,
        oy in -20.0f32..20.0,
        oz in -20.0f32..20.0,
        dx in -1.0f32..1.0,
        dy in -1.0f32..1.0,
        dz in -1.0f32..1.0,
    ) {
        // Only test with a non-zero direction (skip degenerate cases proptest
        // occasionally generates; they are covered by the unit-test above).
        let dir = Vec3::new(dx, dy, dz);
        if dir.length_squared() < 1e-6 {
            return Ok(());
        }
        let Ok(r) = Ray::new(Vec3::new(ox, oy, oz), dir) else {
            return Ok(());
        };
        let b = Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(1.0));
        let a = ray_vs_aabb(&r, &b, 100.0);
        let c = ray_vs_aabb(&r, &b, 100.0);
        prop_assert_eq!(a, c);
    }

    /// LOS(A,B) == LOS(B,A) for any pair of points in a fixed scene.
    #[test]
    fn los_symmetry(
        ax in -10.0f32..10.0, ay in 0.0f32..5.0, az in -10.0f32..10.0,
        bx in -10.0f32..10.0, by in 0.0f32..5.0, bz in -10.0f32..10.0,
    ) {
        // A static wall in the scene.
        let wall = Collider::Aabb(Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(1.0)));
        let colliders = [(0u64, wall)];
        let a = Vec3::new(ax, ay, az);
        let b = Vec3::new(bx, by, bz);
        let ab = line_of_sight(a, b, &colliders);
        let ba = line_of_sight(b, a, &colliders);
        prop_assert_eq!(ab, ba, "LOS symmetry violated");
    }

    /// `capsule_vs_aabb` contacts: same capsule + same box → identical result.
    #[test]
    fn capsule_contacts_are_deterministic(
        cx in -5.0f32..5.0,
        cy in 0.0f32..5.0,
        cz in -5.0f32..5.0,
        bx in -5.0f32..5.0,
        by in 0.0f32..3.0,
        bz in -5.0f32..5.0,
    ) {
        let cap = Capsule::new(
            Vec3::new(cx, cy + 0.4, cz),
            Vec3::new(cx, cy + 1.4, cz),
            0.4,
        );
        let b = Aabb3d::from_center_half(Vec3::new(bx, by, bz), Vec3::splat(1.0));
        let a_result = capsule_vs_aabb(&cap, &b);
        let b_result = capsule_vs_aabb(&cap, &b);
        prop_assert_eq!(a_result, b_result);
    }
}
