//! Unit tests for ray casts and line-of-sight.

use bevy_math::Vec3;

use super::*;

fn ray(origin: Vec3, dir: Vec3) -> Ray {
    Ray::new(origin, dir).expect("valid ray")
}

#[test]
fn ray_new_rejects_zero_direction() {
    assert!(Ray::new(Vec3::ZERO, Vec3::ZERO).is_err());
    assert!(Ray::new(Vec3::ZERO, Vec3::X).is_ok());
}

#[test]
fn ray_hits_box_face_with_normal() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    let hit = ray_vs_aabb(&ray(Vec3::ZERO, Vec3::X), &b, 100.0).expect("hit");
    assert!((hit.toi - 4.0).abs() < 1e-4);
    assert!((hit.normal - Vec3::NEG_X).length() < 1e-5); // entry face faces the ray
}

#[test]
fn ray_misses_box_when_offset() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    assert!(ray_vs_aabb(&ray(Vec3::new(0.0, 5.0, 0.0), Vec3::X), &b, 100.0).is_none());
}

#[test]
fn ray_respects_max_toi() {
    let b = Aabb3d::from_center_half(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0));
    assert!(ray_vs_aabb(&ray(Vec3::ZERO, Vec3::X), &b, 3.0).is_none());
    assert!(ray_vs_aabb(&ray(Vec3::ZERO, Vec3::X), &b, 5.0).is_some());
}

#[test]
fn ray_hits_sphere() {
    let s = Sphere::new(Vec3::new(0.0, 0.0, 10.0), 2.0);
    let hit = ray_vs_sphere(&ray(Vec3::ZERO, Vec3::Z), &s, 100.0).expect("hit");
    assert!((hit.toi - 8.0).abs() < 1e-4);
    assert!((hit.normal - Vec3::NEG_Z).length() < 1e-4);
}

#[test]
fn ray_hits_capsule_body_and_caps() {
    let cap = Capsule::new(Vec3::new(0.0, -1.0, 5.0), Vec3::new(0.0, 1.0, 5.0), 0.5);
    // Through the cylindrical body (mid-height).
    let body = ray_vs_capsule(&ray(Vec3::ZERO, Vec3::Z), &cap, 100.0).expect("body hit");
    assert!((body.toi - 4.5).abs() < 1e-3);
    // Above the spine — only the top cap can catch it.
    let cap_hit = ray_vs_capsule(&ray(Vec3::new(0.0, 1.4, 0.0), Vec3::Z), &cap, 100.0);
    assert!(cap_hit.is_some());
}

#[test]
fn raycast_nearest_breaks_ties_by_key() {
    // Two identical boxes at the same distance; the lower key must win.
    let b = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(7u64, b), (3u64, b)];
    let (key, _) = raycast_nearest(&ray(Vec3::ZERO, Vec3::X), 100.0, &colliders).expect("hit");
    assert_eq!(key, 3);
}

#[test]
fn line_of_sight_blocked_and_clear() {
    let wall = Collider::Aabb(Aabb3d::from_center_half(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::splat(1.0),
    ));
    let colliders = [(0u64, wall)];
    assert!(!line_of_sight(
        Vec3::ZERO,
        Vec3::new(10.0, 0.0, 0.0),
        &colliders
    )); // wall between
    assert!(line_of_sight(
        Vec3::ZERO,
        Vec3::new(10.0, 5.0, 0.0),
        &colliders
    )); // over the wall
    assert!(line_of_sight(
        Vec3::ZERO,
        Vec3::new(3.0, 0.0, 0.0),
        &colliders
    )); // stops short of wall
    assert!(line_of_sight(Vec3::ZERO, Vec3::ZERO, &colliders)); // zero-length is clear
}
