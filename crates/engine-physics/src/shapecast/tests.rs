//! Unit tests for swept-sphere casts and thick line-of-sight.

use super::*;
use crate::query::line_of_sight;

fn ray(origin: Vec3, dir: Vec3) -> Ray {
    Ray::new(origin, dir).expect("valid ray")
}

/// `a` and `b` agree to within `1e-3` on every component.
fn close(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-3
}

#[test]
fn sphere_cast_vs_sphere_hits_earlier_by_radius() {
    let target = Collider::Sphere(Sphere::new(Vec3::new(0.0, 0.0, 10.0), 2.0));
    let hit = sphere_cast(&ray(Vec3::ZERO, Vec3::Z), 0.5, &target, 100.0).expect("hit");
    // Centre stops one combined radius (2.0 + 0.5) short of the sphere centre.
    assert!((hit.toi - 7.5).abs() < 1e-4);
    // Contact is on the *target* surface (z = 8), not the inflated one.
    assert!(close(hit.point, Vec3::new(0.0, 0.0, 8.0)));
    assert!(close(hit.normal, Vec3::NEG_Z));
}

#[test]
fn sphere_cast_vs_capsule_hits_body() {
    let cap = Capsule::new(Vec3::new(0.0, -1.0, 10.0), Vec3::new(0.0, 1.0, 10.0), 0.5);
    let hit = sphere_cast(
        &ray(Vec3::ZERO, Vec3::Z),
        0.5,
        &Collider::Capsule(cap),
        100.0,
    )
    .expect("hit");
    assert!((hit.toi - 9.0).abs() < 1e-3); // 10 - (0.5 body + 0.5 probe)
    assert!(close(hit.point, Vec3::new(0.0, 0.0, 9.5)));
    assert!(close(hit.normal, Vec3::NEG_Z));
}

#[test]
fn sphere_cast_vs_aabb_flat_face() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    let hit = sphere_cast(&ray(Vec3::ZERO, Vec3::X), 0.5, &Collider::Aabb(b), 100.0).expect("hit");
    assert!((hit.toi - 3.5).abs() < 1e-4); // face at x=4, minus probe radius
    assert!(close(hit.point, Vec3::new(4.0, 0.0, 0.0)));
    assert!(close(hit.normal, Vec3::NEG_X));
}

#[test]
fn sphere_cast_vs_aabb_edge_catches_what_a_ray_misses() {
    // Box ahead on +Z; the ray runs parallel to +Z, offset in x so a *thin* ray
    // slips past the +X face but the swept sphere clips the box's near edge.
    let b = Aabb3d::from_center_half(Vec3::new(0.0, 0.0, 5.0), Vec3::splat(1.0));
    let r = ray(Vec3::new(1.4, 0.0, 0.0), Vec3::Z);
    assert!(Collider::Aabb(b).raycast(&r, 100.0).is_none()); // thin ray misses
    let hit = sphere_cast(&r, 0.5, &Collider::Aabb(b), 100.0).expect("edge hit");
    assert!((hit.toi - 3.7).abs() < 1e-3);
    assert!(close(hit.point, Vec3::new(1.0, 0.0, 4.0))); // exactly on the box edge
    assert!(close(hit.normal, Vec3::new(0.8, 0.0, -0.6)));
}

#[test]
fn sphere_cast_vs_aabb_corner_refines_to_vertex() {
    let b = Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(1.0));
    let r = ray(Vec3::new(1.3, 1.3, -3.0), Vec3::Z);
    assert!(Collider::Aabb(b).raycast(&r, 100.0).is_none()); // thin ray misses
    let hit = sphere_cast(&r, 0.5, &Collider::Aabb(b), 100.0).expect("corner hit");
    assert!((hit.toi - 1.7354).abs() < 1e-3);
    assert!(close(hit.point, Vec3::new(1.0, 1.0, -1.0))); // exactly the box corner
    assert!(hit.normal.is_normalized());
    assert!(hit.normal.dot(Vec3::Z) < 0.0); // points back toward the ray
}

