//! Camera-position world streaming — load the tiles around the camera and unload
//! the rest, all under a hard memory budget.
//!
//! Tiles are the quadtree leaf cells of the world model (glTF + heightmap chunks);
//! the client streams them by camera position so there are no loading screens
//! ([world-model](../../game-server/world-model/README.md)). This is the client
//! half of the same partition the server uses for area-of-interest — one mental
//! model, two consumers.
//!
//! Pure and deterministic: a camera position and a per-tile cost estimate in, a
//! [`StreamingDelta`] (what to load, unload, or skip) out. No GPU, no IO — the
//! policy is decided here and the caller performs the async loads. Nearer tiles
//! win the budget: when it is tight, a closer tile evicts a farther resident one.

use std::collections::{BTreeMap, BTreeSet};

use bevy_math::Vec2;

use crate::error::AssetError;

/// Integer tile coordinate in the streaming grid (a quadtree leaf cell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileCoord {
    /// Tile column (world X / `tile_size`, floored).
    pub x: i32,
    /// Tile row (world Y / `tile_size`, floored).
    pub y: i32,
}

/// What one [`StreamingGrid::update`] changed. Partitioned so a tile is in at most
/// one of `loaded`/`skipped`; `unloaded` is orthogonal (it is about what was
/// resident before). Every list is sorted for deterministic replay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamingDelta {
    /// Tiles newly made resident this update.
    pub loaded: Vec<TileCoord>,
    /// Tiles evicted this update (left the view, or lost the budget to a nearer tile).
    pub unloaded: Vec<TileCoord>,
    /// Tiles wanted this update but not loaded — the budget was full. Reported, not
    /// silently dropped, so budget pressure is visible.
    pub skipped: Vec<TileCoord>,
}

/// A square streaming grid with a hard residency budget.
#[derive(Debug, Clone)]
pub struct StreamingGrid {
    tile_size: f32,
    view_radius: f32,
    budget_bytes: u64,
    /// Currently resident tiles → their memory cost in bytes.
    loaded: BTreeMap<TileCoord, u64>,
}

impl StreamingGrid {
    /// Build a streaming grid.
    ///
    /// # Errors
    /// [`AssetError::InvalidStreamingConfig`] if `tile_size` is not finite-positive,
    /// `view_radius` is not finite and `>= 0`, or `budget_bytes` is zero.
    pub fn new(tile_size: f32, view_radius: f32, budget_bytes: u64) -> Result<Self, AssetError> {
        if !tile_size.is_finite() || tile_size <= 0.0 {
            return Err(AssetError::InvalidStreamingConfig { field: "tile_size" });
        }
        if !view_radius.is_finite() || view_radius < 0.0 {
            return Err(AssetError::InvalidStreamingConfig {
                field: "view_radius",
            });
        }
        if budget_bytes == 0 {
            return Err(AssetError::InvalidStreamingConfig { field: "budget" });
        }
        Ok(Self {
            tile_size,
            view_radius,
            budget_bytes,
            loaded: BTreeMap::new(),
        })
    }

