//! Quest definitions — objectives and rewards.

use serde::{Deserialize, Serialize};

/// A quest. Pure data — defines objectives and rewards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description/text.
    #[serde(default)]
    pub description: String,
    /// Quest level.
    #[serde(default)]
    pub level: u8,
    /// Required quests (ids).
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Objectives to complete.
    #[serde(default)]
    pub objectives: Vec<QuestObjective>,
    /// Rewards.
    #[serde(default)]
    pub rewards: QuestRewards,
    /// Quest-giving NPC id.
    #[serde(default)]
    pub giver_id: Option<String>,
    /// Quest-turn-in NPC id.
    #[serde(default)]
    pub turn_in_id: Option<String>,
    /// Next quest id in chain.
    #[serde(default)]
    pub next_quest_id: Option<String>,
}

/// A single quest objective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestObjective {
    /// Objective type.
    pub objective_type: QuestObjectiveType,
    /// Target id (entity id, item id, etc.).
    pub target_id: String,
    /// Required count.
    pub count: u8,
    /// Description.
    #[serde(default)]
    pub description: String,
}

/// Objective type.
// Variants serialize by their PascalCase names (`"Kill"`, `"Speak"` …), matching
// the content authoring guide (docs/plans/.../060-content-assets/authoring-guide.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestObjectiveType {
    Kill,
    Gather,
    Speak,
    Deliver,
    Explore,
}

/// Quest rewards.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct QuestRewards {
    /// Experience reward.
    #[serde(default)]
    pub experience: u32,
    /// Gold reward in copper.
    #[serde(default)]
    pub gold_copper: u32,
    /// Item choice (pick one).
    #[serde(default)]
    pub choice_items: Vec<String>,
    /// Items granted to all.
    #[serde(default)]
    pub items: Vec<String>,
}
