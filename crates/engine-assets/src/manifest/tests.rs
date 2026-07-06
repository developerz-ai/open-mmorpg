use super::*;

fn character_json() -> &'static str {
    r#"{
        "id": "open-mmorpg.characters",
        "version": "0.1.0",
        "api_version": 1,
        "slots": [
            {
                "id": "aurin-body",
                "kind": "character",
                "model": "aurin_body.gltf",
                "textures": {
                    "albedo": "aurin_albedo.ktx2",
                    "normal": "aurin_normal.ktx2"
                },
                "rig": "aurin_rig.gltf"
            }
        ]
    }"#
}

#[test]
fn parses_and_validates_a_character_manifest() {
    let manifest = match AssetManifest::from_json(character_json().as_bytes()) {
        Ok(m) => m,
        Err(err) => panic!("valid manifest rejected: {err}"),
    };
    assert_eq!(manifest.id, "open-mmorpg.characters");
    assert_eq!(manifest.slots.len(), 1);
    let slot = manifest.slot("aurin-body").expect("slot present");
    assert_eq!(slot.kind, SlotKind::Character);
    assert!(slot.kind.requires_rig());
    assert_eq!(slot.rig.as_deref(), Some("aurin_rig.gltf"));
}

#[test]
fn manifest_round_trips_through_json() {
    let manifest = AssetManifest::from_json(character_json().as_bytes()).expect("parse");
    let serialized = serde_json::to_vec(&manifest).expect("serialize");
    let reparsed = AssetManifest::from_json(&serialized).expect("reparse");
    assert_eq!(
        manifest, reparsed,
        "manifest must survive a JSON round-trip"
    );
}

#[test]
fn a_static_prop_slot_needs_no_rig() {
    let json = r#"{
        "id": "b", "version": "1", "api_version": 1,
        "slots": [{
            "id": "torch", "kind": "prop", "model": "torch.glb",
            "textures": { "albedo": "torch.png" }
        }]
    }"#;
    let manifest = AssetManifest::from_json(json.as_bytes()).expect("valid prop");
    let slot = manifest.slot("torch").expect("slot");
    assert!(!slot.kind.requires_rig());
    assert!(slot.rig.is_none());
}