    /// Reconcile residency to the camera at `camera` (world XY), using `cost_of` to
    /// size each tile. Loads tiles within the view radius closest-first, unloads
    /// tiles that left it, and — when the budget is tight — evicts a farther
    /// resident tile so a nearer one fits.
    ///
    /// # Errors
    /// [`AssetError::TileExceedsBudget`] if a single wanted tile costs more than the
    /// whole budget: it can never be resident, which is a content bug, not
    /// back-pressure.
    pub fn update(
        &mut self,
        camera: Vec2,
        cost_of: impl Fn(TileCoord) -> u64,
    ) -> Result<StreamingDelta, AssetError> {
        let before: BTreeSet<TileCoord> = self.loaded.keys().copied().collect();

        // Tiles whose cell touches the view disc, with squared distance to camera.
        let mut desired: Vec<(TileCoord, f32)> = self.tiles_in_view(camera);
        let desired_set: BTreeSet<TileCoord> = desired.iter().map(|(coord, _)| *coord).collect();
        let dist_of: BTreeMap<TileCoord, f32> = desired.iter().copied().collect();

        // Unload everything no longer wanted, then reconsider residency budget.
        self.loaded.retain(|coord, _| desired_set.contains(coord));
        let mut resident: u64 = self.loaded.values().copied().sum();

        // Load wanted-but-absent tiles, nearest first.
        desired.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        for (coord, sq_dist) in &desired {
            if self.loaded.contains_key(coord) {
                continue;
            }
            let cost = cost_of(*coord);
            if cost > self.budget_bytes {
                return Err(AssetError::TileExceedsBudget {
                    x: coord.x,
                    y: coord.y,
                    cost,
                    budget: self.budget_bytes,
                });
            }
            // Make room by evicting the farthest resident tile that is strictly
            // farther than this one — nearer tiles win a tight budget.
            while resident + cost > self.budget_bytes {
                let Some(victim) = self.farthest_resident_beyond(*sq_dist, &dist_of) else {
                    break;
                };
                if let Some(freed) = self.loaded.remove(&victim) {
                    resident -= freed;
                }
            }
            if resident + cost <= self.budget_bytes {
                self.loaded.insert(*coord, cost);
                resident += cost;
            }
        }

        let after: BTreeSet<TileCoord> = self.loaded.keys().copied().collect();
        Ok(StreamingDelta {
            loaded: after.difference(&before).copied().collect(),
            unloaded: before.difference(&after).copied().collect(),
            skipped: desired_set
                .iter()
                .filter(|coord| !after.contains(coord) && !before.contains(coord))
                .copied()
                .collect(),
        })
    }

    /// The resident tile that is strictly farther than `sq_dist`, or `None`.
    fn farthest_resident_beyond(
        &self,
        sq_dist: f32,
        dist_of: &BTreeMap<TileCoord, f32>,
    ) -> Option<TileCoord> {
        self.loaded
            .keys()
            .filter_map(|coord| dist_of.get(coord).map(|d| (*coord, *d)))
            .filter(|(_, d)| *d > sq_dist)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
            .map(|(coord, _)| coord)
    }

    /// Tiles whose cell intersects the view disc, each with its squared distance
    /// (nearest AABB point) to `camera`.
    fn tiles_in_view(&self, camera: Vec2) -> Vec<(TileCoord, f32)> {
        let r = self.view_radius;
        let r_sq = r * r;
        let lo = self.tile_at(camera - Vec2::splat(r));
        let hi = self.tile_at(camera + Vec2::splat(r));
        let mut tiles = Vec::new();
        for x in lo.x..=hi.x {
            for y in lo.y..=hi.y {
                let coord = TileCoord { x, y };
                let sq = self.tile_sq_distance(coord, camera);
                if sq <= r_sq {
                    tiles.push((coord, sq));
                }
            }
        }
        tiles
    }

    /// Squared distance from `camera` to the nearest point of `coord`'s cell.
    fn tile_sq_distance(&self, coord: TileCoord, camera: Vec2) -> f32 {
        let min = Vec2::new(coord.x as f32, coord.y as f32) * self.tile_size;
        let max = min + Vec2::splat(self.tile_size);
        let nearest = camera.clamp(min, max);
        camera.distance_squared(nearest)
    }

    /// The tile containing world position `pos`.
    #[must_use]
    pub fn tile_at(&self, pos: Vec2) -> TileCoord {
        TileCoord {
            x: (pos.x / self.tile_size).floor() as i32,
            y: (pos.y / self.tile_size).floor() as i32,
        }
    }

    /// Whether `coord` is currently resident.
    #[must_use]
    pub fn is_loaded(&self, coord: TileCoord) -> bool {
        self.loaded.contains_key(&coord)
    }

    /// Resident tiles, in coordinate order.
    pub fn loaded_tiles(&self) -> impl Iterator<Item = TileCoord> + '_ {
        self.loaded.keys().copied()
    }

    /// Total bytes currently resident.
    #[must_use]
    pub fn resident_bytes(&self) -> u64 {
        self.loaded.values().copied().sum()
    }

    /// The residency budget in bytes.
    #[must_use]
    pub fn budget_bytes(&self) -> u64 {
        self.budget_bytes
    }
}

#[cfg(test)]
mod tests;
