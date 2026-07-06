//! Unit tests for the [`SceneQuery`] façade — it must delegate faithfully to the
//! free query functions so the client and server get identical answers.

use super::*;
use crate::query::raycast_nearest;
use crate::shapecast::sphere_cast_nearest;
use crate::shapes::{Aabb3d, Sphere};

fn ray(origin: Vec3, dir: Vec3) -> Ray {
    Ray::new(origin, dir).expect("valid ray")
}

fn scene() -> Vec<(u64, Collider)> {
    vec![
        (
            2,
            Collider::Aabb(Aabb3d::from_center_half(
                Vec3::new(4.0, 0.0, 0.0),
                Vec3::splat(1.0),
            )),
        ),
        (
            5,
            Collider::Sphere(Sphere::new(Vec3::new(20.0, 0.0, 0.0), 1.0)),
        ),
    ]
}

#[test]
fn empty_query_reports_empty() {
    let q = SceneQuery::new(&[]);
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
    assert!(q.raycast(&ray(Vec3::ZERO, Vec3::X), 100.0).is_none());
}

#[test]
fn len_counts_colliders() {
    let s = scene();
    assert_eq!(SceneQuery::new(&s).len(), 2);
}

#[test]
fn raycast_delegates_to_nearest() {
    let s = scene();
    let r = ray(Vec3::ZERO, Vec3::X);
    assert_eq!(
        SceneQuery::new(&s).raycast(&r, 100.0),
        raycast_nearest(&r, 100.0, &s)
    );
}

#[test]
fn sphere_cast_delegates_to_nearest() {
    let s = scene();
    let r = ray(Vec3::ZERO, Vec3::X);
    assert_eq!(
        SceneQuery::new(&s).sphere_cast(&r, 0.5, 100.0),
        sphere_cast_nearest(&r, 0.5, &s, 100.0),
    );
}

#[test]
fn line_of_sight_blocked_by_the_near_box() {
    let s = scene();
    let q = SceneQuery::new(&s);
    // The box at x=4 sits between the origin and x=10.
    assert!(!q.line_of_sight(Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0)));
    // Clearing over it (y=5) is unobstructed.
    assert!(q.line_of_sight(Vec3::ZERO, Vec3::new(10.0, 5.0, 0.0)));
}

#[test]
fn thick_line_of_sight_reaches_wider_than_a_ray() {
    // A ray grazes just past a box; a fat probe of the same path is blocked.
    let s = vec![(
        0u64,
        Collider::Aabb(Aabb3d::from_center_half(
            Vec3::new(0.0, 1.4, 5.0),
            Vec3::splat(1.0),
        )),
    )];
    let q = SceneQuery::new(&s);
    let (from, to) = (Vec3::ZERO, Vec3::new(0.0, 0.0, 10.0));
    assert!(q.line_of_sight(from, to)); // thin ray clears (box floor at y=0.4)
    assert!(!q.thick_line_of_sight(from, to, 0.5)); // 0.5 probe touches it
}
