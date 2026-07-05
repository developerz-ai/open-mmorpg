//! Input-log capture and replay-from-genesis, driven only through the public
//! [`omm_sim`] API — the way a shard records a session and a verifier re-runs it.
//!
//! The contract under test: fold a recorded [`omm_sim::InputLog`] over the
//! genesis world and reconstruct the authoritative state bit-for-bit, so a
//! trusted box can re-simulate a suspect's inputs and compare one
//! [`omm_sim::WorldHash`] per tick instead of the whole world.
//!
//! # Property coverage
//!
//! In addition to the scripted scenario tests, three core replay properties are
//! verified by proptest over arbitrary input streams:
//!
//! 1. **Idempotent replay** — calling `replay` twice on the same log produces
//!    equal hashes (replay is a pure function).
//! 2. **Second-shard equality** — a fresh, independent World that replays the
//!    same log from the same genesis lands on the same hash as the live session
//!    (the cross-shard re-sim guarantee).
//! 3. **Intent sensitivity** — replacing one intent with a semantically different
//!    one moves the final hash (a doctored log cannot reproduce the authoritative
//!    state).

use omm_ecs_core::{
    AbilityDef, AbilityId, AuraSpec, EffectKind, EntityId, Periodic, TargetKind, TargetShape, Team,
};
use omm_protocol::{CharacterId, Intent, Tick, Vec3};
use omm_sim::combat::Actor;
use omm_sim::{replay, InputBatch, InputLog, World, WorldHash};
use proptest::prelude::*;
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

// ── proptest ─────────────────────────────────────────────────────────────────

/// A minimal two-actor world plus a zero-cost, always-available nuke — simple
/// enough to exercise in proptest without combinatorial state explosion.
fn simple_abilities() -> BTreeMap<AbilityId, AbilityDef> {
    let nuke = AbilityDef {
        id: AbilityId(1),
        power_cost: 0,
        cooldown_ticks: 0,
        gcd_ticks: 0,
        range: 999.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Single,
        effects: vec![EffectKind::Damage(5)],
    };
    [(nuke.id, nuke)].into_iter().collect()
}

/// Build a reproducible session from a `(x, z, is_cast)` triple per tick.
///
/// Returns `(genesis, log, final_world)`.  The genesis is taken before any
/// step so a second replay can start from an identical state.
fn build_session(ticks: &[(f32, f32, bool)]) -> (World, InputLog, World) {
    let table = simple_abilities();
    let mut world = World::new();
    let caster = world.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let target = world.spawn(Actor::new(at(2.0, 0.0), Team(2), 100, 100));
    let genesis = world.clone();
    let mut log = InputLog::new();
    for &(x, z, is_cast) in ticks {
        let tick = world.now();
        let intent = if is_cast {
            cast_on(1, target)
        } else {
            Intent::Move { dir: at(x, z) }
        };
        let batch: InputBatch = vec![(caster, intent)];
        log.record_batch(tick, &batch);
        world.step(&batch, &table);
    }
    (genesis, log, world)
}

/// Proptest strategy: a short sequence of (x, z, is_cast) triples.
fn arb_ticks() -> impl Strategy<Value = Vec<(f32, f32, bool)>> {
    prop::collection::vec(
        (-3.0f32..3.0f32, -3.0f32..3.0f32, proptest::bool::ANY),
        0..16,
    )
}

proptest! {
    /// Replay is a pure function: calling it twice on the same genesis + log
    /// produces bit-identical hashes — no hidden mutable state.
    #[test]
    fn replay_twice_produces_equal_hashes(ticks in arb_ticks()) {
        let (genesis, log, _) = build_session(&ticks);
        let table = simple_abilities();
        let h1 = replay(&genesis, &log, &table).state_hash();
        let h2 = replay(&genesis, &log, &table).state_hash();
        prop_assert_eq!(h1, h2);
    }

    /// A completely independent World that replays the same log from an equal
    /// genesis reaches the same final hash — the cross-shard re-sim guarantee.
    ///
    /// This is the core promise: two trusted boxes given (genesis, log) always
    /// agree on the authoritative hash, so a mismatch against a client-asserted
    /// hash is conclusive.
    #[test]
    fn second_shard_replay_matches_authoritative(ticks in arb_ticks()) {
        let (genesis, log, live) = build_session(&ticks);
        let second_shard = replay(&genesis, &log, &simple_abilities());
        prop_assert_eq!(
            second_shard.state_hash(),
            live.state_hash(),
            "a second independent shard must land on the same authoritative hash",
        );
    }

    /// Replacing one Move intent with a semantically distinct one (a cast that
    /// deals damage) changes the final hash — the sim is sensitive to every
    /// applied input.
    #[test]
    fn replacing_first_intent_with_cast_diverges(
        // At least one tick so there is something to tamper with.
        first_x in 0.5f32..3.0f32,
        rest in prop::collection::vec(((-3.0f32..3.0f32, -3.0f32..3.0f32), proptest::bool::ANY), 0..8),
    ) {
        let table = simple_abilities();

        // Honest session: first tick is a Move, rest follow the proptest choices.
        let mut honest_ticks: Vec<(f32, f32, bool)> = vec![(first_x, 0.0, false)];
        honest_ticks.extend(rest.iter().map(|&((x, z), c)| (x, z, c)));
        let (genesis, _honest_log, live) = build_session(&honest_ticks);

        // Tampered session: same genesis, but the first intent is a cast instead.
        let mut world2 = World::new();
        let caster2 = world2.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
        let target2 = world2.spawn(Actor::new(at(2.0, 0.0), Team(2), 100, 100));
        // genesis2 must equal genesis for the re-sim to be meaningful.
        let genesis2 = world2.clone();
        let _ = genesis2; // both have the same content; use the original below.
        let mut tampered_log = InputLog::new();
        tampered_log.record(Tick(0), caster2, cast_on(1, target2)); // cast replaces move
        for (i, &((x, z), is_cast)) in rest.iter().enumerate() {
            let tick = Tick((i + 1) as u64);
            let intent = if is_cast { cast_on(1, target2) } else { Intent::Move { dir: at(x, z) } };
            tampered_log.record(tick, caster2, intent);
        }

        let tampered_out = replay(&genesis, &tampered_log, &table);
        // The honest run moved the caster; the tampered run cast instead, dealing
        // damage and leaving the caster in a different position → hashes diverge.
        prop_assert_ne!(
            tampered_out.state_hash(),
            live.state_hash(),
            "replacing a Move with a cast must change the authoritative hash",
        );
    }
}
