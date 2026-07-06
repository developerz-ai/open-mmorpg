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
//! The types here, plus [`load_manifest`], are **pure**: parse + validate, no
//! I/O. Callers reading bytes from disk, an archive, or object storage hand them
//! to [`load_manifest`]. [`load_manifest_dir`] adds a convenience reader for the
//! committed split-tree layout under `content/`.

use serde::{Deserialize, Serialize};

pub mod ability;
pub mod class;
pub mod dungeon;
pub mod economy;
pub mod faction;
pub mod item;
pub mod loader;
pub mod quest;
pub mod race;
pub mod spawn;
pub mod stats;
pub mod validation;
pub mod zone;

pub use ability::{AbilityDef, AbilityEffect, AbilityEffectType};
pub use class::ClassDef;
pub use dungeon::DungeonDef;
pub use economy::{AuctionHouseDef, EconomyData, TradingRuleDef};
pub use faction::{Faction, FactionColors};
pub use item::{ItemDef, ItemType};
pub use loader::{load_manifest, load_manifest_dir};
pub use quest::{QuestDef, QuestObjective, QuestObjectiveType, QuestRewards};
pub use race::RaceDef;
pub use spawn::{SpawnEntry, SpawnTable};
pub use stats::StatModifiers;
pub use validation::validate;
pub use zone::{SafeLocation, ZoneDef};

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
