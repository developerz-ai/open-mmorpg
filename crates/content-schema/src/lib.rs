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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

/// A faction players can belong to. Pure data — no behavior compiled in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faction {
    /// Stable machine id, referenced by other content.
    pub id: String,
    /// Display name (shown via the web/client i18n layer).
    pub name: String,
    /// Faction ids this one is hostile toward.
    #[serde(default)]
    pub hostile_to: Vec<String>,
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
    let known: std::collections::HashSet<&str> =
        manifest.factions.iter().map(|f| f.id.as_str()).collect();
    for faction in &manifest.factions {
        if faction.id.trim().is_empty() {
            return Err(CoreError::BadRequest("faction id is empty".into()));
        }
        for target in &faction.hostile_to {
            if !known.contains(target.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "faction '{}' is hostile to unknown faction '{target}'",
                    faction.id
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
