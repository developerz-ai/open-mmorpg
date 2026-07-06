//! End-to-end exercise of the pure asset-pipeline surface through the crate's
//! **public** API: manifest slot validation → candidate fit → LOD selection →
//! camera-position streaming under a memory budget. Mirrors what a headless agent
//! does when it drops AI-generated content in and reasons about detail/streaming.

use core::num::NonZeroU32;

use bevy_math::{Vec2, Vec3};
use omm_engine_assets::{
    AssetError, AssetManifest, ImposterAtlas, LodChain, LodSelection, SlotCandidate, StreamingGrid,
};

const MANIFEST: &str = r#"{
    "id": "open-mmorpg.characters",
    "version": "0.1.0",
    "api_version": 1,
    "slots": [
        {
            "id": "aurin-body",
            "kind": "character",
            "model": "aurin_body.gltf",
            "textures": { "albedo": "aurin_albedo.ktx2", "normal": "aurin_normal.ktx2" },
            "rig": "aurin_rig.gltf"
        },
        {
            "id": "torch",
            "kind": "prop",
            "model": "torch.glb",
            "textures": { "albedo": "torch_albedo.png" }
        }
    ]
}"#;

#[test]
fn manifest_validates_and_gates_generated_content() {
    let manifest = AssetManifest::from_json(MANIFEST.as_bytes()).expect("valid manifest");
    assert_eq!(manifest.slots.len(), 2);

    let hero = manifest.slot("aurin-body").expect("character slot");

    // A generated bundle that supplies the model, both texture roles and a rig fits.
    let good = SlotCandidate {
        model: Some("gen_aurin.glb".to_owned()),
        textures: ["albedo".to_owned(), "normal".to_owned()]
            .into_iter()
            .collect(),
        rig: Some("gen_aurin_rig.glb".to_owned()),
    };
    assert!(
        hero.accepts(&good).is_ok(),
        "complete bundle must fit the slot"
    );

    // A bundle missing the normal map is rejected, naming the missing role.
    let missing_normal = SlotCandidate {
        model: Some("gen_aurin.glb".to_owned()),
        textures: ["albedo".to_owned()].into_iter().collect(),
        rig: Some("gen_aurin_rig.glb".to_owned()),
    };
    assert!(matches!(
        hero.accepts(&missing_normal),
        Err(AssetError::MissingTexture { role, .. }) if role == "normal"
    ));
}

#[test]
fn lod_picks_tiers_by_screen_size_and_imposter_far() {
    let lod = LodChain::new(vec![0.5, 0.2, 0.05], true, 0.01).expect("valid chain");
    assert_eq!(lod.select(0.9), LodSelection::Mesh(0)); // near/large → finest
    assert_eq!(lod.select(0.1), LodSelection::Mesh(2)); // mid distance → coarse mesh
    assert_eq!(lod.select(0.02), LodSelection::Imposter); // far → octahedral imposter
    assert_eq!(lod.select(0.0), LodSelection::Culled); // tiny → not drawn

    // The far tier samples a deterministic cell of the octahedral view atlas.
    let atlas = ImposterAtlas::new(NonZeroU32::new(8).expect("nonzero"));
    let cell = atlas.cell_for_view(Vec3::Z);
    assert!(cell.x < 8 && cell.y < 8);
    assert_eq!(cell, atlas.cell_for_view(Vec3::Z));
}

#[test]
fn streaming_holds_the_nearest_tiles_within_budget() {
    // Budget for 4 tiles; a wide view wants more, so the farthest are skipped.
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("valid grid");
    let delta = grid
        .update(Vec2::new(5.0, 5.0), |_| 100)
        .expect("streaming update");

    assert_eq!(grid.resident_bytes(), 400, "never over budget");
    assert!(
        grid.is_loaded(grid.tile_at(Vec2::new(5.0, 5.0))),
        "camera tile resident"
    );
    assert_eq!(grid.loaded_tiles().count(), 4);
    assert!(
        !delta.skipped.is_empty(),
        "budget pressure is reported, not hidden"
    );
}