#[test]
fn malformed_json_is_a_parse_error() {
    match AssetManifest::from_json(b"not json {{{") {
        Err(AssetError::Parse(_)) => {}
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn wrong_api_version_is_rejected() {
    let json = r#"{"id":"a","version":"1","api_version":999,"slots":[]}"#;
    match AssetManifest::from_json(json.as_bytes()) {
        Err(AssetError::UnsupportedApiVersion { expected, found }) => {
            assert_eq!(expected, ENGINE_API_VERSION);
            assert_eq!(found, 999);
        }
        other => panic!("expected UnsupportedApiVersion, got {other:?}"),
    }
}

#[test]
fn empty_model_path_is_rejected() {
    let slot = AssetSlot {
        id: "x".to_owned(),
        kind: SlotKind::Prop,
        model: String::new(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.png".to_owned())]),
        rig: None,
    };
    match slot.validate() {
        Err(AssetError::EmptyField { field: "model", .. }) => {}
        other => panic!("expected EmptyField(model), got {other:?}"),
    }
}

#[test]
fn a_proprietary_model_format_is_rejected() {
    let slot = AssetSlot {
        id: "x".to_owned(),
        kind: SlotKind::Prop,
        model: "hero.fbx".to_owned(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.ktx2".to_owned())]),
        rig: None,
    };
    match slot.validate() {
        Err(AssetError::UnsupportedFormat {
            kind: "model",
            path,
            ..
        }) => {
            assert_eq!(path, "hero.fbx");
        }
        other => panic!("expected UnsupportedFormat(model), got {other:?}"),
    }
}

#[test]
fn a_proprietary_texture_format_is_rejected() {
    let slot = AssetSlot {
        id: "x".to_owned(),
        kind: SlotKind::Prop,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.tga".to_owned())]),
        rig: None,
    };
    assert!(matches!(
        slot.validate(),
        Err(AssetError::UnsupportedFormat {
            kind: "texture",
            ..
        })
    ));
}

#[test]
fn a_slot_without_albedo_is_rejected() {
    let slot = AssetSlot {
        id: "x".to_owned(),
        kind: SlotKind::Prop,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([("normal".to_owned(), "n.ktx2".to_owned())]),
        rig: None,
    };
    match slot.validate() {
        Err(AssetError::MissingTexture { role, .. }) => assert_eq!(role, BASE_COLOR_ROLE),
        other => panic!("expected MissingTexture(albedo), got {other:?}"),
    }
}

#[test]
fn a_character_without_a_rig_is_rejected() {
    let slot = AssetSlot {
        id: "hero".to_owned(),
        kind: SlotKind::Character,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.ktx2".to_owned())]),
        rig: None,
    };
    assert!(matches!(
        slot.validate(),
        Err(AssetError::RigRequired { .. })
    ));
}

#[test]
fn a_prop_with_a_rig_is_rejected() {
    let slot = AssetSlot {
        id: "torch".to_owned(),
        kind: SlotKind::Prop,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.ktx2".to_owned())]),
        rig: Some("r.gltf".to_owned()),
    };
    assert!(matches!(
        slot.validate(),
        Err(AssetError::UnexpectedRig { .. })
    ));
}

#[test]
fn an_empty_rig_path_on_a_character_is_rejected() {
    let slot = AssetSlot {
        id: "hero".to_owned(),
        kind: SlotKind::Character,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.ktx2".to_owned())]),
        rig: Some(String::new()),
    };
    match slot.validate() {
        Err(AssetError::EmptyField { field: "rig", .. }) => {}
        other => panic!("expected EmptyField(rig), got {other:?}"),
    }
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

// ---- candidate fit ("content only has to fit the slot") ----

fn character_slot() -> AssetSlot {
    AssetSlot {
        id: "hero".to_owned(),
        kind: SlotKind::Character,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([
            ("albedo".to_owned(), "a.ktx2".to_owned()),
            ("normal".to_owned(), "n.ktx2".to_owned()),
        ]),
        rig: Some("r.gltf".to_owned()),
    }
}

#[test]
fn a_complete_candidate_fits_the_slot() {
    let slot = character_slot();
    let candidate = SlotCandidate {
        model: Some("gen_model.glb".to_owned()),
        textures: BTreeSet::from(["albedo".to_owned(), "normal".to_owned()]),
        rig: Some("gen_rig.glb".to_owned()),
    };
    assert!(slot.accepts(&candidate).is_ok());
}

#[test]
fn a_candidate_missing_the_model_is_rejected() {
    let slot = character_slot();
    let candidate = SlotCandidate {
        model: None,
        textures: BTreeSet::from(["albedo".to_owned(), "normal".to_owned()]),
        rig: Some("r.glb".to_owned()),
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::MissingModel { .. })
    ));
}

#[test]
fn a_candidate_missing_a_declared_texture_role_is_rejected() {
    let slot = character_slot();
    let candidate = SlotCandidate {
        model: Some("m.glb".to_owned()),
        textures: BTreeSet::from(["albedo".to_owned()]), // no `normal`
        rig: Some("r.glb".to_owned()),
    };
    match slot.accepts(&candidate) {
        Err(AssetError::MissingTexture { role, .. }) => assert_eq!(role, "normal"),
        other => panic!("expected MissingTexture(normal), got {other:?}"),
    }
}

#[test]
fn a_candidate_without_a_rig_fails_a_character_slot() {
    let slot = character_slot();
    let candidate = SlotCandidate {
        model: Some("m.glb".to_owned()),
        textures: BTreeSet::from(["albedo".to_owned(), "normal".to_owned()]),
        rig: None,
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::RigRequired { .. })
    ));
}

#[test]
fn a_candidate_with_a_rig_fails_a_prop_slot() {
    let slot = AssetSlot {
        id: "torch".to_owned(),
        kind: SlotKind::Prop,
        model: "m.gltf".to_owned(),
        textures: BTreeMap::from([("albedo".to_owned(), "a.png".to_owned())]),
        rig: None,
    };
    let candidate = SlotCandidate {
        model: Some("m.glb".to_owned()),
        textures: BTreeSet::from(["albedo".to_owned()]),
        rig: Some("unexpected.glb".to_owned()),
    };
    assert!(matches!(
        slot.accepts(&candidate),
        Err(AssetError::UnexpectedRig { .. })
    ));
}

// ---- helpers ----

#[test]
fn extension_is_case_insensitive_and_handles_edge_cases() {
    assert_eq!(extension("a.b.KTX2").as_deref(), Some("ktx2"));
    assert_eq!(extension("model.glb").as_deref(), Some("glb"));
    assert_eq!(extension("noext"), None);
    assert_eq!(extension("trailingdot."), None);
}

#[test]
fn allowed_format_strings_match_their_arrays() {
    // Guard against the human-readable error strings drifting from the arrays.
    for ext in MODEL_FORMATS {
        assert!(
            MODEL_FORMATS_STR.contains(ext),
            "MODEL_FORMATS_STR missing {ext}"
        );
    }
    for ext in TEXTURE_FORMATS {
        assert!(
            TEXTURE_FORMATS_STR.contains(ext),
            "TEXTURE_FORMATS_STR missing {ext}"
        );
    }
}
