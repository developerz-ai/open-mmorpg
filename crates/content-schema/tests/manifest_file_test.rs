//! Guards the committed datapack.
//!
//! `include_str!` embeds `content/manifest.json` at compile time, so this test
//! always validates the *real* committed datapack — not an inline fixture — on
//! every build. If the file moves or grows invalid, this fails (or won't
//! compile), closing the gap where only inline test samples were validated.

use omm_content_schema::load_manifest;

const COMMITTED_MANIFEST: &str = include_str!("../../../content/manifest.json");

/// The two canon rival factions (docs/specs/gameplay/factions).
const CONCORD: &str = "aurelian-concord";
const PACT: &str = "wildreach-pact";

#[test]
fn committed_manifest_loads_and_validates() {
    let manifest = load_manifest(COMMITTED_MANIFEST.as_bytes())
        .expect("content/manifest.json must parse and pass validate()");
    assert_eq!(
        manifest.api_version,
        omm_content_schema::CONTENT_API_VERSION
    );
    assert_eq!(manifest.id, "open-mmorpg.base");

    // Two canon factions, mutually hostile, each with a capital zone.
    let faction_ids: Vec<&str> = manifest.factions.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(faction_ids, [CONCORD, PACT]);
    let concord = manifest.factions.iter().find(|f| f.id == CONCORD).unwrap();
    assert_eq!(concord.hostile_to, vec![PACT]);
    assert!(concord.capital.as_deref().is_some_and(|c| !c.is_empty()));

    // The slice a player actually picks from at character creation.
    assert_eq!(manifest.races.len(), 12, "complete racial slate");
    assert!(manifest.classes.len() >= 4);
    assert!(
        manifest
            .races
            .iter()
            .all(|r| r.faction_id == CONCORD || r.faction_id == PACT),
        "every race must belong to a canon faction"
    );
    assert!(
        manifest.classes.iter().all(|c| !c.abilities.is_empty()),
        "every class must grant at least one ability"
    );
    assert!(
        manifest.races.iter().all(|r| r
            .traits
            .iter()
            .all(|t| manifest.abilities.iter().any(|a| a.id == *t))),
        "every racial trait must resolve to an ability"
    );

    // A walkable, populated world: at least one zone, spawn table, dungeon, AH.
    assert!(!manifest.zones.is_empty());
    assert!(!manifest.spawn_tables.is_empty());
    assert_eq!(manifest.dungeons.len(), 1);
    assert!(!manifest.economy.auction_houses.is_empty());

    // The dungeon sits inside a known zone, and its loot resolves to real items.
    let dungeon = &manifest.dungeons[0];
    assert!(manifest
        .zones
        .iter()
        .any(|z| Some(z.id.as_str()) == dungeon.entrance_zone_id.as_deref()));
}
