//! Input-log capture and replay-from-genesis, driven only through the public
//! [`omm_sim`] API — the way a shard records a session and a verifier re-runs it.
//!
//! The contract under test: fold a recorded [`omm_sim::InputLog`] over the
//! genesis world and reconstruct the authoritative state bit-for-bit, so a
//! trusted box can re-simulate a suspect's inputs and compare one
//! [`omm_sim::WorldHash`] per tick instead of the whole world.

use omm_ecs_core::{
    AbilityDef, AbilityId, AuraSpec, EffectKind, EntityId, Periodic, TargetKind, TargetShape, Team,
};
use omm_protocol::{CharacterId, Intent, Tick, Vec3};
use omm_sim::combat::Actor;
use omm_sim::{replay, InputBatch, InputLog, World, WorldHash};
use std::collections::BTreeMap;

fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 0.0, z }
}

fn cast_on(id: u32, target: EntityId) -> Intent {
    Intent::UseAbility {
        id,
        target: Some(CharacterId::new(target.raw())),
    }
}

/// A single-target nuke plus a periodic-damage DoT, so a session exercises both
/// instant effects and auras that keep ticking through idle ticks.
fn abilities() -> BTreeMap<AbilityId, AbilityDef> {
    let base = AbilityDef {
        id: AbilityId(1),
        power_cost: 0,
        cooldown_ticks: 0,
        gcd_ticks: 0,
        range: 999.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Single,
        effects: vec![EffectKind::Damage(12)],
    };
    let dot = AbilityDef {
        id: AbilityId(2),
        effects: vec![EffectKind::ApplyAura(AuraSpec {
            period_ticks: 1,
            duration_ticks: 6,
            periodic: Periodic::Damage(4),
        })],
        ..base.clone()
    };
    [base, dot].into_iter().map(|d| (d.id, d)).collect()
}

/// Run a scripted session: two actors, a mixed intent stream, several trailing
/// idle ticks while the DoT is still ticking. Returns genesis, the recorded log,
/// the per-tick hash trajectory, and the live final world.
fn session() -> (World, InputLog, Vec<WorldHash>, World) {
    let table = abilities();
    let mut world = World::new();
    let caster = world.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let target = world.spawn(Actor::new(at(2.0, 0.0), Team(2), 100, 100));
    let genesis = world.clone();

    // Per tick: the batch the server applies, in server-issued id order.
    let program: Vec<InputBatch> = vec![
        vec![(caster, cast_on(2, target))], // apply the DoT
        vec![(caster, Intent::Move { dir: at(1.0, 0.0) })],
        vec![(caster, cast_on(1, target))], // nuke
        vec![],                             // idle — DoT keeps ticking
        vec![],                             // idle
        vec![],                             // idle
    ];

    let mut log = InputLog::new();
    let mut trajectory = Vec::new();
    for batch in &program {
        let tick = world.now();
        log.record_batch(tick, batch); // records idle ticks too (empty batch)
        world.step(batch, &table);
        trajectory.push(world.state_hash());
    }
    (genesis, log, trajectory, world)
}

#[test]
fn replay_reconstructs_the_authoritative_final_state() {
    let (genesis, log, _, live) = session();
    let out = replay(&genesis, &log, &abilities());
    assert_eq!(out.state_hash(), live.state_hash(), "final hash must match");
    assert_eq!(out.now(), live.now(), "replay reaches the same tick");
}

#[test]
fn replay_reproduces_the_whole_hash_trajectory() {
    // Re-run tick by tick and check the fingerprint at every tick, the way an
    // anti-cheat verifier compares a stream of authoritative hashes.
    let (genesis, log, trajectory, _) = session();
    let table = abilities();
    let mut world = genesis.clone();
    let mut replayed = Vec::new();
    let last = log.last_tick().expect("session recorded at least one tick");
    while world.now() <= last {
        let tick = world.now();
        let batch: InputBatch = log
            .entries()
            .iter()
            .filter(|(t, _, _)| *t == tick)
            .map(|(_, id, intent)| (*id, intent.clone()))
            .collect();
        world.step(&batch, &table);
        replayed.push(world.state_hash());
    }
    assert_eq!(replayed, trajectory, "hash must match at every tick");
}

#[test]
fn trailing_idle_ticks_are_replayed() {
    // The DoT applied early keeps ticking during the trailing idle ticks; replay
    // must step through them, not stop at the last intent.
    let (genesis, log, _, live) = session();
    let target = live
        .get(EntityId::new(2))
        .expect("target survived the session");
    assert!(
        target.health.current < 100,
        "the DoT ticked over idle ticks"
    );
    let out = replay(&genesis, &log, &abilities());
    assert_eq!(out.state_hash(), live.state_hash());
}

#[test]
fn a_log_survives_json_persistence() {
    // Persist the session, reload it, and confirm the reloaded log replays to the
    // same authoritative state — the durable-capture path.
    let (genesis, log, _, live) = session();
    let json = serde_json::to_string(&log).expect("serialize");
    let restored: InputLog = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(log, restored);
    let out = replay(&genesis, &restored, &abilities());
    assert_eq!(out.state_hash(), live.state_hash());
}

#[test]
fn a_tampered_input_log_diverges() {
    // Swap one recorded intent for a stronger cast: the re-simulated hash no
    // longer matches the authoritative one — exactly how re-sim catches a client
    // that asserted a state the server never produced.
    let (genesis, log, _, live) = session();
    let honest = replay(&genesis, &log, &abilities());
    assert_eq!(honest.state_hash(), live.state_hash());

    // Rebuild the same session but tamper with the opening cast's target so the
    // DoT lands on no one, changing the trajectory.
    let table = abilities();
    let mut world = World::new();
    let caster = world.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    world.spawn(Actor::new(at(2.0, 0.0), Team(2), 100, 100));
    let g2 = world.clone();
    let mut tampered = InputLog::new();
    tampered.record(Tick(0), caster, cast_on(2, EntityId::new(999))); // absent target
    let out = replay(&g2, &tampered, &table);
    assert_ne!(
        out.state_hash(),
        live.state_hash(),
        "a doctored input log cannot reproduce the authoritative hash",
    );
}
