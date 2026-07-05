//! Typed schema for the **data-driven content layer**.
//!
//! Core is compiled; content is data. Factions, classes, races, abilities and
//! quests live as files under `content/` and are described by these types. An
//! operator adds a faction or ships a whole datapack — even a total-conversion
//! that reshapes maps and rules — by editing data and bumping the manifest, with
//! **no `cargo build`**. That is the extensibility contract
//! (docs/architecture/05-ecs-and-scripting.md): our core stays strong and
//! compiled, operators get the full thing to customize.
//!
//! Everything here is **pure**: parse + validate, no I/O. Callers read the bytes
//! (from disk, an archive, object storage) and hand them to [`load_manifest`].

use omm_errors::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

/// The core content-API version this build understands. A datapack declares the
/// API it targets; we refuse to load one built against a different major line.
pub const CONTENT_API_VERSION: u32 = 1;

/// A datapack manifest — the root of a content bundle (`content/manifest.json`).
// No `Eq`: zones/dungeons/abilities carry `f32` positions, and `f32` is not `Eq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Human-facing datapack id, e.g. `"open-mmorpg.base"`.
    pub id: String,
    /// Datapack semantic version string (operator-owned).
    pub version: String,
    /// The core content-API major version this pack targets.
    pub api_version: u32,
    /// Factions shipped by this pack.
    #[serde(default)]
    pub factions: Vec<Faction>,
    /// Races shipped by this pack.
    #[serde(default)]
    pub races: Vec<RaceDef>,
    /// Classes shipped by this pack.
    #[serde(default)]
    pub classes: Vec<ClassDef>,
    /// Abilities shipped by this pack.
    #[serde(default)]
    pub abilities: Vec<AbilityDef>,
    /// Items shipped by this pack.
    #[serde(default)]
    pub items: Vec<ItemDef>,
    /// Quests shipped by this pack.
    #[serde(default)]
    pub quests: Vec<QuestDef>,
    /// Zones shipped by this pack.
    #[serde(default)]
    pub zones: Vec<ZoneDef>,
    /// Spawn tables shipped by this pack.
    #[serde(default)]
    pub spawn_tables: Vec<SpawnTable>,
    /// Dungeons shipped by this pack.
    #[serde(default)]
    pub dungeons: Vec<DungeonDef>,
    /// Economy definitions.
    #[serde(default)]
    pub economy: EconomyData,
    /// Asset manifest reference path (relative to content/).
    #[serde(default)]
    pub asset_manifest_ref: Option<String>,
}

/// Economy data definitions.
// No `Eq`: holds `AuctionHouseDef`, whose fee fields are `f32`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EconomyData {
    /// Auction houses.
    #[serde(default)]
    pub auction_houses: Vec<AuctionHouseDef>,
    /// Trading rules.
    #[serde(default)]
    pub trading_rules: Vec<TradingRuleDef>,
    /// Starting gold for new characters (in copper).
    #[serde(default)]
    pub starting_gold_copper: u32,
}

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

/// Stat modifiers for race/class definitions.
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

/// Auction house definition.
// No `Eq`: fee/min-bid/deposit percentages are `f32`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionHouseDef {
    /// Stable machine id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Hosting zone id (where the AH physically exists).
    pub zone_id: String,
    /// AH position in zone.
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    /// Fee percentage (0.05 = 5%).
    #[serde(default)]
    pub fee_percentage: f32,
    /// Minimum listing increment (percentage of current bid).
    #[serde(default)]
    pub min_bid_increment: f32,
    /// Maximum active listings per account.
    #[serde(default)]
    pub max_listings_per_account: u16,
    /// Listing duration in hours.
    #[serde(default)]
    pub listing_duration_hours: u32,
    /// Deposit cost percentage (0.05 = 5% of item value).
    #[serde(default)]
    pub deposit_percentage: f32,
}

