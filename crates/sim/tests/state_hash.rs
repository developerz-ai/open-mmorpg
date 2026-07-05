//! `World::state_hash` — the deterministic, platform-stable [`omm_sim::WorldHash`]
//! that replay and anti-cheat re-simulation compare each tick.
//!
//! These drive only the public API, the way a shard would: spawn, step, hash,
//! and assert two runs of the same inputs stay bit-identical while any real
//! state change moves the fingerprint.

use omm_ecs_core::{AbilityDef, AbilityId, EffectKind, EntityId, TargetKind, TargetShape, Team};
use omm_protocol::{CharacterId, Intent, Tick, Vec3};
use omm_sim::combat::Actor;
use omm_sim::{InputBatch, World, WorldHash};
use std::collections::BTreeMap;

fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 0.0, z }
}

/// A single-target nuke: 20 damage, range 10, on enemies.
fn nuke() -> AbilityDef {
    AbilityDef {
        id: AbilityId(1),
        power_cost: 10,
        cooldown_ticks: 30,
        gcd_ticks: 3,
        range: 10.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Single,
        effects: vec![EffectKind::Damage(20)],
    }
}

fn abilities() -> BTreeMap<AbilityId, AbilityDef> {
    let mut m = BTreeMap::new();
    let n = nuke();
    m.insert(n.id, n);
    m
}

/// Spawn two actors, drive a mixed input stream for several ticks, and collect a
/// hash after every tick. Returns the whole trajectory so runs can be compared
/// tick-for-tick.
fn trajectory() -> Vec<WorldHash> {
    let table = abilities();
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let target = w.spawn(Actor::new(at(3.0, 0.0), Team(2), 100, 100));
    let mut hashes = Vec::new();
    for tick in 0..6u32 {
        let move_dir = at(1.0, tick as f32);
        let inputs: InputBatch = vec![
            (caster, Intent::Move { dir: move_dir }),
            (
                caster,
                Intent::UseAbility {
                    id: 1,
                    target: Some(CharacterId::new(target.raw())),
                },
            ),
        ];
        w.step(&inputs, &table);
        hashes.push(w.state_hash());
    }
    hashes
}

#[test]
fn replay_reproduces_the_same_hash_trajectory() {
    // The anti-cheat oracle: identical inputs → identical hash at every tick.
    assert_eq!(trajectory(), trajectory());
}

#[test]
fn hash_advances_as_the_world_changes() {
    let hashes = trajectory();
    // Movement plus a landing cast changes state each tick, so consecutive hashes
    // differ — a stalled hash would mean the tick did nothing.
    for pair in hashes.windows(2) {
        assert_ne!(pair[0], pair[1], "each active tick must move the hash");
    }
}

#[test]
fn a_diverging_input_breaks_the_hash() {
    let table = abilities();
    let baseline = {
        let mut w = World::new();
        let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
        w.step(&[(id, Intent::Move { dir: at(1.0, 0.0) })], &table);
        w.state_hash()
    };
    let tampered = {
        let mut w = World::new();
        let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
        // One unit further — a client claiming a position the server didn't grant.
        w.step(&[(id, Intent::Move { dir: at(2.0, 0.0) })], &table);
        w.state_hash()
    };
    assert_ne!(baseline, tampered);
}

#[test]
fn hash_is_display_hex() {
    let h = WorldHash(0x0000_0000_0000_00ff);
    assert_eq!(h.to_string(), "00000000000000ff");
    assert_eq!(h.raw(), 0xff);
}

#[test]
fn absent_entity_id_does_not_shift_the_hash() {
    // A `Move` naming an entity the world never spawned is a no-op, so the hash
    // is exactly the untouched world's — the server ignores what it never issued.
    let mut w = World::new();
    w.spawn(Actor::new(at(4.0, 4.0), Team(1), 100, 100));
    let before = w.state_hash();
    let ghost = EntityId::new(9999);
    w.step(
        &[(ghost, Intent::Move { dir: at(5.0, 5.0) })],
        &BTreeMap::new(),
    );
    assert_eq!(w.state_hash(), before);
    assert_eq!(w.now(), Tick(1));
}
