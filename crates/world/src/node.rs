//! The recursive quadtree node — the tree's private storage.
//!
//! Public spatial operations live on [`crate::Quadtree`]; this module only
//! implements the descent. Everything here is pure and allocation-light: a node
//! holds a small `Vec` of entries until it overflows, then subdivides once.

use crate::geometry::Aabb;
use crate::EntityId;

/// One stored entity: its id and its ground-plane position.
type Entry = (EntityId, f32, f32);

/// A node in the quadtree. Either a **leaf** (holds entries) or an **internal**
/// node (holds four children and no entries of its own).
#[derive(Debug)]
pub(crate) struct Node {
    bounds: Aabb,
    depth: u8,
    /// Entries stored directly on this node. Non-empty only on leaves.
    entries: Vec<Entry>,
    /// Child quadrants, indexed to match [`Aabb::quadrant_of`]. `None` on leaves.
    children: Option<Box<[Node; 4]>>,
}

impl Node {
    /// A fresh empty leaf covering `bounds` at tree `depth`.
    pub(crate) fn new(bounds: Aabb, depth: u8) -> Self {
        Self {
            bounds,
            depth,
            entries: Vec::new(),
            children: None,
        }
    }

    /// Inserts `(id, x, z)`, subdividing this leaf if it overflows `capacity`
    /// and has not reached `max_depth`.
    ///
    /// The caller guarantees the point lies within [`Node::bounds`] (the tree
    /// clamps out-of-bounds inserts before descending).
    pub(crate) fn insert(&mut self, id: EntityId, x: f32, z: f32, max_depth: u8, capacity: usize) {
        if let Some(children) = self.children.as_mut() {
            let q = self.bounds.quadrant_of(x, z);
            children[q].insert(id, x, z, max_depth, capacity);
            return;
        }

        self.entries.push((id, x, z));
        if self.entries.len() > capacity && self.depth < max_depth {
            self.subdivide(max_depth, capacity);
        }
    }

    /// Splits this leaf into four children and re-homes its entries.
    fn subdivide(&mut self, max_depth: u8, capacity: usize) {
        let quads = self.bounds.subdivide();
        let next_depth = self.depth + 1;
        let mut children = Box::new([
            Node::new(quads[0], next_depth),
            Node::new(quads[1], next_depth),
            Node::new(quads[2], next_depth),
            Node::new(quads[3], next_depth),
        ]);
        for (id, x, z) in self.entries.drain(..) {
            let q = self.bounds.quadrant_of(x, z);
            children[q].insert(id, x, z, max_depth, capacity);
        }
        self.children = Some(children);
    }

    /// Removes the entry for `id` located at `(x, z)`. Returns whether it was
    /// found. Descent follows the same quadrant rule as insertion.
    pub(crate) fn remove(&mut self, id: EntityId, x: f32, z: f32) -> bool {
        if let Some(children) = self.children.as_mut() {
            let q = self.bounds.quadrant_of(x, z);
            return children[q].remove(id, x, z);
        }
        if let Some(pos) = self.entries.iter().position(|&(e, _, _)| e == id) {
            self.entries.swap_remove(pos);
            return true;
        }
        false
    }

    /// Appends every entity whose position lies within `radius` of `(cx, cz)`
    /// to `out`. Prunes whole subtrees whose bounds miss the query circle.
    pub(crate) fn query_circle(&self, cx: f32, cz: f32, radius: f32, out: &mut Vec<EntityId>) {
        if !self.bounds.intersects_circle(cx, cz, radius) {
            return;
        }
        if let Some(children) = self.children.as_ref() {
            for child in children.iter() {
                child.query_circle(cx, cz, radius, out);
            }
            return;
        }
        let r2 = radius * radius;
        for &(id, x, z) in &self.entries {
            let dx = x - cx;
            let dz = z - cz;
            if dx * dx + dz * dz <= r2 {
                out.push(id);
            }
        }
    }

    /// Appends every entity whose position lies within `area` to `out`. Prunes
    /// whole subtrees whose bounds miss the query box.
    pub(crate) fn query_aabb(&self, area: &Aabb, out: &mut Vec<EntityId>) {
        if !self.bounds.intersects(area) {
            return;
        }
        if let Some(children) = self.children.as_ref() {
            for child in children.iter() {
                child.query_aabb(area, out);
            }
            return;
        }
        for &(id, x, z) in &self.entries {
            if area.contains(x, z) {
                out.push(id);
            }
        }
    }
}
