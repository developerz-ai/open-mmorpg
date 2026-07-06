//! Integration tests for manifest slot validation — valid and invalid slots
//! exercised through the public API only, no `crate::` imports.
//!
//! These mirror what a CI agent or content-pipeline tool does when it validates
//! a manifest drop: parse the bytes, check every slot, and gate AI-generated
//! bundles against the slot spec.

use omm_engine_assets::{AssetError, AssetManifest, SlotCandidate, SlotKind};

// ── fixtures ──────────────────────────────────────────────────────────────────

const CHARACTER_MANIFEST: &str = r#"{
    "id": "open-mmorpg.characters",
    "version": "0.2.0",
    "api_version": 1,
    "slots": [
        {
            "id": "aurin-body",
            "kind": "character",
            "model": "aurin_body.gltf",
            "textures": {
                "albedo": "aurin_albedo.ktx2",
                "normal": "aurin_normal.ktx2",
                "emissive": "aurin_emissive.ktx2"
            },
            "rig": "aurin_rig.gltf"
        }
    ]
}"#;

const PROP_MANIFEST: &str = r#"{
    "id": "open-mmorpg.props",
    "version": "0.1.0",
    "api_version": 1,
    "slots": [
        {
            "id": "torch",
            "kind": "prop",
            "model": "torch.glb",
            "textures": { "albedo": "torch.png" }
        }
    ]
}"#;

// ── valid slot: character ─────────────────────────────────────────────────────

#[test]
fn valid_character_slot_parses_and_validates() {
    let manifest =
        AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("valid character manifest");
    assert_eq!(manifest.id, "open-mmorpg.characters");
    assert_eq!(manifest.slots.len(), 1);

    let slot = manifest.slot("aurin-body").expect("slot present");
    assert_eq!(slot.kind, SlotKind::Character);
    assert!(slot.kind.requires_rig());
    assert_eq!(slot.rig.as_deref(), Some("aurin_rig.gltf"));
    assert_eq!(slot.textures.len(), 3);
}

#[test]
fn character_manifest_round_trips_through_json() {
    let manifest = AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("parse");
    let bytes = serde_json::to_vec(&manifest).expect("serialize");
    let reparsed = AssetManifest::from_json(&bytes).expect("reparse");
    assert_eq!(
        manifest, reparsed,
        "manifest must survive a JSON round-trip"
    );
}

// ── valid slot: prop ──────────────────────────────────────────────────────────

#[test]
fn valid_prop_slot_parses_and_validates() {
    let manifest = AssetManifest::from_json(PROP_MANIFEST.as_bytes()).expect("valid prop manifest");
    let slot = manifest.slot("torch").expect("slot present");
    assert_eq!(slot.kind, SlotKind::Prop);
    assert!(!slot.kind.requires_rig());
    assert!(slot.rig.is_none(), "props carry no rig");
}

// ── invalid slot: parse errors ────────────────────────────────────────────────

#[test]
fn malformed_json_yields_parse_error() {
    match AssetManifest::from_json(b"{ not json }}}") {
        Err(AssetError::Parse(_)) => {}
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn wrong_api_version_is_rejected() {
    let json = r#"{"id":"a","version":"1","api_version":42,"slots":[]}"#;
    match AssetManifest::from_json(json.as_bytes()) {
        Err(AssetError::UnsupportedApiVersion { found: 42, .. }) => {}
        other => panic!("expected UnsupportedApiVersion(42), got {other:?}"),
    }
}

// ── invalid slot: field-level errors ─────────────────────────────────────────

#[test]
fn empty_model_path_is_rejected() {
    let json = r#"{
        "id":"x","version":"1","api_version":1,
        "slots":[{
            "id":"empty-model","kind":"prop","model":"",
            "textures":{"albedo":"a.png"}
        }]
    }"#;
    match AssetManifest::from_json(json.as_bytes()) {
        Err(AssetError::EmptyField { field: "model", .. }) => {}
        other => panic!("expected EmptyField(model), got {other:?}"),
    }
}

#[test]
fn proprietary_model_extension_is_rejected() {
    let json = r#"{
        "id":"x","version":"1","api_version":1,
        "slots":[{
            "id":"hero","kind":"prop","model":"hero.fbx",
            "textures":{"albedo":"a.ktx2"}
        }]
    }"#;
    match AssetManifest::from_json(json.as_bytes()) {
        Err(AssetError::UnsupportedFormat {
            kind: "model",
            path,
            ..
        }) => assert_eq!(path, "hero.fbx"),
        other => panic!("expected UnsupportedFormat(model), got {other:?}"),
    }
}

#[test]
fn proprietary_texture_extension_is_rejected() {
    let json = r#"{
        "id":"x","version":"1","api_version":1,
        "slots":[{
            "id":"hero","kind":"prop","model":"m.gltf",
            "textures":{"albedo":"a.tga"}
        }]
    }"#;
    assert!(matches!(
        AssetManifest::from_json(json.as_bytes()),
        Err(AssetError::UnsupportedFormat {
            kind: "texture",
            ..
        })
    ));
}

#[test]
fn slot_without_albedo_is_rejected() {
    let json = r#"{
        "id":"x","version":"1","api_version":1,
        "slots":[{
            "id":"hero","kind":"prop","model":"m.gltf",
            "textures":{"normal":"n.ktx2"}
        }]
    }"#;
    match AssetManifest::from_json(json.as_bytes()) {
        Err(AssetError::MissingTexture { role, .. }) => assert_eq!(role, "albedo"),
        other => panic!("expected MissingTexture(albedo), got {other:?}"),
    }
}

