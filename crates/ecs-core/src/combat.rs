//! Combat data model — the compiled surface the resolution engine dispatches on.
//!
//! Mirrors TrinityCore's `SpellInfo` (immutable definition) vs `Spell` (live
//! cast), ported to ECS and made deterministic. [`AbilityDef`] + [`EffectKind`]
//! are the definition (authored as `content-schema` data, loaded onto this
//! runtime shape — adding an ability is data, no recompile). Effect *kinds* are
//! the compiled escape hatch: a new kind is a new variant + a system. Live state
//! is the components below ([`Cooldowns`], [`Auras`], [`Threat`], [`Power`]);
//! resolution lives in `omm-sim`. See docs/specs/game-server/combat/README.md.
//!
//! Pure: no I/O, no wall-clock, no RNG. Ordered [`BTreeMap`]s keep iteration
//! deterministic so replay and anti-cheat re-sim line up bit-for-bit.

use omm_protocol::Tick;
use std::collections::BTreeMap;

/// A simulation entity handle. The shared identity across position, health,
/// cooldowns and threat. Server-issued; never trusted from a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Wrap a raw value.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying raw value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Stable id of an ability definition. Mirrors the `id` a client names in
/// [`omm_protocol::Intent::UseAbility`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AbilityId(pub u32);

/// A spendable resource pool (mana/energy/rage). `current` is clamped to `max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Power {
    pub current: u32,
    pub max: u32,
}

impl Power {
    /// A full pool.
    #[must_use]
    pub const fn full(max: u32) -> Self {
        Self { current: max, max }
    }

    /// Whether `cost` can be paid right now.
    #[must_use]
    pub const fn can_pay(&self, cost: u32) -> bool {
        self.current >= cost
    }

    /// Spend `cost`, saturating at 0. Returns whether it was affordable.
    pub fn spend(&mut self, cost: u32) -> bool {
        if self.can_pay(cost) {
            self.current -= cost;
            true
        } else {
            false
        }
    }

    /// Regenerate `amount`, never exceeding `max`.
    pub fn regen(&mut self, amount: u32) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }
}

/// Who an ability may legally target, validated before resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// The caster only — no target entity required.
    SelfOnly,
    /// A hostile entity (different [`Team`]).
    Enemy,
    /// A friendly entity (same [`Team`]).
    Ally,
}

/// The geometry an ability's effects apply over, resolved against the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetShape {
    /// A single primary target.
    Single,
    /// Every valid entity within `radius` of the primary target (or caster for
    /// [`TargetKind::SelfOnly`]).
    Radius(f32),
}

/// One immediate effect an ability applies. New kinds are compiled systems —
/// this enum is the dispatch key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectKind {
    /// Direct damage.
    Damage(u32),
    /// Direct heal.
    Heal(u32),
    /// Apply a periodic aura (DoT/HoT).
    ApplyAura(AuraSpec),
}

/// A periodic effect applied each `period_ticks` for `duration_ticks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuraSpec {
    /// Ticks between applications (must be ≥ 1; 0 is treated as 1).
    pub period_ticks: u32,
    /// Total lifetime in ticks.
    pub duration_ticks: u32,
    /// What each tick does.
    pub periodic: Periodic,
}

/// The per-tick payload of an aura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Periodic {
    /// Damage over time.
    Damage(u32),
    /// Heal over time.
    Heal(u32),
}

/// An immutable ability definition — the runtime shape the engine resolves.
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityDef {
    pub id: AbilityId,
    /// Resource cost, paid atomically with the cast.
    pub power_cost: u32,
    /// Per-ability cooldown in ticks.
    pub cooldown_ticks: u32,
    /// Global cooldown triggered on use, in ticks.
    pub gcd_ticks: u32,
    /// Max distance from caster to primary target (world units).
    pub range: f32,
    /// Legal target relationship.
    pub target_kind: TargetKind,
    /// Effect geometry.
    pub shape: TargetShape,
    /// Effects applied to each selected target, in order.
    pub effects: Vec<EffectKind>,
}

/// Team/faction membership for target validation. Same team = ally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Team(pub u16);

/// Per-entity cooldown bookkeeping: per-ability ready ticks plus the shared GCD.
#[derive(Debug, Clone, Default)]
pub struct Cooldowns {
    ready_at: BTreeMap<AbilityId, Tick>,
    gcd_until: Tick,
}

impl Cooldowns {
    /// Whether `ability` is off cooldown *and* the GCD has elapsed at `now`.
    #[must_use]
    pub fn is_ready(&self, ability: AbilityId, now: Tick) -> bool {
        now >= self.gcd_until && self.ready_at.get(&ability).is_none_or(|&t| now >= t)
    }

    /// Arm the ability cooldown and the GCD after a successful cast.
    pub fn trigger(&mut self, ability: AbilityId, now: Tick, cooldown_ticks: u32, gcd_ticks: u32) {
        self.ready_at.insert(
            ability,
            Tick(now.0.saturating_add(u64::from(cooldown_ticks))),
        );
        self.gcd_until = Tick(now.0.saturating_add(u64::from(gcd_ticks)));
    }

