//! Unit tests for closest-point and shape/box penetration math.

use bevy_math::Vec3;

use super::*;
use crate::shapes::{Aabb3d, Capsule, Sphere};

fn unit_box() -> Aabb3d {
    Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(1.0))
}

#[test]
fn closest_point_on_segment_endpoints_and_middle() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 4.0, 0.0);
    assert_eq!(closest_point_on_segment(a, b, Vec3::new(0.0, -2.0, 0.0)), a); // before start
    assert_eq!(closest_point_on_segment(a, b, Vec3::new(0.0, 9.0, 0.0)), b); // past end
    assert_eq!(
        closest_point_on_segment(a, b, Vec3::new(3.0, 2.0, 0.0)),
        Vec3::new(0.0, 2.0, 0.0)
    );
    // Degenerate segment collapses to `a`.
    assert_eq!(closest_point_on_segment(a, a, Vec3::new(1.0, 1.0, 1.0)), a);
}

#[test]
fn sphere_vs_aabb_side_penetration_normal_and_depth() {
    let b = unit_box();
    // Sphere centre just past the +x face by 0.5, radius 1.0 → depth 0.5, normal +x.
    let s = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
    let pen = sphere_vs_aabb(&s, &b).expect("overlap");
    assert!((pen.normal - Vec3::X).length() < 1e-5);
    assert!((pen.depth - 0.5).abs() < 1e-5);
}

#[test]
fn sphere_vs_aabb_miss_returns_none() {
    let b = unit_box();
    assert!(sphere_vs_aabb(&Sphere::new(Vec3::new(3.0, 0.0, 0.0), 1.0), &b).is_none());
    // Non-positive radius never collides.
    assert!(sphere_vs_aabb(&Sphere::new(Vec3::ZERO, 0.0), &b).is_none());
    // NaN centre is rejected, not resolved into NaNs.
    assert!(sphere_vs_aabb(&Sphere::new(Vec3::new(f32::NAN, 0.0, 0.0), 1.0), &b).is_none());
}

#[test]
fn sphere_center_inside_pushes_out_nearest_face() {
    let b = unit_box();
    // Centre near the +y face (0.9): nearest face is +y, gap 0.1 → push +y.
    let s = Sphere::new(Vec3::new(0.0, 0.9, 0.0), 0.5);
    let pen = sphere_vs_aabb(&s, &b).expect("overlap");
    assert!((pen.normal - Vec3::Y).length() < 1e-5);
    // depth = face gap (0.1) + radius (0.5).
    assert!((pen.depth - 0.6).abs() < 1e-5);
}

#[test]
fn separating_by_penetration_removes_overlap() {
    let b = unit_box();
    let s = Sphere::new(Vec3::new(1.4, 0.0, 0.0), 1.0);
    let pen = sphere_vs_aabb(&s, &b).expect("overlap");
    let moved = Sphere::new(s.center + pen.normal * pen.depth, s.radius);
    // After the push-out the residual penetration is negligible.
    match sphere_vs_aabb(&moved, &b) {
        None => {}
        Some(p) => assert!(p.depth < 1e-4, "residual depth {}", p.depth),
    }
}

#[test]
fn capsule_vs_aabb_upright_against_wall() {
    let b = unit_box();
    // Upright capsule spine along y, offset +x so it overlaps the box side.
    let cap = Capsule::new(Vec3::new(1.4, -0.5, 0.0), Vec3::new(1.4, 0.5, 0.0), 0.5);
    let pen = capsule_vs_aabb(&cap, &b).expect("overlap");
    assert!((pen.normal - Vec3::X).length() < 1e-5);
    assert!(pen.depth > 0.0);
}

#[test]
fn capsule_resting_on_top_has_no_penetration() {
    let b = unit_box();
    // Lower cap sphere centre at y = 1 + radius means the cap just touches the top.
    let cap = Capsule::new(Vec3::new(0.0, 1.5, 0.0), Vec3::new(0.0, 2.5, 0.0), 0.5);
    assert!(capsule_vs_aabb(&cap, &b).is_none());
}
