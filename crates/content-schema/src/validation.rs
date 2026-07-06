//! Cross-reference and version validation for an assembled [`Manifest`].

use std::collections::HashSet;

use omm_errors::{CoreError, CoreResult};

use crate::{quest::QuestObjectiveType, Manifest, CONTENT_API_VERSION};

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
    let known_factions: HashSet<&str> = manifest.factions.iter().map(|f| f.id.as_str()).collect();
    let known_abilities: HashSet<&str> = manifest.abilities.iter().map(|a| a.id.as_str()).collect();
    let known_items: HashSet<&str> = manifest.items.iter().map(|i| i.id.as_str()).collect();
    let known_quests: HashSet<&str> = manifest.quests.iter().map(|q| q.id.as_str()).collect();
    let known_zones: HashSet<&str> = manifest.zones.iter().map(|z| z.id.as_str()).collect();
    let known_spawn_tables: HashSet<&str> = manifest
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
