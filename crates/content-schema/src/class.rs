//! Playable class definitions.

use serde::{Deserialize, Serialize};

use crate::stats::StatModifiers;

/// A playable class. Pure data — defines abilities and role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description/flavor text.
    #[serde(default)]
    pub description: String,
    /// Primary role (tank, healer, dps, support).
    #[serde(default)]
    pub role: String,
    /// Core ability ids granted by this class.
    #[serde(default)]
    pub abilities: Vec<String>,
    /// Stat bonuses per level.
    #[serde(default)]
    pub stat_growth: StatModifiers,
    /// Hit points per level.
    #[serde(default)]
    pub hp_per_level: u16,
    /// Resource (mana/energy/rage) per level.
    #[serde(default)]
    pub resource_per_level: u16,
}
