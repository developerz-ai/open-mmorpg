//! Integration tests for AoI-scoped emitter selection: **in-range** and
//! **out-of-range** culling via the world quadtree, plus voice-budget caps.
//!
//! These tests exercise only the public crate API (`omm_engine_audio::*`). No
//! audio device is opened — safe under CI `--all-features`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy_math::Vec3;
use omm_engine_audio::{Attenuation, AudioAoi, EmitterInput, Listener, Rolloff};
use omm_protocol::Vec3 as WorldVec3;
use omm_world::{Aabb, EntityId, Quadtree};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// A 100×100 world centred on the origin-facing positive X/Z quadrant.
fn world() -> Quadtree {
    Quadtree::new(Aabb::new(0.0, 0.0, 100.0, 100.0), 6, 4)
}

fn wpos(x: f32, z: f32) -> WorldVec3 {
    WorldVec3 { x, y: 0.0, z }
}

fn listener_at(pos: Vec3) -> Listener {
    Listener::new(pos, Vec3::NEG_Z, Vec3::Y).expect("valid listener")
}

fn atten() -> Attenuation {
    Attenuation::new(1.0, 10.0, Rolloff::Linear).expect("valid attenuation")
}

// ── `AudioAoi::new` validation ───────────────────────────────────────────────

#[test]
fn aoi_new_rejects_bad_config() {
    assert!(AudioAoi::new(0.0, 8).is_err(), "zero radius");
    assert!(AudioAoi::new(-1.0, 8).is_err(), "negative radius");
    assert!(AudioAoi::new(f32::NAN, 8).is_err(), "NaN radius");
    assert!(AudioAoi::new(f32::INFINITY, 8).is_err(), "infinite radius");
    assert!(AudioAoi::new(10.0, 0).is_err(), "zero max_voices");
    assert!(AudioAoi::new(10.0, 8).is_ok(), "valid config");
}

#[test]
fn for_attenuation_sets_radius_to_max_distance() {
    let a = Attenuation::new(2.0, 40.0, Rolloff::Inverse).expect("valid");
    let aoi = AudioAoi::for_attenuation(&a, 16).expect("valid");
    assert!(
        (aoi.radius - 40.0).abs() < 1e-6,
        "radius should equal max_distance: {}",
        aoi.radius
    );
    assert_eq!(aoi.max_voices, 16);
}

// ── In-range / out-of-range cull (`audible_ids`) ─────────────────────────────

/// Emitters within `radius` are returned; emitters beyond are culled.
#[test]
fn audible_ids_includes_in_range_excludes_out_of_range() {
    let mut tree = world();
    let listener_pos = Vec3::new(50.0, 0.0, 50.0);

    // id=1: 1 unit away → in range
    tree.insert(EntityId(1), wpos(50.0, 51.0));
    // id=2: 4 units away → in range
    tree.insert(EntityId(2), wpos(54.0, 50.0));
    // id=3: ~63 units away → out of range
    tree.insert(EntityId(3), wpos(95.0, 95.0));

    let aoi = AudioAoi::new(5.0, 32).expect("valid");
    let ids = aoi.audible_ids(&tree, listener_pos);

    assert!(ids.contains(&EntityId(1)), "id=1 should be in range");
    assert!(ids.contains(&EntityId(2)), "id=2 should be in range");
    assert!(
        !ids.contains(&EntityId(3)),
        "id=3 should be culled (out of range)"
    );
}

/// Empty world → empty result.
#[test]
fn audible_ids_empty_world() {
    let tree = world();
    let aoi = AudioAoi::new(25.0, 32).expect("valid");
    assert!(
        aoi.audible_ids(&tree, Vec3::new(10.0, 0.0, 10.0))
            .is_empty(),
        "no emitters → no ids"
    );
}

/// Emitter exactly at `radius` boundary: the quadtree uses an inclusive circle,
/// so it may or may not be included — but beyond radius it must be absent.
#[test]
fn audible_ids_emitter_well_beyond_radius_is_absent() {
    let mut tree = world();
    let listener_pos = Vec3::new(50.0, 0.0, 50.0);

    // id=1: 2× radius away
    tree.insert(EntityId(1), wpos(50.0, 70.0)); // 20 units; radius=8
                                                // id=2: inside
    tree.insert(EntityId(2), wpos(50.0, 54.0)); // 4 units

    let aoi = AudioAoi::new(8.0, 32).expect("valid");
    let ids = aoi.audible_ids(&tree, listener_pos);

    assert!(
        !ids.contains(&EntityId(1)),
        "id=1 is 20 units out, well beyond radius 8"
    );
    assert!(
        ids.contains(&EntityId(2)),
        "id=2 is 4 units in, within radius 8"
    );
}

