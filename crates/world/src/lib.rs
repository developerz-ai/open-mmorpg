//! `omm-world` — the deterministic spatial index for a zone.
//!
//! One quadtree over the world **ground plane** (`x`/`z`; `y` is up) serves the
//! three jobs the world-model spec assigns it: **interest management** (who sees
//! whom), **streaming** (what the client loads), and **shard boundaries** (who a
//! shard owns). There is deliberately only one index — no second, forked
//! structure to drift out of sync.
//!
//! The crate is **pure**: no I/O, no wall-clock, no RNG. Given the same ordered
//! operations it reaches the same state and returns the same, id-sorted query
//! results — the property that lets the sim replay and re-simulate for
//! anti-cheat.
//!
//! # Example
//! ```
//! use omm_world::{Aabb, EntityId, Quadtree};
//! use omm_protocol::Vec3;
//!
//! let mut tree = Quadtree::new(Aabb::new(0.0, 0.0, 256.0, 256.0), 6, 8);
//! tree.insert(EntityId(1), Vec3 { x: 10.0, y: 0.0, z: 10.0 });
//! tree.insert(EntityId(2), Vec3 { x: 12.0, y: 4.0, z: 11.0 });
//! tree.insert(EntityId(3), Vec3 { x: 200.0, y: 0.0, z: 200.0 });
//!
//! // Who does a viewer at (10, 10) see within a 5-unit radius?
//! let seen = tree.interest_set(Vec3 { x: 10.0, y: 0.0, z: 10.0 }, 5.0);
//! assert_eq!(seen, vec![EntityId(1), EntityId(2)]);
//! ```

mod aoi;
mod geometry;
mod node;
mod tree;

pub use geometry::Aabb;
pub use tree::Quadtree;

/// A world entity handle, as seen by the spatial index.
///
/// A lightweight local newtype for now: the index only needs a stable, ordered
/// key. Query and interest results are sorted by this id for deterministic,
/// replay-stable output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u64);
