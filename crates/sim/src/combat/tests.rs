//! Combat resolution tests: validation gates, effect application, aura ticking,
//! threat, and determinism.

use super::*;
use crate::World;
use omm_ecs_core::{AbilityId, AuraSpec};
use proptest::prelude::*;

fn at(x: f32, z: f32) -> Vec3 {
    Vec3 { x, y: 0.0, z }
}

/// A single-target nuke: 20 damage, 30-tick cooldown, 3-tick GCD, range 10.
fn nuke() -> AbilityDef {
    AbilityDef {
        id: AbilityId(1),
        power_cost: 30,
        cooldown_ticks: 30,
        gcd_ticks: 3,
        range: 10.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Single,
        effects: vec![EffectKind::Damage(20)],
    }
}

fn two_actor_world() -> (World, EntityId, EntityId) {
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let target = w.spawn(Actor::new(at(3.0, 0.0), Team(2), 100, 100));
    (w, caster, target)
}

#[test]
fn nuke_damages_target_and_pays_costs() {
    let (mut w, caster, target) = two_actor_world();
    w.cast(Tick(0), caster, &nuke(), Some(target)).unwrap();
    assert_eq!(w.get(target).unwrap().health.current, 80);
    assert_eq!(w.get(caster).unwrap().power.current, 70);
    // Damage generated threat from the caster on the target's table.
    assert_eq!(w.get(target).unwrap().threat.of(caster), 20);
}

#[test]
fn rejects_when_on_cooldown() {
    let (mut w, caster, target) = two_actor_world();
    w.cast(Tick(0), caster, &nuke(), Some(target)).unwrap();
    let err = w.cast(Tick(5), caster, &nuke(), Some(target)).unwrap_err();
    assert_eq!(err, RejectReason::OnCooldown);
    // Nothing changed on the second, rejected cast.
    assert_eq!(w.get(target).unwrap().health.current, 80);
}

#[test]
fn rejects_out_of_range() {
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let target = w.spawn(Actor::new(at(50.0, 0.0), Team(2), 100, 100));
    assert_eq!(
        w.cast(Tick(0), caster, &nuke(), Some(target)),
        Err(RejectReason::OutOfRange)
    );
}

#[test]
fn rejects_not_enough_power() {
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 10));
    let target = w.spawn(Actor::new(at(1.0, 0.0), Team(2), 100, 100));
    assert_eq!(
        w.cast(Tick(0), caster, &nuke(), Some(target)),
        Err(RejectReason::NotEnoughPower)
    );
}

#[test]
fn rejects_friendly_fire_on_enemy_ability() {
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let ally = w.spawn(Actor::new(at(1.0, 0.0), Team(1), 100, 100));
    assert_eq!(
        w.cast(Tick(0), caster, &nuke(), Some(ally)),
        Err(RejectReason::InvalidTarget)
    );
}

#[test]
fn radius_ability_hits_all_enemies_in_range() {
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let near_a = w.spawn(Actor::new(at(1.0, 0.0), Team(2), 100, 100));
    let near_b = w.spawn(Actor::new(at(2.0, 0.0), Team(2), 100, 100));
    let far = w.spawn(Actor::new(at(40.0, 0.0), Team(2), 100, 100));
    let aoe = AbilityDef {
        id: AbilityId(2),
        power_cost: 0,
        cooldown_ticks: 0,
        gcd_ticks: 0,
        range: 100.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Radius(5.0),
        effects: vec![EffectKind::Damage(10)],
    };
    w.cast(Tick(0), caster, &aoe, Some(near_a)).unwrap();
    assert_eq!(w.get(near_a).unwrap().health.current, 90);
    assert_eq!(w.get(near_b).unwrap().health.current, 90);
    assert_eq!(w.get(far).unwrap().health.current, 100); // out of radius
}

#[test]
fn dot_aura_ticks_on_period_and_expires() {
    let mut w = World::new();
    let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
    let target = w.spawn(Actor::new(at(1.0, 0.0), Team(2), 100, 100));
    let dot = AbilityDef {
        id: AbilityId(3),
        power_cost: 0,
        cooldown_ticks: 0,
        gcd_ticks: 0,
        range: 10.0,
        target_kind: TargetKind::Enemy,
        shape: TargetShape::Single,
        effects: vec![EffectKind::ApplyAura(AuraSpec {
            period_ticks: 2,
            duration_ticks: 6,
            periodic: Periodic::Damage(5),
        })],
    };
    w.cast(Tick(0), caster, &dot, Some(target)).unwrap();
    // Ticks fire at 2, 4, 6 — but 6 == expire, so only 2 and 4 land (< expire).
    for t in 1..=8 {
        w.tick_auras(Tick(t));
    }
    assert_eq!(w.get(target).unwrap().health.current, 90);
    assert!(w.get(target).unwrap().auras.active.is_empty());
    // Threat accrued from the DoT source.
    assert_eq!(w.get(target).unwrap().threat.of(caster), 10);
}

proptest! {
    /// Determinism: the same ordered cast stream yields identical final health.
    #[test]
    fn casts_are_deterministic(seq in prop::collection::vec(0u64..5, 0..40)) {
        let build = || {
            let (mut w, caster, target) = two_actor_world();
            for &t in &seq {
                // Ignore rejections (cooldown/power) — they must be identical too.
                let _ = w.cast(Tick(t), caster, &nuke(), Some(target));
                w.tick_auras(Tick(t));
            }
            (w, caster, target)
        };
        let (a, a_caster, a_target) = build();
        let (b, b_caster, b_target) = build();
        // Server-issued ids are deterministic across identical spawn order.
        prop_assert_eq!(a_caster, b_caster);
        prop_assert_eq!(a_target, b_target);
        prop_assert_eq!(
            a.get(a_target).unwrap().health.current,
            b.get(b_target).unwrap().health.current
        );
        prop_assert_eq!(
            a.get(a_caster).unwrap().power.current,
            b.get(b_caster).unwrap().power.current
        );
    }
}