// ── Voice selection and budget (`mix`) ───────────────────────────────────────

/// Emitters beyond `max_distance` are dropped; the rest are ordered nearest-first.
#[test]
fn mix_drops_out_of_range_emitters_and_orders_by_distance() {
    let listener = listener_at(Vec3::ZERO);
    let a = atten(); // max_distance = 10
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
        }, // beyond max_distance → dropped
    ];
    let aoi = AudioAoi::new(60.0, 32).expect("valid");
    let voices = aoi.mix(&listener, &a, &candidates);

    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![3, 7], "out-of-range dropped, nearest first");
    for v in &voices {
        assert!(
            v.mix.gain > 0.0,
            "every retained voice must have positive gain"
        );
    }
}

/// Nearest emitter must be louder than a farther one.
#[test]
fn mix_nearer_voice_is_louder() {
    let listener = listener_at(Vec3::ZERO);
    let a = atten();
    let candidates = [
        EmitterInput {
            id: EntityId(1),
            position: Vec3::new(0.0, 0.0, 2.0),
        },
        EmitterInput {
            id: EntityId(2),
            position: Vec3::new(0.0, 0.0, 8.0),
        },
    ];
    let aoi = AudioAoi::new(20.0, 32).expect("valid");
    let voices = aoi.mix(&listener, &a, &candidates);
    assert_eq!(voices.len(), 2);
    assert!(
        voices[0].mix.gain > voices[1].mix.gain,
        "near (d=2) gain {} should exceed far (d=8) gain {}",
        voices[0].mix.gain,
        voices[1].mix.gain
    );
}

/// `max_voices` caps the selection; the *nearest* N survive.
#[test]
fn mix_caps_voices_to_budget_keeping_nearest() {
    let listener = listener_at(Vec3::ZERO);
    let a = Attenuation::new(1.0, 100.0, Rolloff::Linear).expect("valid");
    let candidates: Vec<EmitterInput> = (1u64..=6)
        .map(|i| EmitterInput {
            id: EntityId(i),
            position: Vec3::new(0.0, 0.0, i as f32 * 5.0),
        })
        .collect();
    let aoi = AudioAoi::new(200.0, 3).expect("valid");
    let voices = aoi.mix(&listener, &a, &candidates);

    assert_eq!(voices.len(), 3, "capped at max_voices=3");
    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![1, 2, 3], "kept the three nearest emitters");
}

/// Distance ties break deterministically on entity id (lower id first).
#[test]
fn mix_distance_ties_break_on_id() {
    let listener = listener_at(Vec3::ZERO);
    let a = Attenuation::new(1.0, 50.0, Rolloff::Linear).expect("valid");
    // Two emitters at the same distance but mirrored.
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
    let voices = aoi.mix(&listener, &a, &candidates);
    let ids: Vec<u64> = voices.iter().map(|v| v.id.0).collect();
    assert_eq!(ids, vec![2, 8], "lower entity id wins the distance tie");
}

/// No candidates → empty voices (no panic).
#[test]
fn mix_empty_candidates_is_empty() {
    let listener = listener_at(Vec3::ZERO);
    let a = atten();
    let aoi = AudioAoi::default();
    assert!(aoi.mix(&listener, &a, &[]).is_empty());
}

/// A single emitter at min_distance is kept at full gain.
#[test]
fn mix_single_emitter_at_min_distance_is_full_gain() {
    let listener = listener_at(Vec3::ZERO);
    let a = Attenuation::new(2.0, 20.0, Rolloff::Linear).expect("valid");
    let candidates = [EmitterInput {
        id: EntityId(1),
        position: Vec3::new(0.0, 0.0, 2.0), // exactly at min_distance
    }];
    let aoi = AudioAoi::new(30.0, 8).expect("valid");
    let voices = aoi.mix(&listener, &a, &candidates);
    assert_eq!(voices.len(), 1);
    assert!(
        (voices[0].mix.gain - 1.0).abs() < 1e-4,
        "at min_distance gain should be 1.0, got {}",
        voices[0].mix.gain
    );
}
