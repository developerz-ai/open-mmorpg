//! Ability (skill, spell, attack) definitions. Effects are data-driven.

use serde::{Deserialize, Serialize};

/// An ability (skill, spell, attack). Pure data — effects are data-driven.
// No `Eq`: cooldown/cast/range are `f32`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbilityDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Ability icon path.
    #[serde(default)]
    pub icon: Option<String>,
    /// Max rank (1 = single-tier ability).
    #[serde(default = "default_max_rank")]
    pub max_rank: u8,
    /// Cooldown in seconds.
    #[serde(default)]
    pub cooldown_sec: f32,
    /// Resource cost.
    #[serde(default)]
    pub resource_cost: u16,
    /// Cast time in seconds (0 = instant).
    #[serde(default)]
    pub cast_time_sec: f32,
    /// Range in yards (0 = melee).
    #[serde(default)]
    pub range_yards: f32,
    /// Effects this ability applies.
    #[serde(default)]
    pub effects: Vec<AbilityEffect>,
}

fn default_max_rank() -> u8 {
    1
}

/// A single effect an ability can apply.
// No `Eq`: magnitude/scaling are `f32`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbilityEffect {
    /// Effect type.
    pub effect: AbilityEffectType,
    /// Base magnitude (damage, healing, duration, etc.).
    pub magnitude: f32,
    /// Scaling coefficient (e.g., per-attack-power or per-spell-power).
    #[serde(default)]
    pub scaling: f32,
    /// Target type (self, enemy, ally, point).
    #[serde(default)]
    pub target: String,
}

/// The type of effect an ability applies.
// Variants serialize by their PascalCase names (`"Damage"`, `"Heal"` …), matching
// the content authoring guide (docs/plans/.../060-content-assets/authoring-guide.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityEffectType {
    Damage,
    Heal,
    ApplyAura,
    Summon,
    Teleport,
    Buff,
    Debuff,
    Dot,
    Hot,
}
