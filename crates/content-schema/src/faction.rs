//! Faction definitions.

use serde::{Deserialize, Serialize};

/// A faction players can belong to. Pure data — no behavior compiled in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faction {
    /// Stable machine id, referenced by other content.
    pub id: String,
    /// Display name (shown via the web/client i18n layer).
    pub name: String,
    /// Faction ethos/description.
    #[serde(default)]
    pub description: String,
    /// Faction colors (primary hex, secondary hex).
    #[serde(default)]
    pub colors: FactionColors,
    /// Capital zone id.
    #[serde(default)]
    pub capital: Option<String>,
    /// Faction ids this one is hostile toward.
    #[serde(default)]
    pub hostile_to: Vec<String>,
}

/// Faction color scheme for UI/textures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionColors {
    /// Primary color hex.
    #[serde(default)]
    pub primary: String,
    /// Secondary color hex.
    #[serde(default)]
    pub secondary: String,
}

impl Default for FactionColors {
    fn default() -> Self {
        Self {
            primary: "#ffffff".to_string(),
            secondary: "#808080".to_string(),
        }
    }
}
