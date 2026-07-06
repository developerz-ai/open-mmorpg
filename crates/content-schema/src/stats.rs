//! Shared stat-modifier block used by races, classes, and items.

use serde::{Deserialize, Serialize};

/// Stat modifiers for race/class/item definitions (str, dex, con, int, wis, cha).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StatModifiers {
    #[serde(default)]
    pub strength: i8,
    #[serde(default)]
    pub dexterity: i8,
    #[serde(default)]
    pub constitution: i8,
    #[serde(default)]
    pub intelligence: i8,
    #[serde(default)]
    pub wisdom: i8,
    #[serde(default)]
    pub charisma: i8,
}
