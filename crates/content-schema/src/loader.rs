//! Manifest loading.
//!
//! Two entry points:
//! - [`load_manifest`] parses a single assembled manifest document (pure: bytes
//!   in, [`Manifest`] out). Callers that read bytes from an archive or object
//!   storage use this.
//! - [`load_manifest_dir`] reads the on-disk split-tree layout (`content/` with
//!   one file per entity under per-domain directories) and assembles a
//!   [`Manifest`]. This is the layout the committed datapack ships in.

use std::fs;
use std::path::{Path, PathBuf};

use omm_errors::{CoreError, CoreResult};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::{
    ability::AbilityDef, class::ClassDef, dungeon::DungeonDef, economy::AuctionHouseDef,
    economy::EconomyData, economy::TradingRuleDef, faction::Faction, item::ItemDef,
    quest::QuestDef, race::RaceDef, spawn::SpawnTable, validation::validate, zone::ZoneDef,
    Manifest,
};

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

/// Load and validate the committed split-tree datapack at `content_dir`.
///
/// `content_dir` must contain a metadata `manifest.json` plus per-domain
/// directories (`factions/`, `races/`, `classes/`, `abilities/`, `items/`,
/// `quests/`, `zones/`, `spawn-tables/`, `dungeons/`, `economy/`), where each
/// `.json` file holds a single entity. Files are read in sorted path order so
/// the resulting [`Manifest`] is deterministic regardless of filesystem
/// traversal order.
///
/// # Errors
/// [`CoreError::BadRequest`] for any unreadable file, malformed JSON, or
/// validation failure.
pub fn load_manifest_dir(content_dir: &Path) -> CoreResult<Manifest> {
    let root_path = content_dir.join("manifest.json");
    let root_bytes = fs::read(&root_path)
        .map_err(|e| CoreError::BadRequest(format!("manifest.json unreadable: {e}")))?;
    let mut manifest: Manifest = serde_json::from_slice(&root_bytes)
        .map_err(|e| CoreError::BadRequest(format!("manifest json: {e}")))?;

    // Only datapack metadata is read from the root; the domains come from the
    // per-entity tree, so any domain arrays present in the root are ignored.
    manifest.factions = load_domain_dir::<Faction>(&content_dir.join("factions"))?;
    manifest.races = load_domain_dir::<RaceDef>(&content_dir.join("races"))?;
    manifest.classes = load_domain_dir::<ClassDef>(&content_dir.join("classes"))?;
    manifest.abilities = load_domain_dir::<AbilityDef>(&content_dir.join("abilities"))?;
    manifest.items = load_domain_dir::<ItemDef>(&content_dir.join("items"))?;
    manifest.quests = load_domain_dir::<QuestDef>(&content_dir.join("quests"))?;
    manifest.zones = load_domain_dir::<ZoneDef>(&content_dir.join("zones"))?;
    manifest.spawn_tables = load_domain_dir::<SpawnTable>(&content_dir.join("spawn-tables"))?;
    manifest.dungeons = load_domain_dir::<DungeonDef>(&content_dir.join("dungeons"))?;
    manifest.economy = load_economy(&content_dir.join("economy"))?;

    validate(&manifest)?;
    Ok(manifest)
}

/// Read every `.json` file under `dir` (recursively), in sorted path order, as
/// a single entity of type `T`. Returns an empty vec when the dir is absent.
fn load_domain_dir<T: DeserializeOwned>(dir: &Path) -> CoreResult<Vec<T>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = collect_json_files(dir)?;
    paths.sort();
    paths.into_iter().map(|p| parse_file(&p)).collect()
}

/// Assemble [`EconomyData`] from its `economy/` subdirectories plus
/// `economy/starting.json`.
fn load_economy(dir: &Path) -> CoreResult<EconomyData> {
    if !dir.is_dir() {
        return Ok(EconomyData::default());
    }
    let auction_houses = load_domain_dir::<AuctionHouseDef>(&dir.join("auction-houses"))?;
    let trading_rules = load_domain_dir::<TradingRuleDef>(&dir.join("trading-rules"))?;
    let starting_gold_copper = load_starting_gold(&dir.join("starting.json"))?;
    Ok(EconomyData {
        auction_houses,
        trading_rules,
        starting_gold_copper,
    })
}

#[derive(Deserialize)]
struct StartingGold {
    #[serde(default)]
    starting_gold_copper: u32,
}

fn load_starting_gold(path: &Path) -> CoreResult<u32> {
    if !path.is_file() {
        return Ok(0);
    }
    Ok(parse_file::<StartingGold>(path)?.starting_gold_copper)
}

fn collect_json_files(dir: &Path) -> CoreResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_json_files_into(dir, &mut out)?;
    Ok(out)
}

fn collect_json_files_into(dir: &Path, out: &mut Vec<PathBuf>) -> CoreResult<()> {
    let read_dir = fs::read_dir(dir).map_err(|e| {
        CoreError::BadRequest(format!("content dir '{}' unreadable: {e}", dir.display()))
    })?;
    for entry in read_dir {
        let path = entry
            .map_err(|e| {
                CoreError::BadRequest(format!("content dir '{}' unreadable: {e}", dir.display()))
            })?
            .path();
        if path.is_dir() {
            collect_json_files_into(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }
    Ok(())
}

fn parse_file<T: DeserializeOwned>(path: &Path) -> CoreResult<T> {
    let bytes = fs::read(path).map_err(|e| {
        CoreError::BadRequest(format!("content file '{}' unreadable: {e}", path.display()))
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::BadRequest(format!("content file '{}' json: {e}", path.display())))
}
