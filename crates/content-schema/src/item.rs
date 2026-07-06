//! Item template definitions (templates, not instances).

use serde::{Deserialize, Serialize};

use crate::stats::StatModifiers;

/// An item template (not an instance). Pure data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Item type.
    pub item_type: ItemType,
    /// Item slot (if equippable).
    #[serde(default)]
    pub slot: Option<String>,
    /// Required level.
    #[serde(default)]
    pub required_level: u8,
    /// Stats granted (if any).
    #[serde(default)]
    pub stats: StatModifiers,
    /// Item quality (common, uncommon, rare, epic, legendary).
    #[serde(default)]
    pub quality: String,
    /// Value in copper (0 = soulbound).
    #[serde(default)]
    pub value_copper: u32,
    /// Unique icon path.
    #[serde(default)]
    pub icon: Option<String>,
    /// Mesh path (if equippable).
    #[serde(default)]
    pub mesh: Option<String>,
    /// Maximum stack size.
    #[serde(default = "default_stack_size")]
    pub max_stack: u16,
}

fn default_stack_size() -> u16 {
    1
}

/// Item category.
// Variants serialize by their PascalCase names (`"Weapon"`, `"CraftingMaterial"` …),
// matching the content authoring guide (docs/plans/.../060-content-assets/authoring-guide.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Trinket,
    Consumable,
    Quest,
    CraftingMaterial,
    Misc,
}
