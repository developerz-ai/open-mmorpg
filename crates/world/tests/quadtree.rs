//! End-to-end tests for the `omm-world` quadtree, including the key correctness
//! proof: a radius query must agree with a brute-force O(n) distance scan.

use omm_protocol::Vec3;
use omm_world::{Aabb, EntityId, Quadtree};
use proptest::prelude::*;

fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 7.0, z } // non-zero y proves height is ignored by the index
}

fn world() -> Quadtree {
    // Small capacity forces real subdivision under test.
    Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 8, 2)
}

#[test]
fn insert_query_remove_round_trip() {
    let mut tree = world();
    assert!(tree.is_empty());

    tree.insert(EntityId(10), at(20.0, 20.0));
    tree.insert(EntityId(11), at(21.0, 20.0));
    assert_eq!(tree.len(), 2);
    assert!(tree.contains(EntityId(10)));

    let near = tree.query_radius(at(20.0, 20.0), 3.0);
    assert_eq!(near, vec![EntityId(10), EntityId(11)]);

    assert!(tree.remove(EntityId(11)));
    assert!(!tree.remove(EntityId(11))); // second remove is a no-op
    assert_eq!(tree.query_radius(at(20.0, 20.0), 3.0), vec![EntityId(10)]);
    assert_eq!(tree.len(), 1);
}

#[test]
fn radius_includes_near_excludes_far() {
    let mut tree = world();
    tree.insert(EntityId(1), at(50.0, 50.0));
    tree.insert(EntityId(2), at(53.0, 50.0)); // 3.0 away — inside r=5
    tree.insert(EntityId(3), at(60.0, 50.0)); // 10.0 away — outside r=5

    assert_eq!(
        tree.query_radius(at(50.0, 50.0), 5.0),
        vec![EntityId(1), EntityId(2)]
    );
}

#[test]
fn update_moves_the_entity() {
    let mut tree = world();
    tree.insert(EntityId(1), at(10.0, 10.0));
    tree.update(EntityId(1), at(90.0, 90.0));

    assert_eq!(tree.len(), 1);
    assert!(tree.query_radius(at(10.0, 10.0), 5.0).is_empty());
    assert_eq!(tree.query_radius(at(90.0, 90.0), 5.0), vec![EntityId(1)]);
}

#[test]
fn query_on_empty_tree_is_empty() {
    let tree = world();
    assert!(tree.query_radius(at(50.0, 50.0), 50.0).is_empty());
    assert!(tree
        .query_aabb(&Aabb::new(0.0, 0.0, 100.0, 100.0))
        .is_empty());
}

#[test]
fn non_positive_radius_yields_nothing() {
    let mut tree = world();
    tree.insert(EntityId(1), at(50.0, 50.0));
    assert!(tree.query_radius(at(50.0, 50.0), 0.0).is_empty());
    assert!(tree.query_radius(at(50.0, 50.0), -5.0).is_empty());
}

#[test]
fn out_of_bounds_insert_is_clamped_not_dropped() {
    let mut tree = world();
    // Far outside the world; clamps to the (100, 100) corner.
    tree.insert(EntityId(1), at(9999.0, 9999.0));
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.query_radius(at(100.0, 100.0), 1.0), vec![EntityId(1)]);
}

#[test]
fn boundary_points_are_found_once() {
    let mut tree = world();
    // Corner and mid-line points — the delicate quadrant seams.
    tree.insert(EntityId(1), at(0.0, 0.0));
    tree.insert(EntityId(2), at(50.0, 50.0));
    tree.insert(EntityId(3), at(100.0, 100.0));

    let all = tree.query_aabb(&Aabb::new(0.0, 0.0, 100.0, 100.0));
    assert_eq!(all, vec![EntityId(1), EntityId(2), EntityId(3)]);
}

#[test]
fn aabb_query_selects_only_inside() {
    let mut tree = world();
    tree.insert(EntityId(1), at(10.0, 10.0));
    tree.insert(EntityId(2), at(40.0, 40.0));
    tree.insert(EntityId(3), at(80.0, 80.0));

    let hits = tree.query_aabb(&Aabb::new(5.0, 5.0, 45.0, 45.0));
    assert_eq!(hits, vec![EntityId(1), EntityId(2)]);
}

/// Brute-force reference: the exact set the quadtree must reproduce.
fn naive_radius(points: &[(u64, f32, f32)], cx: f32, cz: f32, radius: f32) -> Vec<EntityId> {
    if radius <= 0.0 {
        return Vec::new();
    }
    let r2 = radius * radius;
    let mut out: Vec<EntityId> = points
        .iter()
        .filter(|&&(_, x, z)| {
            let dx = x - cx;
            let dz = z - cz;
            dx * dx + dz * dz <= r2
        })
        .map(|&(id, _, _)| EntityId(id))
        .collect();
    out.sort_unstable();
    out
}

proptest! {
    /// The heart of the suite: for arbitrary point clouds and query circles, the
    /// quadtree must return exactly what an O(n) scan returns.
    #[test]
    fn query_radius_matches_brute_force(
        coords in prop::collection::vec((0.0f32..100.0, 0.0f32..100.0), 0..200),
        cx in -20.0f32..120.0,
        cz in -20.0f32..120.0,
        radius in 0.0f32..150.0,
    ) {
        // Index by position so ids are unique (no accidental upsert).
        let points: Vec<(u64, f32, f32)> = coords
            .iter()
            .enumerate()
            .map(|(i, &(x, z))| (i as u64, x, z))
            .collect();

        let mut tree = Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 8, 4);
        for &(id, x, z) in &points {
            tree.insert(EntityId(id), at(x, z));
        }

        let got = tree.query_radius(at(cx, cz), radius);
        let want = naive_radius(&points, cx, cz, radius);
        prop_assert_eq!(got, want);
    }

    /// Removing entities keeps the index consistent with a shrinking scan.
    #[test]
    fn remove_keeps_index_consistent(
        coords in prop::collection::vec((0.0f32..100.0, 0.0f32..100.0), 1..80),
        remove_mask in prop::collection::vec(any::<bool>(), 1..80),
    ) {
        let points: Vec<(u64, f32, f32)> = coords
            .iter()
            .enumerate()
            .map(|(i, &(x, z))| (i as u64, x, z))
            .collect();

        let mut tree = Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 8, 4);
        for &(id, x, z) in &points {
            tree.insert(EntityId(id), at(x, z));
        }

        let mut survivors = points.clone();
        for (i, remove) in remove_mask.iter().enumerate() {
            if *remove && (i as u64) < points.len() as u64 {
                tree.remove(EntityId(i as u64));
                survivors.retain(|&(id, _, _)| id != i as u64);
            }
        }

        let got = tree.query_radius(at(50.0, 50.0), 200.0);
        let want = naive_radius(&survivors, 50.0, 50.0, 200.0);
        prop_assert_eq!(got, want);
    }
}