#[test]
fn sphere_cast_zero_radius_is_a_raycast() {
    let b = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let r = ray(Vec3::ZERO, Vec3::X);
    let thin = b.raycast(&r, 100.0).expect("ray hit");
    let cast = sphere_cast(&r, 0.0, &b, 100.0).expect("cast hit");
    assert!((thin.toi - cast.toi).abs() < 1e-6);
    assert!(close(thin.point, cast.point));
}

#[test]
fn sphere_cast_misses_when_gap_exceeds_radius() {
    let b = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::splat(1.0),
    ));
    // Offset x=2.0: 1.0 clear of the +X face, wider than the 0.5 probe.
    assert!(sphere_cast(&ray(Vec3::new(2.0, 0.0, 0.0), Vec3::Z), 0.5, &b, 100.0).is_none());
}

#[test]
fn sphere_cast_nearest_breaks_ties_by_key() {
    let s = Collider::Sphere(Sphere::new(Vec3::new(0.0, 0.0, 10.0), 1.0));
    let colliders = [(7u64, s), (3u64, s)];
    let (key, _) =
        sphere_cast_nearest(&ray(Vec3::ZERO, Vec3::Z), 0.5, &colliders, 100.0).expect("hit");
    assert_eq!(key, 3);
}

#[test]
fn sphere_cast_nearest_is_order_independent() {
    let near = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let far = Collider::Sphere(Sphere::new(Vec3::new(20.0, 0.0, 0.0), 1.0));
    let r = ray(Vec3::ZERO, Vec3::X);
    let a = sphere_cast_nearest(&r, 0.5, &[(1, near), (2, far)], 100.0).expect("hit");
    let b = sphere_cast_nearest(&r, 0.5, &[(2, far), (1, near)], 100.0).expect("hit");
    assert_eq!(a.0, b.0);
    assert!((a.1.toi - b.1.toi).abs() < 1e-6);
}

#[test]
fn thick_los_blocked_by_gap_narrower_than_probe() {
    // A 0.8-wide doorway: a thin ray passes down the centre, a 0.5-radius sphere
    // (needs 1.0 of clearance) cannot.
    let left = Collider::Aabb(Aabb3d::new(
        Vec3::new(-3.0, -1.0, 4.0),
        Vec3::new(-0.4, 1.0, 6.0),
    ));
    let right = Collider::Aabb(Aabb3d::new(
        Vec3::new(0.4, -1.0, 4.0),
        Vec3::new(3.0, 1.0, 6.0),
    ));
    let scene = [(0u64, left), (1u64, right)];
    let (from, to) = (Vec3::ZERO, Vec3::new(0.0, 0.0, 10.0));
    assert!(line_of_sight(from, to, &scene)); // thin ray threads the gap
    assert!(!thick_line_of_sight(from, to, 0.5, &scene)); // sphere is too wide
}

#[test]
fn thick_los_zero_radius_matches_line_of_sight() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let scene = [(0u64, wall)];
    for to in [Vec3::new(10.0, 0.0, 0.0), Vec3::new(10.0, 5.0, 0.0)] {
        assert_eq!(
            thick_line_of_sight(Vec3::ZERO, to, 0.0, &scene),
            line_of_sight(Vec3::ZERO, to, &scene),
        );
    }
}

#[test]
fn collider_method_matches_free_function() {
    let b = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let r = ray(Vec3::ZERO, Vec3::X);
    assert_eq!(
        b.sphere_cast(&r, 0.5, 100.0),
        sphere_cast(&r, 0.5, &b, 100.0)
    );
}

#[test]
fn sphere_cast_is_deterministic() {
    let b = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::splat(1.0),
    ));
    let r = ray(Vec3::new(1.3, 0.7, 0.0), Vec3::Z);
    assert_eq!(
        sphere_cast(&r, 0.5, &b, 100.0),
        sphere_cast(&r, 0.5, &b, 100.0)
    );
}
