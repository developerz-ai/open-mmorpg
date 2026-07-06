//! Unit tests for the quadtree-backed broadphase.

use bevy_math::Vec3;

use super::*;

fn bp() -> Broadphase {
    Broadphase::new(
        Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(128.0)),
        6,
        8,
    )
}

fn box_at(center: Vec3, half: f32) -> Aabb3d {
    Aabb3d::from_center_half(center, Vec3::splat(half))
}

#[test]
fn insert_len_get_remove() {
    let mut b = bp();
    assert!(b.is_empty());
    b.insert(1, box_at(Vec3::new(10.0, 0.0, 10.0), 1.0));
    b.insert(2, box_at(Vec3::new(-5.0, 0.0, 3.0), 2.0));
    assert_eq!(b.len(), 2);
    assert!(b.get(1).is_some());
    assert!(b.remove(1));
    assert!(!b.remove(1)); // already gone
    assert_eq!(b.len(), 1);
}

#[test]
fn query_returns_only_overlapping_boxes_sorted() {
    let mut b = bp();
    b.insert(5, box_at(Vec3::new(0.0, 0.0, 0.0), 1.0));
    b.insert(2, box_at(Vec3::new(1.5, 0.0, 0.0), 1.0)); // overlaps origin box region
    b.insert(9, box_at(Vec3::new(50.0, 0.0, 50.0), 1.0)); // far away
    let hits = b.query(&box_at(Vec3::ZERO, 1.0));
    assert_eq!(hits, vec![2, 5]); // sorted ascending, far box excluded
}

#[test]
fn query_finds_large_box_whose_center_is_outside_query() {
    let mut b = bp();
    // A big box centred far from the query but extending into it — the margin
    // expansion must keep it as a candidate.
    b.insert(1, box_at(Vec3::new(8.0, 0.0, 0.0), 9.0)); // spans x in [-1, 17]
    let hits = b.query(&box_at(Vec3::ZERO, 0.5));
    assert_eq!(hits, vec![1]);
}

#[test]
fn query_filters_on_y_axis() {
    let mut b = bp();
    b.insert(1, box_at(Vec3::new(0.0, 100.0, 0.0), 1.0)); // high up
                                                          // Same x/z column, but query is near the ground → no y overlap.
    let hits = b.query(&box_at(Vec3::new(0.0, 0.0, 0.0), 1.0));
    assert!(hits.is_empty());
}

#[test]
fn margin_shrinks_after_removing_the_large_box() {
    let mut b = bp();
    b.insert(1, box_at(Vec3::new(8.0, 0.0, 0.0), 9.0));
    b.insert(2, box_at(Vec3::new(30.0, 0.0, 0.0), 0.5));
    b.remove(1);
    // With the big box gone the small distant box must not be a false candidate.
    assert!(b.query(&box_at(Vec3::ZERO, 0.5)).is_empty());
}

#[test]
fn query_aabbs_returns_world_boxes() {
    let mut b = bp();
    let target = box_at(Vec3::new(1.0, 0.0, 0.0), 1.0);
    b.insert(1, target);
    let out = b.query_aabbs(&box_at(Vec3::ZERO, 1.0));
    assert_eq!(out, vec![target]);
}
