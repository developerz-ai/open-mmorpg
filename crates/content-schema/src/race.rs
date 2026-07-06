//! Playable race definitions.

use serde::{Deserialize, Serialize};

use crate::stats::StatModifiers;

/// A playable race. Pure data — defines starting stats and racial traits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description/flavor text.
    #[serde(default)]
    pub description: String,
    /// Starting faction id.
    pub faction_id: String,
    /// Racial passive trait ids.
    #[serde(default)]
    pub traits: Vec<String>,
    /// Starting stat modifiers (str, dex, con, int, wis, cha).
    #[serde(default)]
    pub stats: StatModifiers,
}
