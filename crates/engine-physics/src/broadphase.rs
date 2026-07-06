//! Broadphase — prune the collider set to a small candidate list before the
//! exact (narrowphase) test.
//!
//! Rather than fork a second spatial structure, this **reuses [`omm_world`]'s
//! quadtree** — the same deterministic `x`/`z` index the server uses for
//! interest management (CLAUDE.md: one index, no drift). The quadtree stores
//! *points* (collider centres); to find every box that overlaps a query region
//! we expand the query by the largest collider half-extent seen, so no
//! overlapping box is ever missed, then confirm each candidate with an exact 3D
//! [`Aabb3d::intersects`] test that also filters the `y` axis the quadtree
//! ignores. Candidates come back sorted by key for replay-stable output.

use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use bevy_math::Vec3;
use omm_protocol::Vec3 as WorldVec3;
use omm_world::{Aabb as GroundAabb, EntityId, Quadtree};

use crate::shapes::Aabb3d;

/// Default world half-extent (metres) used by [`Broadphase::default`]. Large
/// enough for a zone chunk; callers with known bounds use [`Broadphase::new`].
const DEFAULT_HALF_EXTENT: f32 = 4096.0;
/// Default quadtree depth cap and leaf capacity — mirrors the world spec's
/// interest-management tree.
const DEFAULT_DEPTH: u8 = 8;
/// Default leaf capacity before a quadtree cell splits.
const DEFAULT_CAPACITY: usize = 16;

/// A spatial index of static collider world-AABBs, keyed by an opaque `u64`
/// (the controller uses `Entity::to_bits()`).
#[derive(Resource)]
pub struct Broadphase {
    tree: Quadtree,
    entries: HashMap<u64, Aabb3d>,
    /// Largest collider half-extent on `x`/`z`, used to expand queries so no
    /// overlapping box is pruned.
    margin: Vec3,
}

impl Default for Broadphase {
    fn default() -> Self {
        Self::new(
            Aabb3d::from_center_half(Vec3::ZERO, Vec3::splat(DEFAULT_HALF_EXTENT)),
            DEFAULT_DEPTH,
            DEFAULT_CAPACITY,
        )
    }
}

impl Broadphase {
    /// Builds an empty broadphase covering `bounds` on the `x`/`z` plane.
    #[must_use]
    pub fn new(bounds: Aabb3d, max_depth: u8, capacity: usize) -> Self {
        let ground = GroundAabb::new(bounds.min.x, bounds.min.z, bounds.max.x, bounds.max.z);
        Self {
            tree: Quadtree::new(ground, max_depth, capacity),
            entries: HashMap::new(),
            margin: Vec3::ZERO,
        }
    }

    /// Number of tracked colliders.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no colliders are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The tracked world-AABB for `key`, if any.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<Aabb3d> {
        self.entries.get(&key).copied()
    }

    /// Inserts or moves the collider `key`, storing its world-space AABB.
    pub fn insert(&mut self, key: u64, aabb: Aabb3d) {
        let c = aabb.center();
        self.tree.insert(
            EntityId(key),
            WorldVec3 {
                x: c.x,
                y: c.y,
                z: c.z,
            },
        );
        let half = aabb.half_extents();
        self.margin = self.margin.max(half);
        self.entries.insert(key, aabb);
    }

    /// Removes the collider `key`. Returns whether it was tracked.
    pub fn remove(&mut self, key: u64) -> bool {
        let removed = self.tree.remove(EntityId(key));
        if self.entries.remove(&key).is_some() {
            self.recompute_margin();
            return true;
        }
        removed
    }

    /// Keys of colliders whose world-AABB overlaps `area`, sorted ascending.
    ///
    /// The quadtree prunes on `x`/`z`; the exact [`Aabb3d::intersects`] test then
    /// rejects false positives and filters the `y` axis.
    #[must_use]
    pub fn query(&self, area: &Aabb3d) -> Vec<u64> {
        let ground = GroundAabb::new(
            area.min.x - self.margin.x,
            area.min.z - self.margin.z,
            area.max.x + self.margin.x,
            area.max.z + self.margin.z,
        );
        self.tree
            .query_aabb(&ground)
            .into_iter()
            .filter(|e| self.entries.get(&e.0).is_some_and(|a| a.intersects(area)))
            .map(|e| e.0)
            .collect()
    }

    /// The world-AABBs for `keys`, in the same order (missing keys skipped).
    #[must_use]
    pub fn world_aabbs(&self, keys: &[u64]) -> Vec<Aabb3d> {
        keys.iter()
            .filter_map(|k| self.entries.get(k).copied())
            .collect()
    }

    /// The world-AABBs overlapping `area`, sorted by key — the controller's
    /// narrowphase input in one call.
    #[must_use]
    pub fn query_aabbs(&self, area: &Aabb3d) -> Vec<Aabb3d> {
        self.world_aabbs(&self.query(area))
    }

    /// Recomputes the query margin after a removal so it stays tight.
    fn recompute_margin(&mut self) {
        self.margin = self
            .entries
            .values()
            .fold(Vec3::ZERO, |acc, a| acc.max(a.half_extents()));
    }
}

#[cfg(test)]
mod tests;
