//! Guards the committed datapack.
//!
//! `include_str!` embeds `content/manifest.json` at compile time, so this test
//! always validates the *real* committed datapack — not an inline fixture — on
//! every build. If the file moves or grows invalid, this fails (or won't
//! compile), closing the gap where only inline test samples were validated.

use omm_content_schema::load_manifest;

const COMMITTED_MANIFEST: &str = include_str!("../../../content/manifest.json");

#[test]
fn committed_manifest_loads_and_validates() {
    let manifest = load_manifest(COMMITTED_MANIFEST.as_bytes())
        .expect("content/manifest.json must parse and pass validate()");
    assert_eq!(
        manifest.api_version,
        omm_content_schema::CONTENT_API_VERSION
    );
    assert!(!manifest.id.trim().is_empty());
    // The seed pack ships two rival factions (see docs/specs/gameplay/factions).
    assert_eq!(manifest.factions.len(), 2);
}
