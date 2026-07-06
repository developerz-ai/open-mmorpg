//! Zone definitions — level range, safe locations, spawns.

use serde::{Deserialize, Serialize};

/// A zone. Pure data — defines level range, safe locations, and spawns.
// No `Eq`: holds `SafeLocation` (`f32` positions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Minimum level.
    #[serde(default)]
    pub min_level: u8,
    /// Maximum level.
    #[serde(default)]
    pub max_level: u8,
    /// Safe locations (respawn points).
    #[serde(default)]
    pub safe_locations: Vec<SafeLocation>,
    /// Factions that control this zone (empty = contested).
    #[serde(default)]
    pub controlling_factions: Vec<String>,
    /// Associated spawn table ids.
    #[serde(default)]
    pub spawn_tables: Vec<String>,
    /// Parent zone id (if sub-zone).
    #[serde(default)]
    pub parent_zone_id: Option<String>,
    /// Navmesh resource path.
    #[serde(default)]
    pub navmesh: Option<String>,
}

/// A safe respawn location.
// No `Eq`: position/yaw are `f32`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafeLocation {
    /// Location id.
    pub id: String,
    /// Position (x, y, z).
    pub position: [f32; 3],
    /// Rotation yaw.
    #[serde(default)]
    pub yaw: f32,
}
