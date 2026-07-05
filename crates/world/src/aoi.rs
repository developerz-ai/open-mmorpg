//! Area-of-interest (AoI): the per-viewer entity set the replication layer
//! consumes each tick.
//!
//! AoI is a first-class read of the one spatial index — not a side effect of a
//! cell visitor. The netcode delta encoder turns the returned set into a
//! per-client snapshot, which is what keeps bandwidth `O(nearby)` instead of
//! `O(world)`.

use omm_protocol::Vec3;

use crate::{EntityId, Quadtree};

impl Quadtree {
    /// The interest set for a viewer at `viewer_pos`: every entity within
    /// `view_radius` on the ground plane, sorted by [`EntityId`].
    ///
    /// The result includes any entity sitting at the viewer's own position; the
    /// caller (which knows the viewer's id) filters *self* if desired. Cell size
    /// is chosen ≈ view radius, so this is a shallow, cheap descent.
    #[must_use]
    pub fn interest_set(&self, viewer_pos: Vec3, view_radius: f32) -> Vec<EntityId> {
        self.query_radius(viewer_pos, view_radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Aabb;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn world() -> Quadtree {
        Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 6, 4)
    }

    #[test]
    fn interest_set_includes_near_excludes_far() {
        let mut tree = world();
        tree.insert(EntityId(1), at(50.0, 50.0)); // the viewer's tile
        tree.insert(EntityId(2), at(52.0, 50.0)); // near
        tree.insert(EntityId(3), at(90.0, 90.0)); // far

        let set = tree.interest_set(at(50.0, 50.0), 5.0);
        assert_eq!(set, vec![EntityId(1), EntityId(2)]);
    }

    #[test]
    fn interest_set_is_empty_on_empty_world() {
        let tree = world();
        assert!(tree.interest_set(at(10.0, 10.0), 25.0).is_empty());
    }

    #[test]
    fn interest_set_is_sorted_by_id() {
        let mut tree = world();
        for id in [7u64, 3, 9, 1, 5] {
            tree.insert(EntityId(id), at(50.0, 50.0));
        }
        let set = tree.interest_set(at(50.0, 50.0), 10.0);
        let ids: Vec<u64> = set.iter().map(|e| e.0).collect();
        assert_eq!(ids, vec![1, 3, 5, 7, 9]);
    }
}
