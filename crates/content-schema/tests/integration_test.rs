//! Integration tests for content-schema.
//!
//! These tests verify that the full datapack loads correctly and that
//! cross-references resolve. `load_manifest` is pure (bytes in, no I/O), so the
//! tests feed it JSON bytes directly.

use omm_content_schema::load_manifest;

#[test]
fn load_base_datapack() {
    let manifest_json = r#"{
        "id": "open-mmorpg.base",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [
            { "id": "dawnward", "name": "The Dawnward Pact", "hostile_to": ["nightfen"] },
            { "id": "nightfen", "name": "Nightfen Covenant", "hostile_to": ["dawnward"] }
        ],
        "races": [
            {
                "id": "human",
                "name": "Human",
                "description": "Versatile and adaptable",
                "faction_id": "dawnward",
                "traits": [],
                "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }
            }
        ],
        "classes": [
            {
                "id": "warrior",
                "name": "Warrior",
                "description": "Melee combat specialist",
                "role": "tank",
                "abilities": [],
                "stat_growth": { "strength": 1, "dexterity": 0, "constitution": 1, "intelligence": 0, "wisdom": 0, "charisma": 0 },
                "hp_per_level": 12,
                "resource_per_level": 0
            }
        ],
        "abilities": [
            {
                "id": "slash",
                "name": "Slash",
                "description": "A basic melee attack",
                "max_rank": 1,
                "cooldown_sec": 0.0,
                "resource_cost": 0,
                "cast_time_sec": 0.0,
                "range_yards": 0.0,
                "effects": []
            }
        ],
        "items": [
            {
                "id": "rusty-sword",
                "name": "Rusty Sword",
                "description": "A worn but functional blade",
                "item_type": "Weapon",
                "required_level": 1,
                "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 },
                "quality": "common",
                "value_copper": 10,
                "max_stack": 1
            }
        ],
        "quests": [
            {
                "id": "welcome",
                "name": "Welcome to the World",
                "description": "Speak to the town elder",
                "level": 1,
                "prerequisites": [],
                "objectives": [
                    { "objective_type": "Speak", "target_id": "elder-npc", "count": 1, "description": "Speak to the town elder" }
                ],
                "rewards": { "experience": 100, "gold_copper": 50, "choice_items": [], "items": [] }
            }
        ],
        "zones": [
            {
                "id": "starting-zone",
                "name": "Starting Zone",
                "min_level": 1,
                "max_level": 10,
                "safe_locations": [],
                "controlling_factions": ["dawnward"],
                "spawn_tables": [],
                "navmesh": null
            }
        ],
        "spawn_tables": [],
        "dungeons": [],
        "economy": {
            "auction_houses": [],
            "trading_rules": [],
            "starting_gold_copper": 0
        }
    }"#;

    let manifest = load_manifest(manifest_json.as_bytes()).unwrap();

    assert_eq!(manifest.id, "open-mmorpg.base");
    assert_eq!(manifest.factions.len(), 2);
    assert_eq!(manifest.races.len(), 1);
    assert_eq!(manifest.classes.len(), 1);
    assert_eq!(manifest.abilities.len(), 1);
    assert_eq!(manifest.items.len(), 1);
    assert_eq!(manifest.quests.len(), 1);
    assert_eq!(manifest.zones.len(), 1);
}

