//! Spawn table definitions — what spawns where.

use serde::{Deserialize, Serialize};

/// A spawn table. Pure data — defines what spawns where.
// No `Eq`: holds `SpawnEntry` (`f32` positions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnTable {
    /// Stable machine id.
    pub id: String,
    /// Spawn entries.
    #[serde(default)]
    pub entries: Vec<SpawnEntry>,
    /// Respawn time in seconds.
    #[serde(default)]
    pub respawn_sec: u32,
}

/// A single spawn entry.
// No `Eq`: positions are `[f32; 3]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnEntry {
    /// Entity id to spawn.
    pub entity_id: String,
    /// Spawn weight (higher = more common).
    #[serde(default)]
    pub weight: u32,
    /// Max count alive at once.
    #[serde(default)]
    pub max_count: u8,
    /// Spawn positions (relative to zone origin).
    #[serde(default)]
    pub positions: Vec<[f32; 3]>,
}
