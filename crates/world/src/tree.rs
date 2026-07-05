//! The [`Quadtree`] — the single spatial authority for a zone.
//!
//! Interest management, tile streaming, and shard boundaries all read this one
//! index; per the world-model spec there is never a second, forked index. The
//! tree partitions the `x`/`z` ground plane and answers radius / box queries in
//! `O(nearby)` rather than `O(world)`.

use std::collections::HashMap;

use omm_protocol::Vec3;

use crate::geometry::Aabb;
use crate::node::Node;
use crate::EntityId;

/// A deterministic quadtree over the world ground plane (`x`/`z`).
///
/// Same ordered operations always yield the same state and the same
/// (id-sorted) query results, so the index is safe to drive from the
/// deterministic sim and to re-run for anti-cheat replay.
#[derive(Debug)]
pub struct Quadtree {
    root: Node,
    bounds: Aabb,
    max_depth: u8,
    capacity: usize,
    /// Last known position per entity, so `update`/`remove` can locate the
    /// owning leaf without scanning the tree.
    positions: HashMap<EntityId, (f32, f32)>,
}

impl Quadtree {
    /// Builds an empty tree covering `bounds`.
    ///
    /// - `max_depth` caps subdivision (and thus the smallest cell size).
    /// - `capacity` is the entry count a leaf holds before it splits.
    ///
    /// `capacity` is clamped to at least `1` so a full leaf can always split.
    #[must_use]
    pub fn new(bounds: Aabb, max_depth: u8, capacity: usize) -> Self {
        Self {
            root: Node::new(bounds, 0),
            bounds,
            max_depth,
            capacity: capacity.max(1),
            positions: HashMap::new(),
        }
    }

    /// The world bounds this tree covers.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        self.bounds
    }

    /// The number of tracked entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether the tree tracks no entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Whether `id` is currently tracked.
    #[must_use]
    pub fn contains(&self, id: EntityId) -> bool {
        self.positions.contains_key(&id)
    }

    /// Inserts `id` at `pos`, or moves it there if already tracked (upsert).
    ///
    /// Only `pos.x`/`pos.z` are used; `pos.y` (height) is ignored by the index.
    /// Positions outside [`Quadtree::bounds`] are **clamped** to the nearest
    /// edge so an entity is never silently dropped — callers that treat
    /// out-of-bounds as an error should check [`Aabb::contains`] first.
    pub fn insert(&mut self, id: EntityId, pos: Vec3) {
        let (x, z) = self.bounds.clamp_point(pos.x, pos.z);
        if let Some(&(ox, oz)) = self.positions.get(&id) {
            self.root.remove(id, ox, oz);
        }
        self.root.insert(id, x, z, self.max_depth, self.capacity);
        self.positions.insert(id, (x, z));
    }

    /// Moves an already-tracked entity to `pos`. Identical to [`Quadtree::insert`]
    /// (an upsert); provided as intent-revealing sugar for the movement path.
    pub fn update(&mut self, id: EntityId, pos: Vec3) {
        self.insert(id, pos);
    }

    /// Removes `id` from the index. Returns whether it was tracked.
    pub fn remove(&mut self, id: EntityId) -> bool {
        match self.positions.remove(&id) {
            Some((x, z)) => self.root.remove(id, x, z),
            None => false,
        }
    }

    /// Every entity whose position lies within `radius` of `center`, sorted by
    /// [`EntityId`] for replay-stable output. A non-positive `radius` yields an
    /// empty set.
    #[must_use]
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<EntityId> {
        let mut out = Vec::new();
        if radius > 0.0 {
            self.root.query_circle(center.x, center.z, radius, &mut out);
            out.sort_unstable();
        }
        out
    }

    /// Every entity whose position lies within `area`, sorted by [`EntityId`]
    /// for replay-stable output.
    #[must_use]
    pub fn query_aabb(&self, area: &Aabb) -> Vec<EntityId> {
        let mut out = Vec::new();
        self.root.query_aabb(area, &mut out);
        out.sort_unstable();
        out
    }
}
