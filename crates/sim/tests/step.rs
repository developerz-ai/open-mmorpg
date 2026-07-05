//! `World::step` — the fixed-schedule tick bound to an input batch.
//!
//! These exercise the determinism contract of [`omm_sim::schedule`]: intents
//! route to the right system, the schedule runs in order (movement commits
//! before casts resolve), dead actors are pruned, the tick advances by exactly
//! one, and identical inputs yield identical state.

use omm_ecs_core::{AbilityDef, AbilityId, EffectKind, EntityId, TargetKind, TargetShape, Team};
use omm_ecs_core::{Position, TICK_DT};
use omm_protocol::{CharacterId, Intent, Tick, Vec3};
use omm_sim::combat::Actor;
use omm_sim::{InputBatch, World};
use std::collections::BTreeMap;

fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 0.0, z }
}

fn actor(x: f32, z: f32, team: u16) -> Actor {
    Actor::new(at(x, z), Team(team), 100, 100)
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

/// A lethal enemy nuke — enough to drop a full-health target in one cast.
fn execute() -> AbilityDef {
    AbilityDef {
        id: AbilityId(2),
        power_cost: 10,
        cooldown_ticks: 30,
        gcd_ticks: 3,
        range: 10.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Single,
        effects: vec![EffectKind::Damage(1000)],
    }
}

fn book() -> BTreeMap<AbilityId, AbilityDef> {
    [(AbilityId(1), nuke()), (AbilityId(2), execute())]
        .into_iter()
        .collect()
}

fn move_intent(x: f32, z: f32) -> Intent {
    Intent::Move { dir: at(x, z) }
}

/// Name the target by the character id a client would send on the wire.
fn cast_intent(ability: u32, target: EntityId) -> Intent {
    Intent::UseAbility {
        id: ability,
        target: Some(CharacterId::new(target.raw())),
    }
}

#[test]
fn step_advances_tick_by_exactly_one() {
    let mut w = World::new();
    assert_eq!(w.now(), Tick(0));
    w.step(&Vec::new(), &book());
    assert_eq!(w.now(), Tick(1));
    w.step(&Vec::new(), &book());
    assert_eq!(w.now(), Tick(2));
}

#[test]
fn empty_batch_leaves_positions_but_advances_tick() {
    let mut w = World::new();
    let id = w.spawn(actor(5.0, 5.0, 1));
    w.step(&Vec::new(), &book());
    assert_eq!(w.get(id).unwrap().pos, Position(at(5.0, 5.0)));
    assert_eq!(w.now(), Tick(1));
}

#[test]
fn move_intent_integrates_one_tick() {
    let mut w = World::new();
    let id = w.spawn(actor(0.0, 0.0, 1));
    let batch: InputBatch = vec![(id, move_intent(3.0, -6.0))];
    w.step(&batch, &book());
    // Velocity 3 for one tick at TICK_DT: +3*dt on x, -6*dt on z.
    assert_eq!(
        w.get(id).unwrap().pos,
        Position(at(3.0 * TICK_DT, -6.0 * TICK_DT))
    );
}

#[test]
fn move_naming_absent_entity_is_a_noop() {
    let mut w = World::new();
    let batch: InputBatch = vec![(EntityId::new(999), move_intent(1.0, 0.0))];
    w.step(&batch, &book()); // must not panic
    assert!(w.is_empty());
    assert_eq!(w.now(), Tick(1));
}

#[test]
fn use_ability_intent_damages_target() {
    let mut w = World::new();
    let caster = w.spawn(actor(0.0, 0.0, 1));
    let target = w.spawn(actor(3.0, 0.0, 2));
    let batch: InputBatch = vec![(caster, cast_intent(1, target))];
    w.step(&batch, &book());
    assert_eq!(w.get(target).unwrap().health.current, 80);
    assert_eq!(w.get(caster).unwrap().power.current, 90);
}

#[test]
fn unknown_ability_id_is_dropped() {
    let mut w = World::new();
    let caster = w.spawn(actor(0.0, 0.0, 1));
    let target = w.spawn(actor(3.0, 0.0, 2));
    let batch: InputBatch = vec![(caster, cast_intent(404, target))];
    w.step(&batch, &book()); // unknown id → no-op, no panic
    assert_eq!(w.get(target).unwrap().health.current, 100);
    assert_eq!(w.now(), Tick(1));
}

#[test]
fn rejected_cast_changes_nothing_but_still_ticks() {
    let mut w = World::new();
    let caster = w.spawn(actor(0.0, 0.0, 1));
    // Out of range (range 10, distance 50): validation rejects the cast.
    let target = w.spawn(actor(50.0, 0.0, 2));
    let batch: InputBatch = vec![(caster, cast_intent(1, target))];
    w.step(&batch, &book());
    assert_eq!(w.get(target).unwrap().health.current, 100);
    assert_eq!(w.get(caster).unwrap().power.current, 100);
    assert_eq!(w.now(), Tick(1));
}

#[test]
fn prune_removes_actor_killed_this_tick() {
    let mut w = World::new();
    let caster = w.spawn(actor(0.0, 0.0, 1));
    let target = w.spawn(actor(3.0, 0.0, 2));
    let batch: InputBatch = vec![(caster, cast_intent(2, target))]; // execute = 1000 dmg
    w.step(&batch, &book());
    assert!(
        w.get(target).is_none(),
        "a zero-health actor must be pruned"
    );
    assert_eq!(w.len(), 1);
    assert!(w.get(caster).is_some());
}

#[test]
fn movement_commits_before_casts_resolve() {
    // Contract: Movement runs before ResolveCasts, so a cast sees this tick's
    // committed positions. The target starts in range and moves out within the
    // same tick; the cast must therefore miss.
    let mut w = World::new();
    let caster = w.spawn(actor(0.0, 0.0, 1));
    // Target starts in range (< 10); velocity 31 over one tick (~+1.03) carries
    // it past range 10 before the cast resolves.
    let target = w.spawn(actor(9.0, 0.0, 2));
    let batch: InputBatch = vec![
        (caster, cast_intent(1, target)),
        (target, move_intent(31.0, 0.0)),
    ];
    w.step(&batch, &book());
    assert!(
        w.get(target).unwrap().pos.0.x > 10.0,
        "target integrated out of range"
    );
    assert_eq!(
        w.get(target).unwrap().health.current,
        100,
        "cast resolves against the post-movement position, so it is out of range"
    );
}

#[test]
fn target_named_by_raw_id_resolves_to_the_entity() {
    // A client names its target by wire CharacterId; the scaffold bridges it to
    // the sim EntityId by raw value. Only the intended target takes damage.
    let mut w = World::new();
    let caster = w.spawn(actor(0.0, 0.0, 1));
    let bystander = w.spawn(actor(1.0, 0.0, 2));
    let target = w.spawn(actor(2.0, 0.0, 2));
    let batch: InputBatch = vec![(caster, cast_intent(1, target))];
    w.step(&batch, &book());
    assert_eq!(w.get(target).unwrap().health.current, 80);
    assert_eq!(w.get(bystander).unwrap().health.current, 100);
}

#[test]
fn identical_inputs_yield_identical_state() {
    let run = || {
        let mut w = World::new();
        // Attacker nukes each tick (only the first lands; the rest hit cooldown);
        // the victim drifts a varying amount. Both are pure functions of inputs.
        let attacker = w.spawn(actor(0.0, 0.0, 1)); // id 1, Team 1
        let victim = w.spawn(actor(3.0, 0.0, 2)); // id 2, Team 2
        for &vx in &[0.0f32, 1.0, 2.0, 3.0, 2.0, 1.0, 0.0, 4.0] {
            let batch: InputBatch = vec![
                (attacker, cast_intent(1, victim)),
                (victim, move_intent(vx, 0.0)),
            ];
            w.step(&batch, &book());
        }
        (
            w.now(),
            w.len(),
            w.get(attacker).map(|a| (a.pos, a.power.current)),
            w.get(victim).map(|a| (a.pos, a.health.current)),
        )
    };
    assert_eq!(run(), run());
}
