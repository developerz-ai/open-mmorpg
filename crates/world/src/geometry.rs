//! Axis-aligned rectangle math on the world **ground plane** (`x`/`z`).
//!
//! The world model treats `y` as *up*, so all spatial partitioning happens in
//! the `x`/`z` plane. This module is pure `f32` geometry with no allocation and
//! no I/O — the deterministic bedrock the quadtree stands on.

/// An axis-aligned bounding box on the `x`/`z` ground plane.
///
/// Bounds are **inclusive** on every edge. Sibling quadtree cells therefore
/// share their touching edges; a point on a shared edge is *stored* in exactly
/// one cell (see [`Aabb::quadrant_of`]) but *matched* by a query against either,
/// so no entity is ever double-counted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Minimum `x` (west edge).
    pub min_x: f32,
    /// Minimum `z` (south edge).
    pub min_z: f32,
    /// Maximum `x` (east edge).
    pub max_x: f32,
    /// Maximum `z` (north edge).
    pub max_z: f32,
}

impl Aabb {
    /// Builds a box, normalising so `min <= max` on both axes.
    #[must_use]
    pub fn new(min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_z: min_z.min(max_z),
            max_x: min_x.max(max_x),
            max_z: min_z.max(max_z),
        }
    }

    /// Builds a box from a centre point and half-extents on each axis.
    #[must_use]
    pub fn from_center(cx: f32, cz: f32, half_x: f32, half_z: f32) -> Self {
        Self::new(cx - half_x, cz - half_z, cx + half_x, cz + half_z)
    }

    /// The centre `x`.
    #[must_use]
    pub fn mid_x(&self) -> f32 {
        (self.min_x + self.max_x) * 0.5
    }

    /// The centre `z`.
    #[must_use]
    pub fn mid_z(&self) -> f32 {
        (self.min_z + self.max_z) * 0.5
    }

    /// Whether the point lies inside the box (edges inclusive).
    #[must_use]
    pub fn contains(&self, x: f32, z: f32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }

    /// Whether this box shares any area with `other` (touching edges count).
    #[must_use]
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_z <= other.max_z
            && self.max_z >= other.min_z
    }

    /// Whether the circle `(cx, cz, radius)` overlaps this box.
    ///
    /// Uses the closest-point-on-box test: clamp the centre into the box, then
    /// compare the squared gap against `radius²`. A non-negative `radius` is
    /// assumed; a negative radius simply never intersects.
    #[must_use]
    pub fn intersects_circle(&self, cx: f32, cz: f32, radius: f32) -> bool {
        let nearest_x = cx.clamp(self.min_x, self.max_x);
        let nearest_z = cz.clamp(self.min_z, self.max_z);
        let dx = cx - nearest_x;
        let dz = cz - nearest_z;
        dx * dx + dz * dz <= radius * radius
    }

    /// Clamps a point to the nearest position inside the box.
    ///
    /// Used to keep out-of-bounds inserts tracked at the world edge rather than
    /// dropping them (see [`crate::Quadtree::insert`]).
    #[must_use]
    pub fn clamp_point(&self, x: f32, z: f32) -> (f32, f32) {
        (
            x.clamp(self.min_x, self.max_x),
            z.clamp(self.min_z, self.max_z),
        )
    }

    /// The index `0..4` of the child quadrant a point falls into.
    ///
    /// Layout: `0 = SW`, `1 = SE`, `2 = NW`, `3 = NE`. A point on a mid-line is
    /// assigned to the higher (east/north) quadrant, deterministically.
    #[must_use]
    pub fn quadrant_of(&self, x: f32, z: f32) -> usize {
        let east = usize::from(x >= self.mid_x());
        let north = usize::from(z >= self.mid_z());
        east + north * 2
    }

    /// The four child boxes, indexed to match [`Aabb::quadrant_of`].
    #[must_use]
    pub fn subdivide(&self) -> [Aabb; 4] {
        let mx = self.mid_x();
        let mz = self.mid_z();
        [
            Aabb::new(self.min_x, self.min_z, mx, mz), // 0 SW
            Aabb::new(mx, self.min_z, self.max_x, mz), // 1 SE
            Aabb::new(self.min_x, mz, mx, self.max_z), // 2 NW
            Aabb::new(mx, mz, self.max_x, self.max_z), // 3 NE
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_is_edge_inclusive() {
        let b = Aabb::new(0.0, 0.0, 10.0, 10.0);
        assert!(b.contains(0.0, 0.0));
        assert!(b.contains(10.0, 10.0));
        assert!(b.contains(5.0, 5.0));
        assert!(!b.contains(-0.1, 5.0));
        assert!(!b.contains(5.0, 10.1));
    }

    #[test]
    fn quadrants_tile_the_parent() {
        let b = Aabb::new(0.0, 0.0, 8.0, 8.0);
        let kids = b.subdivide();
        for (x, z, want) in [
            (1.0, 1.0, 0usize),
            (7.0, 1.0, 1),
            (1.0, 7.0, 2),
            (7.0, 7.0, 3),
            (4.0, 4.0, 3), // mid-line -> east/north
        ] {
            assert_eq!(b.quadrant_of(x, z), want);
            assert!(kids[want].contains(x, z));
        }
    }

    #[test]
    fn circle_intersection_matches_geometry() {
        let b = Aabb::new(0.0, 0.0, 10.0, 10.0);
        assert!(b.intersects_circle(5.0, 5.0, 1.0));
        assert!(b.intersects_circle(12.0, 5.0, 2.0)); // reaches the east edge
        assert!(!b.intersects_circle(12.0, 5.0, 1.0)); // falls short
        assert!(!b.intersects_circle(20.0, 20.0, 5.0));
    }

    #[test]
    fn clamp_pulls_outside_points_to_the_edge() {
        let b = Aabb::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(b.clamp_point(-5.0, 20.0), (0.0, 10.0));
        assert_eq!(b.clamp_point(3.0, 4.0), (3.0, 4.0));
    }
}
