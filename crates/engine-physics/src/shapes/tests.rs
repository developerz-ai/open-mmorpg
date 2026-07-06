//! Unit tests for collision shape types and their inherent geometry.

use bevy_math::Vec3;

use super::*;

fn unit_box() -> Aabb3d {
    Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(1.0))
}

#[test]
fn aabb_normalises_and_reports_center() {
    let b = Aabb3d::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(-1.0, 0.0, 1.0));
    assert_eq!(b.min, Vec3::new(-1.0, 0.0, 1.0));
    assert_eq!(b.max, Vec3::new(2.0, 2.0, 2.0));
    assert_eq!(b.center(), Vec3::new(0.5, 1.0, 1.5));
}

#[test]
fn aabb_contains_and_intersects() {
    let b = unit_box();
    assert!(b.contains_point(Vec3::ZERO));
    assert!(b.contains_point(Vec3::splat(1.0)));
    assert!(!b.contains_point(Vec3::splat(1.1)));
    assert!(b.intersects(&b.translated(Vec3::new(1.5, 0.0, 0.0))));
    assert!(!b.intersects(&b.translated(Vec3::new(2.1, 0.0, 0.0))));
}

#[test]
fn closest_point_clamps_into_box() {
    let b = unit_box();
    assert_eq!(
        b.closest_point(Vec3::new(5.0, 0.0, -5.0)),
        Vec3::new(1.0, 0.0, -1.0)
    );
    assert_eq!(
        b.closest_point(Vec3::new(0.2, 0.3, 0.4)),
        Vec3::new(0.2, 0.3, 0.4)
    );
}

#[test]
fn expanded_and_union_grow_the_box() {
    let b = unit_box();
    let grown = b.expanded(Vec3::splat(0.5));
    assert_eq!(grown.min, Vec3::splat(-1.5));
    assert_eq!(grown.max, Vec3::splat(1.5));
    let far = Aabb3d::from_center_half(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0));
    let u = b.union(&far);
    assert_eq!(u.min.x, -1.0);
    assert_eq!(u.max.x, 11.0);
}

#[test]
fn collider_bounding_and_translate_and_validate() {
    let c = Collider::Capsule(Capsule::new(Vec3::ZERO, Vec3::new(0.0, 2.0, 0.0), 0.5));
    let bb = c.bounding_aabb();
    assert_eq!(bb.min, Vec3::new(-0.5, -0.5, -0.5));
    assert_eq!(bb.max, Vec3::new(0.5, 2.5, 0.5));
    let moved = c.translated(Vec3::new(10.0, 0.0, 0.0));
    assert_eq!(moved.bounding_aabb().center().x, 10.0);
    assert!(c.validate().is_ok());
    assert!(Collider::Sphere(Sphere::new(Vec3::ZERO, -1.0))
        .validate()
        .is_err());
    assert!(
        Collider::Sphere(Sphere::new(Vec3::new(f32::NAN, 0.0, 0.0), 1.0))
            .validate()
            .is_err()
    );
}
