//! Unit tests for AoI-scoped emitter selection and voice budgeting.

use bevy_math::Vec3;
use omm_protocol::Vec3 as WorldVec3;
use omm_world::{Aabb, EntityId, Quadtree};

use super::*;
use crate::spatial::Rolloff;

fn world() -> Quadtree {
    Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 6, 4)
}

fn wpos(x: f32, z: f32) -> WorldVec3 {
    WorldVec3 { x, y: 0.0, z }
}

fn listener_at(pos: Vec3) -> Listener {
    Listener::new(pos, Vec3::NEG_Z, Vec3::Y).expect("valid listener")
}

#[test]
fn new_rejects_bad_config() {
    assert!(AudioAoi::new(0.0, 8).is_err());
    assert!(AudioAoi::new(-1.0, 8).is_err());
    assert!(AudioAoi::new(f32::NAN, 8).is_err());
    assert!(AudioAoi::new(10.0, 0).is_err());
    assert!(AudioAoi::new(10.0, 8).is_ok());
}

#[test]
fn for_attenuation_matches_far_edge() {
    let a = Attenuation::new(2.0, 40.0, Rolloff::Inverse).expect("valid");
    let aoi = AudioAoi::for_attenuation(&a, 16).expect("valid");
    assert!((aoi.radius - 40.0).abs() < 1e-6);
    assert_eq!(aoi.max_voices, 16);
}

#[test]
fn audible_ids_reuses_world_quadtree() {
    let mut tree = world();
    tree.insert(EntityId(1), wpos(50.0, 50.0)); // on the listener tile
    tree.insert(EntityId(2), wpos(52.0, 50.0)); // near
    tree.insert(EntityId(3), wpos(95.0, 95.0)); // far

    let aoi = AudioAoi::new(5.0, 32).expect("valid");
    let ids = aoi.audible_ids(&tree, Vec3::new(50.0, 0.0, 50.0));
    assert_eq!(ids, vec![EntityId(1), EntityId(2)]);
}

#[test]
fn audible_ids_empty_world_is_empty() {
    let tree = world();
    let aoi = AudioAoi::new(25.0, 32).expect("valid");
    assert!(aoi
        .audible_ids(&tree, Vec3::new(10.0, 0.0, 10.0))
        .is_empty());
}

#[test]
fn mix_drops_inaudible_and_orders_by_distance() {
    let listener = listener_at(Vec3::ZERO);
    let atten = Attenuation::new(1.0, 10.0, Rolloff::Linear).expect("valid");
    let candidates = [
        EmitterInput {
            id: EntityId(7),
            position: Vec3::new(0.0, 0.0, 4.0),
        }, // mid
        EmitterInput {
            id: EntityId(3),
            position: Vec3::new(0.0, 0.0, 2.0),
        }, // nearest
        EmitterInput {
            id: EntityId(9),
            position: Vec3::new(0.0, 0.0, 50.0),
        }, // beyond max → dropped
    ];
    let aoi = AudioAoi::new(60.0, 32).expect("valid");
    let voices = aoi.mix(&listener, &atten, &candidates);

    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![3, 7], "inaudible dropped, nearest first");
    assert!(voices[0].mix.gain > voices[1].mix.gain, "nearer is louder");
    for v in &voices {
        assert!(v.mix.gain > 0.0);
    }
}

#[test]
fn mix_caps_voices_to_budget_keeping_nearest() {
    let listener = listener_at(Vec3::ZERO);
    let atten = Attenuation::new(1.0, 100.0, Rolloff::Linear).expect("valid");
    let candidates: Vec<EmitterInput> = [1u64, 2, 3, 4, 5, 6]
        .into_iter()
        .map(|i| EmitterInput {
            id: EntityId(i),
            position: Vec3::new(0.0, 0.0, i as f32 * 5.0),
        })
        .collect();
    let aoi = AudioAoi::new(200.0, 3).expect("valid");
    let voices = aoi.mix(&listener, &atten, &candidates);

    assert_eq!(voices.len(), 3, "capped to max_voices");
    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![1, 2, 3], "kept the three nearest");
}

#[test]
fn mix_breaks_distance_ties_by_id_deterministically() {
    let listener = listener_at(Vec3::ZERO);
    let atten = Attenuation::new(1.0, 50.0, Rolloff::Linear).expect("valid");
    // Two equidistant emitters (same distance, mirrored) — tie must break on id.
    let candidates = [
        EmitterInput {
            id: EntityId(8),
            position: Vec3::new(5.0, 0.0, 0.0),
        },
        EmitterInput {
            id: EntityId(2),
            position: Vec3::new(-5.0, 0.0, 0.0),
        },
    ];
    let aoi = AudioAoi::new(60.0, 32).expect("valid");
    let voices = aoi.mix(&listener, &atten, &candidates);
    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![2, 8], "lower id wins the tie");
}

#[test]
fn mix_empty_candidates_is_empty() {
    let listener = listener_at(Vec3::ZERO);
    let atten = Attenuation::default();
    let aoi = AudioAoi::default();
    assert!(aoi.mix(&listener, &atten, &[]).is_empty());
}