#[test]
fn character_without_rig_is_rejected() {
    let json = r#"{
        "id":"x","version":"1","api_version":1,
        "slots":[{
            "id":"hero","kind":"character","model":"m.gltf",
            "textures":{"albedo":"a.ktx2"}
        }]
    }"#;
    assert!(matches!(
        AssetManifest::from_json(json.as_bytes()),
        Err(AssetError::RigRequired { .. })
    ));
}

#[test]
fn prop_with_rig_is_rejected() {
    let json = r#"{
        "id":"x","version":"1","api_version":1,
        "slots":[{
            "id":"torch","kind":"prop","model":"m.gltf",
            "textures":{"albedo":"a.ktx2"},
            "rig":"r.gltf"
        }]
    }"#;
    assert!(matches!(
        AssetManifest::from_json(json.as_bytes()),
        Err(AssetError::UnexpectedRig { .. })
    ));
}

#[test]
fn duplicate_slot_ids_are_rejected() {
    let json = r#"{
        "id":"a","version":"1","api_version":1,
        "slots":[
            {"id":"dup","kind":"prop","model":"m.gltf","textures":{"albedo":"a.png"}},
            {"id":"dup","kind":"prop","model":"n.gltf","textures":{"albedo":"b.png"}}
        ]
    }"#;
    match AssetManifest::from_json(json.as_bytes()) {
        Err(AssetError::DuplicateSlot { id }) => assert_eq!(id, "dup"),
        other => panic!("expected DuplicateSlot, got {other:?}"),
    }
}

// ── candidate fit — "content only has to fit the slot" ───────────────────────

#[test]
fn complete_candidate_fits_a_character_slot() {
    let manifest = AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("parse");
    let slot = manifest.slot("aurin-body").expect("slot");
    let candidate = SlotCandidate {
        model: Some("gen_aurin.glb".to_owned()),
        textures: ["albedo", "normal", "emissive"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
        rig: Some("gen_rig.glb".to_owned()),
    };
    assert!(
        slot.accepts(&candidate).is_ok(),
        "complete bundle must fit the slot"
    );
}

#[test]
fn candidate_missing_model_is_rejected() {
    let manifest = AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("parse");
    let slot = manifest.slot("aurin-body").expect("slot");
    let candidate = SlotCandidate {
        model: None,
        textures: ["albedo", "normal", "emissive"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
        rig: Some("gen_rig.glb".to_owned()),
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::MissingModel { .. })
    ));
}

#[test]
fn candidate_missing_declared_texture_role_is_rejected() {
    let manifest = AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("parse");
    let slot = manifest.slot("aurin-body").expect("slot");
    // Only albedo — missing normal and emissive.
    let candidate = SlotCandidate {
        model: Some("gen.glb".to_owned()),
        textures: ["albedo"].into_iter().map(ToOwned::to_owned).collect(),
        rig: Some("gen_rig.glb".to_owned()),
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::MissingTexture { .. })
    ));
}

#[test]
fn candidate_without_rig_fails_character_slot() {
    let manifest = AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("parse");
    let slot = manifest.slot("aurin-body").expect("slot");
    let candidate = SlotCandidate {
        model: Some("gen.glb".to_owned()),
        textures: ["albedo", "normal", "emissive"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
        rig: None,
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::RigRequired { .. })
    ));
}

#[test]
fn candidate_with_rig_fails_prop_slot() {
    let manifest = AssetManifest::from_json(PROP_MANIFEST.as_bytes()).expect("parse");
    let slot = manifest.slot("torch").expect("slot");
    let candidate = SlotCandidate {
        model: Some("gen_torch.glb".to_owned()),
        textures: ["albedo"].into_iter().map(ToOwned::to_owned).collect(),
        rig: Some("unexpected_rig.glb".to_owned()),
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::UnexpectedRig { .. })
    ));
}

// ── slot lookup ───────────────────────────────────────────────────────────────

#[test]
fn slot_lookup_by_id_works() {
    let manifest = AssetManifest::from_json(CHARACTER_MANIFEST.as_bytes()).expect("parse");
    assert!(manifest.slot("aurin-body").is_some());
    assert!(manifest.slot("nonexistent").is_none());
}

// ── multiple slots in one manifest ───────────────────────────────────────────

#[test]
fn manifest_with_mixed_slot_kinds_validates() {
    let json = r#"{
        "id":"mixed","version":"1","api_version":1,
        "slots":[
            {
                "id":"hero","kind":"character","model":"m.gltf",
                "textures":{"albedo":"a.ktx2"},"rig":"r.gltf"
            },
            {
                "id":"torch","kind":"prop","model":"t.glb",
                "textures":{"albedo":"t.png"}
            }
        ]
    }"#;
    let manifest = AssetManifest::from_json(json.as_bytes()).expect("valid mixed manifest");
    assert_eq!(manifest.slots.len(), 2);
    assert_eq!(
        manifest.slot("hero").map(|s| s.kind),
        Some(SlotKind::Character)
    );
    assert_eq!(manifest.slot("torch").map(|s| s.kind), Some(SlotKind::Prop));
}