/// Trading rules for item types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradingRuleDef {
    /// Item pattern this applies to (e.g., "soulbound", "quest_*").
    pub item_pattern: String,
    /// Whether this item can be traded.
    pub tradable: bool,
    /// Whether this item can be auctioned.
    pub auctionable: bool,
    /// Whether this item can be mailed.
    pub mailing_allowed: bool,
}

/// Parse and validate a manifest from raw JSON bytes.
///
/// # Errors
/// - [`CoreError::BadRequest`] if the JSON is malformed or fails validation
///   (unknown API version, empty ids, or a faction hostile to an unknown id).
pub fn load_manifest(bytes: &[u8]) -> CoreResult<Manifest> {
    let manifest: Manifest = serde_json::from_slice(bytes)
        .map_err(|e| CoreError::BadRequest(format!("manifest json: {e}")))?;
    validate(&manifest)?;
    Ok(manifest)
}

/// Validate cross-references and version compatibility within a manifest.
///
/// # Errors
/// [`CoreError::BadRequest`] describing the first violation found.
pub fn validate(manifest: &Manifest) -> CoreResult<()> {
    if manifest.api_version != CONTENT_API_VERSION {
        return Err(CoreError::BadRequest(format!(
            "datapack targets content API v{}, core provides v{CONTENT_API_VERSION}",
            manifest.api_version
        )));
    }
    if manifest.id.trim().is_empty() {
        return Err(CoreError::BadRequest("manifest id is empty".into()));
    }

    // Collect the IDs validation actually cross-references. Races, classes and
    // dungeons aren't referenced by id from other content today, so their id
    // sets aren't needed here — adding such checks would re-introduce them.
    let known_factions: std::collections::HashSet<&str> =
        manifest.factions.iter().map(|f| f.id.as_str()).collect();
    let known_abilities: std::collections::HashSet<&str> =
        manifest.abilities.iter().map(|a| a.id.as_str()).collect();
    let known_items: std::collections::HashSet<&str> =
        manifest.items.iter().map(|i| i.id.as_str()).collect();
    let known_quests: std::collections::HashSet<&str> =
        manifest.quests.iter().map(|q| q.id.as_str()).collect();
    let known_zones: std::collections::HashSet<&str> =
        manifest.zones.iter().map(|z| z.id.as_str()).collect();
    let known_spawn_tables: std::collections::HashSet<&str> = manifest
        .spawn_tables
        .iter()
        .map(|s| s.id.as_str())
        .collect();

    // Validate factions
    for faction in &manifest.factions {
        if faction.id.trim().is_empty() {
            return Err(CoreError::BadRequest("faction id is empty".into()));
        }
        for target in &faction.hostile_to {
            if !known_factions.contains(target.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "faction '{}' is hostile to unknown faction '{target}'",
                    faction.id
                )));
            }
        }
        if let Some(capital) = &faction.capital {
            if !known_zones.contains(capital.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "faction '{}' capital '{capital}' references unknown zone",
                    faction.id
                )));
            }
        }
    }

    // Validate races
    for race in &manifest.races {
        if race.id.trim().is_empty() {
            return Err(CoreError::BadRequest("race id is empty".into()));
        }
        if !known_factions.contains(race.faction_id.as_str()) {
            return Err(CoreError::BadRequest(format!(
                "race '{}' faction_id '{}' references unknown faction",
                race.id, race.faction_id
            )));
        }
        for trait_id in &race.traits {
            if !known_abilities.contains(trait_id.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "race '{}' trait '{trait_id}' references unknown ability",
                    race.id
                )));
            }
        }
    }

    // Validate classes
    for class in &manifest.classes {
        if class.id.trim().is_empty() {
            return Err(CoreError::BadRequest("class id is empty".into()));
        }
        for ability_id in &class.abilities {
            if !known_abilities.contains(ability_id.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "class '{}' ability '{ability_id}' references unknown ability",
                    class.id
                )));
            }
        }
    }

    // Validate quests
    for quest in &manifest.quests {
        if quest.id.trim().is_empty() {
            return Err(CoreError::BadRequest("quest id is empty".into()));
        }
        for prereq in &quest.prerequisites {
            if !known_quests.contains(prereq.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "quest '{}' prereq '{prereq}' references unknown quest",
                    quest.id
                )));
            }
        }
        if let Some(next_id) = &quest.next_quest_id {
            if !known_quests.contains(next_id.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "quest '{}' next_quest_id '{next_id}' references unknown quest",
                    quest.id
                )));
            }
        }
        for objective in &quest.objectives {
            // Validate objective target_id based on type
            match objective.objective_type {
                QuestObjectiveType::Kill | QuestObjectiveType::Speak => {
                    // Entity IDs - we don't validate these as they're runtime entities
                }
                QuestObjectiveType::Gather | QuestObjectiveType::Deliver => {
                    if !known_items.contains(objective.target_id.as_str()) {
                        return Err(CoreError::BadRequest(format!(
                            "quest '{}' objective target '{}' references unknown item",
                            quest.id, objective.target_id
                        )));
                    }
                }
                QuestObjectiveType::Explore => {
                    // Zone IDs
                    if !known_zones.contains(objective.target_id.as_str()) {
                        return Err(CoreError::BadRequest(format!(
                            "quest '{}' objective target '{}' references unknown zone",
                            quest.id, objective.target_id
                        )));
                    }
                }
            }
        }
        for reward_item in &quest.rewards.items {
            if !known_items.contains(reward_item.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "quest '{}' reward item '{reward_item}' references unknown item",
                    quest.id
                )));
            }
        }
        for choice_item in &quest.rewards.choice_items {
            if !known_items.contains(choice_item.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "quest '{}' choice item '{choice_item}' references unknown item",
                    quest.id
                )));
            }
        }
    }

    // Validate zones
    for zone in &manifest.zones {
        if zone.id.trim().is_empty() {
            return Err(CoreError::BadRequest("zone id is empty".into()));
        }
        for spawn_table_id in &zone.spawn_tables {
            if !known_spawn_tables.contains(spawn_table_id.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "zone '{}' spawn_table '{spawn_table_id}' references unknown spawn table",
                    zone.id
                )));
            }
        }
        if let Some(parent_id) = &zone.parent_zone_id {
            if !known_zones.contains(parent_id.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "zone '{}' parent_zone_id '{parent_id}' references unknown zone",
                    zone.id
                )));
            }
        }
    }

    // Validate dungeons
    for dungeon in &manifest.dungeons {
        if dungeon.id.trim().is_empty() {
            return Err(CoreError::BadRequest("dungeon id is empty".into()));
        }
        if let Some(zone_id) = &dungeon.entrance_zone_id {
            if !known_zones.contains(zone_id.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "dungeon '{}' entrance_zone_id '{zone_id}' references unknown zone",
                    dungeon.id
                )));
            }
        }
        for loot_table in &dungeon.loot_tables {
            // Loot tables are item IDs for now
            if !known_items.contains(loot_table.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "dungeon '{}' loot_table '{loot_table}' references unknown item",
                    dungeon.id
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "id": "open-mmorpg.base",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [
            { "id": "dawnward", "name": "The Dawnward Pact", "hostile_to": ["nightfen"] },
            { "id": "nightfen", "name": "Nightfen Covenant", "hostile_to": ["dawnward"] }
        ]
    }"#;

    #[test]
    fn loads_and_validates_the_base_pack() {
        let m = load_manifest(SAMPLE.as_bytes()).unwrap();
        assert_eq!(m.id, "open-mmorpg.base");
        assert_eq!(m.factions.len(), 2);
    }

    #[test]
    fn rejects_wrong_api_version() {
        let bad = SAMPLE.replace("\"api_version\": 1", "\"api_version\": 2");
        let err = load_manifest(bad.as_bytes()).unwrap_err();
        assert_eq!(err.code(), omm_errors::ClientCode::BadRequest);
    }

    #[test]
    fn rejects_dangling_hostile_reference() {
        let bad = SAMPLE.replace("[\"nightfen\"]", "[\"ghosts\"]");
        assert!(load_manifest(bad.as_bytes()).is_err());
    }
}
