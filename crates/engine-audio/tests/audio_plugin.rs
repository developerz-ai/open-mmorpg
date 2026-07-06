//! Integration test: the pure audio pipeline end to end — world AoI cull →
//! attenuate → voice budget — exercised through the crate's public API only.
//!
//! Device-free by construction: it never adds `AudioPlugin` to an `App`, so it is
//! safe under CI's `--all-features` run (which has no audio sink).

use bevy_math::Vec3;
use omm_engine_audio::{Attenuation, AudioAoi, EmitterInput, Listener, Rolloff};
use omm_protocol::Vec3 as WorldVec3;
use omm_world::{Aabb, EntityId, Quadtree};

#[test]
fn world_aoi_to_voice_budget_pipeline() {
    // A world with three emitters: two within earshot, one across the map.
    let mut tree = Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 6, 4);
    tree.insert(
        EntityId(1),
        WorldVec3 {
            x: 50.0,
            y: 0.0,
            z: 51.0,
        },
    );
    tree.insert(
        EntityId(2),
        WorldVec3 {
            x: 50.0,
            y: 0.0,
            z: 55.0,
        },
    );
    tree.insert(
        EntityId(3),
        WorldVec3 {
            x: 95.0,
            y: 0.0,
            z: 95.0,
        },
    );

    let listener_pos = Vec3::new(50.0, 0.0, 50.0);
    let attenuation = Attenuation::new(1.0, 10.0, Rolloff::Inverse).expect("valid attenuation");
    let aoi = AudioAoi::for_attenuation(&attenuation, 8).expect("valid aoi");

    // Coarse cull reuses the world quadtree.
    let audible = aoi.audible_ids(&tree, listener_pos);
    assert_eq!(
        audible,
        vec![EntityId(1), EntityId(2)],
        "far emitter culled"
    );

    // Resolve the culled ids to positions (the ECS bridge, faked here).
    let candidates: Vec<EmitterInput> = audible
        .iter()
        .map(|&id| {
            let z = if id == EntityId(1) { 51.0 } else { 55.0 };
            EmitterInput {
                id,
                position: Vec3::new(50.0, 0.0, z),
            }
        })
        .collect();

    let listener = Listener::new(listener_pos, Vec3::NEG_Z, Vec3::Y).expect("valid listener");
    let voices = aoi.mix(&listener, &attenuation, &candidates);

    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![1, 2], "nearest first");
    assert!(voices[0].mix.gain > voices[1].mix.gain, "nearer is louder");
}