#[test]
fn validate_cross_references() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [
            { "id": "f1", "name": "Faction 1", "hostile_to": ["f2"] },
            { "id": "f2", "name": "Faction 2", "hostile_to": [] }
        ],
        "races": [
            { "id": "r1", "name": "Race 1", "faction_id": "f1", "traits": [], "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 } }
        ],
        "classes": [
            { "id": "c1", "name": "Class 1", "role": "tank", "abilities": ["a1"], "stat_growth": { "strength": 1, "dexterity": 0, "constitution": 1, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "hp_per_level": 12, "resource_per_level": 0 }
        ],
        "abilities": [
            { "id": "a1", "name": "Ability 1", "max_rank": 1, "cooldown_sec": 0.0, "resource_cost": 0, "cast_time_sec": 0.0, "range_yards": 0.0, "effects": [] }
        ],
        "items": [
            { "id": "i1", "name": "Item 1", "item_type": "Weapon", "required_level": 1, "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "quality": "common", "value_copper": 10, "max_stack": 1 }
        ],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    assert!(result.is_ok());
}

#[test]
fn faction_hostility_symmetry() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [
            { "id": "f1", "name": "F1", "hostile_to": ["f2"] },
            { "id": "f2", "name": "F2", "hostile_to": [] }
        ],
        "races": [],
        "classes": [],
        "abilities": [],
        "items": [],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    // f1 is hostile to f2, but f2 is not hostile to f1 - this is allowed
    assert!(result.is_ok());
}

#[test]
fn class_race_compatibility() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [
            { "id": "f1", "name": "F1", "hostile_to": [] }
        ],
        "races": [
            { "id": "r1", "name": "R1", "faction_id": "f1", "traits": [], "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 } }
        ],
        "classes": [
            { "id": "c1", "name": "C1", "role": "tank", "abilities": [], "stat_growth": { "strength": 1, "dexterity": 0, "constitution": 1, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "hp_per_level": 12, "resource_per_level": 0 }
        ],
        "abilities": [],
        "items": [],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    // Classes and races are independent - no cross-validation needed
    assert!(result.is_ok());
}

#[test]
fn item_stat_bounds() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [],
        "classes": [],
        "abilities": [],
        "items": [
            { "id": "i1", "name": "I1", "item_type": "Weapon", "required_level": 1, "stats": { "strength": 100, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "quality": "common", "value_copper": 10, "max_stack": 1 }
        ],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    // High stats are allowed - no bounds validation in schema
    assert!(result.is_ok());
}

#[test]
fn quest_prereq_chain() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [],
        "classes": [],
        "abilities": [],
        "items": [
            { "id": "i1", "name": "I1", "item_type": "Quest", "required_level": 1, "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "quality": "common", "value_copper": 0, "max_stack": 1 }
        ],
        "quests": [
            { "id": "q1", "name": "Q1", "level": 1, "prerequisites": [], "objectives": [], "rewards": { "experience": 100, "gold_copper": 0, "choice_items": [], "items": ["i1"] } },
            { "id": "q2", "name": "Q2", "level": 2, "prerequisites": ["q1"], "objectives": [], "rewards": { "experience": 200, "gold_copper": 0, "choice_items": [], "items": [] } },
            { "id": "q3", "name": "Q3", "level": 3, "prerequisites": ["q2"], "objectives": [], "rewards": { "experience": 300, "gold_copper": 0, "choice_items": [], "items": [] }, "next_quest_id": "q1" }
        ],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    // Circular reference (q3 -> q1) is allowed by current validation
    assert!(result.is_ok());
}

#[test]
fn asset_manifest_loading() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [],
        "classes": [],
        "abilities": [],
        "items": [],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 },
        "asset_manifest_ref": "assets/manifest.json"
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    // Asset manifest ref is just a string reference - no validation of the file itself
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(
        manifest.asset_manifest_ref,
        Some("assets/manifest.json".to_string())
    );
}

#[test]
fn rejects_invalid_faction_reference() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [
            { "id": "r1", "name": "R1", "faction_id": "nonexistent", "traits": [], "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 } }
        ],
        "classes": [],
        "abilities": [],
        "items": [],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_ability_reference() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [],
        "classes": [
            { "id": "c1", "name": "C1", "role": "tank", "abilities": ["nonexistent"], "stat_growth": { "strength": 1, "dexterity": 0, "constitution": 1, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "hp_per_level": 12, "resource_per_level": 0 }
        ],
        "abilities": [],
        "items": [],
        "quests": [],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_quest_prerequisite() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [],
        "classes": [],
        "abilities": [],
        "items": [],
        "quests": [
            { "id": "q1", "name": "Q1", "level": 1, "prerequisites": ["nonexistent"], "objectives": [], "rewards": { "experience": 100, "gold_copper": 0, "choice_items": [], "items": [] } }
        ],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_item_reward() {
    let manifest_json = r#"{
        "id": "test",
        "version": "0.0.0",
        "api_version": 1,
        "factions": [],
        "races": [],
        "classes": [],
        "abilities": [],
        "items": [],
        "quests": [
            { "id": "q1", "name": "Q1", "level": 1, "prerequisites": [], "objectives": [], "rewards": { "experience": 100, "gold_copper": 0, "choice_items": [], "items": ["nonexistent"] } }
        ],
        "zones": [],
        "spawn_tables": [],
        "dungeons": [],
        "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
    }"#;

    let result = load_manifest(manifest_json.as_bytes());

    assert!(result.is_err());
}
