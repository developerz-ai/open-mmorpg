//! Dungeon definitions — instanced content.

use serde::{Deserialize, Serialize};

/// A dungeon. Pure data — instanced content definition.
// No `Eq`: entrance_position is `Option<[f32; 3]>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DungeonDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Minimum level.
    #[serde(default)]
    pub min_level: u8,
    /// Recommended level.
    #[serde(default)]
    pub recommended_level: u8,
    /// Maximum group size.
    #[serde(default)]
    pub max_players: u8,
    /// Boss encounter ids (in order).
    #[serde(default)]
    pub boss_ids: Vec<String>,
    /// Trash spawn table ids.
    #[serde(default)]
    pub trash_spawn_tables: Vec<String>,
    /// Loot table ids per boss.
    #[serde(default)]
    pub loot_tables: Vec<String>,
    /// Instance time limit in minutes (0 = unlimited).
    #[serde(default)]
    pub time_limit_minutes: u32,
    /// Lockout time in hours (0 = no lockout).
    #[serde(default)]
    pub lockout_hours: u32,
    /// Entrance zone id.
    #[serde(default)]
    pub entrance_zone_id: Option<String>,
    /// Entrance position.
    #[serde(default)]
    pub entrance_position: Option<[f32; 3]>,
}
