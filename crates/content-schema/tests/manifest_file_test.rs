//! Guards the committed datapack.
//!
//! Loads the *real* committed split-tree datapack under `content/` — not an
//! inline fixture — via [`load_manifest_dir`], so every build re-validates the
//! actual data on disk. If the tree grows invalid or a domain dir goes missing,
//! this fails, closing the gap where only inline test samples were validated.

use std::path::PathBuf;

use omm_content_schema::load_manifest_dir;

/// Resolve `content/` relative to this crate (workspace root holds it).
fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content")
}

/// The two canon rival factions (docs/specs/gameplay/factions).
const CONCORD: &str = "aurelian-concord";
const PACT: &str = "wildreach-pact";

#[test]
fn committed_manifest_loads_and_validates() {
    let manifest =
        load_manifest_dir(&content_dir()).expect("content/ must parse and pass validate()");
    assert_eq!(
        manifest.api_version,
        omm_content_schema::CONTENT_API_VERSION
    );
    assert_eq!(manifest.id, "open-mmorpg.base");

    // Two canon factions, mutually hostile, each with a capital zone. The dir
    // loader emits entities in sorted-id order, which is the canon order here.
    let faction_ids: Vec<&str> = manifest.factions.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(faction_ids, [CONCORD, PACT]);
    let concord = manifest.factions.iter().find(|f| f.id == CONCORD).unwrap();
    assert_eq!(concord.hostile_to, vec![PACT]);
    assert!(concord.capital.as_deref().is_some_and(|c| !c.is_empty()));

    // The slice a player actually picks from at character creation.
    assert_eq!(manifest.races.len(), 12, "complete racial slate");
    assert_eq!(manifest.classes.len(), 13, "complete hero class set");
    assert!(
        manifest
            .races
            .iter()
            .all(|r| r.faction_id == CONCORD || r.faction_id == PACT),
        "every race must belong to a canon faction"
    );
    assert!(
        manifest.classes.iter().all(|c| c.abilities.len() >= 3),
        "every class must grant at least three abilities"
    );
    assert!(
        manifest.races.iter().all(|r| r
            .traits
            .iter()
            .all(|t| manifest.abilities.iter().any(|a| a.id == *t))),
        "every racial trait must resolve to an ability"
    );
    assert!(
        manifest.classes.iter().all(|c| c
            .abilities
            .iter()
            .all(|a| manifest.abilities.iter().any(|ability| ability.id == *a))),
        "every class ability must resolve to an ability definition"
    );

    // A walkable, populated world: at least one zone, spawn table, dungeon, AH.
    assert!(!manifest.zones.is_empty());
    assert!(!manifest.spawn_tables.is_empty());
    assert!(!manifest.dungeons.is_empty());
    assert!(!manifest.economy.auction_houses.is_empty());

    // Each dungeon sits inside a known zone.
    for dungeon in &manifest.dungeons {
        assert!(manifest
            .zones
            .iter()
            .any(|z| Some(z.id.as_str()) == dungeon.entrance_zone_id.as_deref()));
        // Dungeon loot tables must reference real items.
        for item_id in &dungeon.loot_tables {
            assert!(
                manifest.items.iter().any(|i| i.id == *item_id),
                "dungeon '{}' loot_tables references unknown item '{item_id}'",
                dungeon.id
            );
        }
    }
}