    /// The shared global-cooldown deadline — the tick the GCD elapses. Exposed
    /// for deterministic state hashing and inspection; the fields stay private
    /// so only [`trigger`](Self::trigger) can arm a cooldown.
    #[must_use]
    pub fn gcd_until(&self) -> Tick {
        self.gcd_until
    }

    /// The armed per-ability ready ticks in deterministic ability-id order — the
    /// ordered `BTreeMap`, never insertion order — so a state hash over them is
    /// stable across runs and machines.
    pub fn ready_ticks(&self) -> impl Iterator<Item = (AbilityId, Tick)> + '_ {
        self.ready_at.iter().map(|(&id, &tick)| (id, tick))
    }
}

/// A live aura application on a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveAura {
    /// The entity that applied it (for threat attribution).
    pub source: EntityId,
    /// The aura's periodic behavior.
    pub spec: AuraSpec,
    /// The next tick this aura fires on.
    pub next_tick: Tick,
    /// The tick at (or after) which the aura expires.
    pub expire_tick: Tick,
}

/// The set of auras currently on an entity.
#[derive(Debug, Clone, Default)]
pub struct Auras {
    pub active: Vec<ActiveAura>,
}

impl Auras {
    /// Apply a new aura, scheduling its first tick.
    pub fn apply(&mut self, source: EntityId, spec: AuraSpec, now: Tick) {
        let period = u64::from(spec.period_ticks.max(1));
        self.active.push(ActiveAura {
            source,
            spec,
            next_tick: Tick(now.0.saturating_add(period)),
            expire_tick: Tick(now.0.saturating_add(u64::from(spec.duration_ticks))),
        });
    }

    /// Drop expired auras as of `now`.
    pub fn prune(&mut self, now: Tick) {
        self.active.retain(|a| now < a.expire_tick);
    }
}

/// A threat table: how much aggro each entity has generated against this one.
/// Ordered so the top-threat lookup is deterministic on ties (lowest id wins).
#[derive(Debug, Clone, Default)]
pub struct Threat {
    table: BTreeMap<EntityId, i64>,
}

impl Threat {
    /// Add `amount` of threat from `who`.
    pub fn add(&mut self, who: EntityId, amount: i64) {
        let slot = self.table.entry(who).or_insert(0);
        *slot = slot.saturating_add(amount);
    }

    /// Current threat from `who`.
    #[must_use]
    pub fn of(&self, who: EntityId) -> i64 {
        self.table.get(&who).copied().unwrap_or(0)
    }

    /// The highest-threat entity, ties broken by lowest [`EntityId`].
    #[must_use]
    pub fn top(&self) -> Option<EntityId> {
        self.table
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(&who, _)| who)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_spends_and_regens_within_bounds() {
        let mut p = Power::full(100);
        assert!(p.spend(40));
        assert_eq!(p.current, 60);
        assert!(!p.spend(1000));
        assert_eq!(p.current, 60);
        p.regen(1000);
        assert_eq!(p.current, 100);
    }

    #[test]
    fn cooldown_and_gcd_gate_readiness() {
        let mut cds = Cooldowns::default();
        let a = AbilityId(7);
        assert!(cds.is_ready(a, Tick(0)));
        cds.trigger(a, Tick(0), 30, 3);
        assert!(!cds.is_ready(a, Tick(2)), "GCD not elapsed");
        assert!(!cds.is_ready(a, Tick(10)), "ability still on cooldown");
        assert!(cds.is_ready(a, Tick(30)), "cooldown elapsed");
    }

    #[test]
    fn cooldown_summary_is_id_ordered_for_hashing() {
        let mut cds = Cooldowns::default();
        // Arm out of id order; the summary must still come back id-sorted.
        cds.trigger(AbilityId(9), Tick(0), 30, 3);
        cds.trigger(AbilityId(2), Tick(0), 15, 3);
        assert_eq!(cds.gcd_until(), Tick(3));
        let armed: Vec<_> = cds.ready_ticks().collect();
        assert_eq!(
            armed,
            vec![(AbilityId(2), Tick(15)), (AbilityId(9), Tick(30))]
        );
    }

    #[test]
    fn auras_schedule_and_expire() {
        let mut auras = Auras::default();
        let spec = AuraSpec {
            period_ticks: 3,
            duration_ticks: 9,
            periodic: Periodic::Damage(5),
        };
        auras.apply(EntityId::new(1), spec, Tick(0));
        assert_eq!(auras.active[0].next_tick, Tick(3));
        assert_eq!(auras.active[0].expire_tick, Tick(9));
        auras.prune(Tick(9));
        assert!(auras.active.is_empty());
    }

    #[test]
    fn threat_tracks_top_with_deterministic_tiebreak() {
        let mut t = Threat::default();
        let (a, b) = (EntityId::new(2), EntityId::new(5));
        t.add(a, 100);
        t.add(b, 100);
        // Tie → lowest id wins.
        assert_eq!(t.top(), Some(a));
        t.add(b, 1);
        assert_eq!(t.top(), Some(b));
        assert_eq!(t.of(a), 100);
    }
}
