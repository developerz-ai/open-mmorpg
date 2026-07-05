// Criterion benchmarks for content-schema loading and validation.
// Benches aren't `#[test]`s, so clippy's `allow-unwrap-in-tests` doesn't cover
// them — but a load failure in a bench SHOULD panic loudly, exactly like a test.
#![allow(clippy::unwrap_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use omm_content_schema::{load_manifest, validate};

/// A minimal manifest for baseline testing. Uses `r##"…"`## so the hex colors
/// (`"#4a90e2"`) inside the realistic manifest can't prematurely close the raw
/// string — the `"#` sequence is exactly the `r#"…"`# closer.
const MINIMAL_MANIFEST: &str = r##"{
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
    "economy": { "auction_houses": [], "trading_rules": [], "starting_gold_copper": 0 }
}"##;

/// A manifest with a realistic amount of content.
const REALISTIC_MANIFEST: &str = r##"{
    "id": "open-mmorpg.base",
    "version": "0.0.0",
    "api_version": 1,
    "factions": [
        { "id": "dawnward", "name": "The Dawnward Pact", "description": "Order and progress", "colors": { "primary": "#4a90e2", "secondary": "#2c5f8d" }, "capital": "ember-city", "hostile_to": ["nightfen"] },
        { "id": "nightfen", "name": "Nightfen Covenant", "description": "Shadow and secrets", "colors": { "primary": "#8b5a9e", "secondary": "#5c3a6e" }, "capital": "shadow-haven", "hostile_to": ["dawnward"] }
    ],
    "races": [
        { "id": "human", "name": "Human", "description": "Versatile and adaptable", "faction_id": "dawnward", "traits": [], "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 } },
        { "id": "elf", "name": "Elf", "description": "Graceful and intelligent", "faction_id": "dawnward", "traits": [], "stats": { "strength": -1, "dexterity": 2, "constitution": -1, "intelligence": 1, "wisdom": 1, "charisma": 0 } }
    ],
    "classes": [
        { "id": "warrior", "name": "Warrior", "description": "Melee combat specialist", "role": "tank", "abilities": ["slash", "taunt"], "stat_growth": { "strength": 1, "dexterity": 0, "constitution": 1, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "hp_per_level": 12, "resource_per_level": 0 },
        { "id": "mage", "name": "Mage", "description": "Arcane spellcaster", "role": "dps", "abilities": ["fireball", "frostbolt"], "stat_growth": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 2, "wisdom": 1, "charisma": 0 }, "hp_per_level": 8, "resource_per_level": 10 }
    ],
    "abilities": [
        { "id": "slash", "name": "Slash", "description": "A basic melee attack", "max_rank": 1, "cooldown_sec": 0.0, "resource_cost": 0, "cast_time_sec": 0.0, "range_yards": 0.0, "effects": [{ "effect": "Damage", "magnitude": 10.0, "scaling": 1.0, "target": "enemy" }] },
        { "id": "taunt", "name": "Taunt", "description": "Force enemy to attack you", "max_rank": 1, "cooldown_sec": 3.0, "resource_cost": 0, "cast_time_sec": 0.0, "range_yards": 10.0, "effects": [{ "effect": "Buff", "magnitude": 0.0, "scaling": 0.0, "target": "self" }] },
        { "id": "fireball", "name": "Fireball", "description": "Hurl a ball of fire", "max_rank": 1, "cooldown_sec": 2.5, "resource_cost": 50, "cast_time_sec": 2.0, "range_yards": 30.0, "effects": [{ "effect": "Damage", "magnitude": 25.0, "scaling": 0.8, "target": "enemy" }] },
        { "id": "frostbolt", "name": "Frostbolt", "description": "Launch a bolt of frost", "max_rank": 1, "cooldown_sec": 2.0, "resource_cost": 40, "cast_time_sec": 1.5, "range_yards": 30.0, "effects": [{ "effect": "Damage", "magnitude": 20.0, "scaling": 0.7, "target": "enemy" }] }
    ],
    "items": [
        { "id": "rusty-sword", "name": "Rusty Sword", "description": "A worn but functional blade", "item_type": "Weapon", "slot": "mainhand", "required_level": 1, "stats": { "strength": 2, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "quality": "common", "value_copper": 10, "max_stack": 1 },
        { "id": "iron-sword", "name": "Iron Sword", "description": "A reliable iron blade", "item_type": "Weapon", "slot": "mainhand", "required_level": 5, "stats": { "strength": 5, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "quality": "uncommon", "value_copper": 100, "max_stack": 1 },
        { "id": "health-potion", "name": "Health Potion", "description": "Restores health", "item_type": "Consumable", "required_level": 1, "stats": { "strength": 0, "dexterity": 0, "constitution": 0, "intelligence": 0, "wisdom": 0, "charisma": 0 }, "quality": "common", "value_copper": 5, "max_stack": 20 }
    ],
    "quests": [
        { "id": "welcome", "name": "Welcome to the World", "description": "Speak to the town elder", "level": 1, "prerequisites": [], "objectives": [{ "objective_type": "Speak", "target_id": "elder-npc", "count": 1, "description": "Speak to the town elder" }], "rewards": { "experience": 100, "gold_copper": 50, "choice_items": [], "items": [] } },
        { "id": "slay-wolves", "name": "Slay Wolves", "description": "Kill 5 wolves", "level": 2, "prerequisites": [], "objectives": [{ "objective_type": "Kill", "target_id": "wolf", "count": 5, "description": "Kill wolves" }], "rewards": { "experience": 200, "gold_copper": 100, "choice_items": ["iron-sword"], "items": [] } }
    ],
    "zones": [
        { "id": "starting-zone", "name": "Starting Zone", "min_level": 1, "max_level": 10, "safe_locations": [{ "id": "town-square", "position": [0.0, 0.0, 0.0], "yaw": 0.0 }], "controlling_factions": ["dawnward"], "spawn_tables": ["starter-mobs"], "navmesh": "nav/starter.nav" }
    ],
    "spawn_tables": [
        { "id": "starter-mobs", "entries": [{ "entity_id": "wolf", "weight": 100, "max_count": 10, "positions": [[10.0, 0.0, 10.0], [20.0, 0.0, 15.0]] }], "respawn_sec": 60 }
    ],
    "dungeons": [],
    "economy": {
        "auction_houses": [{ "id": "main-ah", "name": "Main Auction House", "zone_id": "starting-zone", "position": [5.0, 0.0, 5.0], "fee_percentage": 0.05, "min_bid_increment": 0.05, "max_listings_per_account": 20, "listing_duration_hours": 48, "deposit_percentage": 0.05 }],
        "trading_rules": [{ "item_pattern": "soulbound", "tradable": false, "auctionable": false, "mailing_allowed": false }],
        "starting_gold_copper": 100
    }
}"##;

fn bench_manifest_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest_parsing");

    group.bench_function("minimal", |b| {
        b.iter(|| load_manifest(black_box(MINIMAL_MANIFEST.as_bytes())).unwrap())
    });

    group.bench_function("realistic", |b| {
        b.iter(|| load_manifest(black_box(REALISTIC_MANIFEST.as_bytes())).unwrap())
    });

    group.finish();
}

fn bench_manifest_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest_validation");

    let minimal = load_manifest(MINIMAL_MANIFEST.as_bytes()).unwrap();
    let realistic = load_manifest(REALISTIC_MANIFEST.as_bytes()).unwrap();

    group.bench_function("minimal", |b| b.iter(|| validate(black_box(&minimal))));

    group.bench_function("realistic", |b| b.iter(|| validate(black_box(&realistic))));

    group.finish();
}

fn bench_full_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_load");

    group.bench_function("minimal", |b| {
        b.iter(|| load_manifest(black_box(MINIMAL_MANIFEST.as_bytes())))
    });

    group.bench_function("realistic", |b| {
        b.iter(|| load_manifest(black_box(REALISTIC_MANIFEST.as_bytes())))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_manifest_parsing,
    bench_manifest_validation,
    bench_full_load
);
criterion_main!(benches);
