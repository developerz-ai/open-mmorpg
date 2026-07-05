//! Deterministic, server-authoritative combat resolution.
//!
//! Turns a validated `UseAbility` intent into authoritative state change. The
//! engine *resolves*; content *defines* what resolves ([`AbilityDef`]). Same
//! inputs → same outcome on any box — the basis for replay and anti-cheat
//! re-sim. Pure: no I/O, no wall-clock, no RNG.
//!
//! The cast pipeline (docs/specs/game-server/combat/README.md):
//! validate → select targets → take costs → resolve effects → apply + threat.
//! Any economic side effect (loot/consume) is *not* done here — that goes
//! through an `omm-persistence` transaction on the host side.

use crate::World;
use omm_ecs_core::{
    AbilityDef, Auras, Cooldowns, EffectKind, EntityId, Health, Periodic, Position, Power,
    TargetKind, TargetShape, Team, Threat, Velocity,
};
use omm_errors::ClientCode;
use omm_protocol::{Tick, Vec3};

/// One combat participant. A superset of the toy [`crate::EntityState`] with the
/// live combat components attached.
#[derive(Debug, Clone)]
pub struct Actor {
    pub pos: Position,
    pub vel: Velocity,
    pub health: Health,
    pub power: Power,
    pub team: Team,
    pub cooldowns: Cooldowns,
    pub auras: Auras,
    pub threat: Threat,
}

impl Actor {
    /// A full-health actor at `pos` on `team` with the given resource pool.
    #[must_use]
    pub fn new(pos: Vec3, team: Team, health_max: u32, power_max: u32) -> Self {
        Self {
            pos: Position(pos),
            vel: Velocity(Vec3::default()),
            health: Health::full(health_max),
            power: Power::full(power_max),
            team,
            cooldowns: Cooldowns::default(),
            auras: Auras::default(),
            threat: Threat::default(),
        }
    }
}

/// Why a cast was rejected. Maps to a stable, wire-safe [`ClientCode`]; the
/// server answers a rejected cast, it never crashes on one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    NoSuchCaster,
    CasterDead,
    NoSuchTarget,
    OnCooldown,
    NotEnoughPower,
    OutOfRange,
    InvalidTarget,
}

impl RejectReason {
    /// The stable client code for this rejection.
    #[must_use]
    pub const fn code(self) -> ClientCode {
        match self {
            Self::NoSuchCaster | Self::NoSuchTarget => ClientCode::NotFound,
            Self::CasterDead | Self::OnCooldown | Self::NotEnoughPower | Self::OutOfRange => {
                ClientCode::Conflict
            }
            Self::InvalidTarget => ClientCode::BadRequest,
        }
    }
}

/// Squared distance — squared to avoid a platform-variant `sqrt` on the
/// authoritative path.
#[must_use]
fn dist_sq(a: Vec3, b: Vec3) -> f32 {
    let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);
    dx * dx + dy * dy + dz * dz
}

/// Combat resolution hung off the authoritative [`World`]: casts, effects, and
/// aura ticking. Kept in its own module so the entity store and the combat
/// rules each stay under one reason to change.
impl World {
    /// Validate whether `caster` may legally target `target` with `def` right
    /// now — the guard the tick loop runs before mutating anything.
    fn validate(
        &self,
        now: Tick,
        caster: EntityId,
        def: &AbilityDef,
        target: Option<EntityId>,
    ) -> Result<EntityId, RejectReason> {
        let c = self.actors.get(&caster).ok_or(RejectReason::NoSuchCaster)?;
        if c.health.is_dead() {
            return Err(RejectReason::CasterDead);
        }
        if !c.cooldowns.is_ready(def.id, now) {
            return Err(RejectReason::OnCooldown);
        }
        if !c.power.can_pay(def.power_cost) {
            return Err(RejectReason::NotEnoughPower);
        }
        // SelfOnly resolves against the caster; everything else needs a target.
        let primary = match def.target_kind {
            TargetKind::SelfOnly => return Ok(caster),
            _ => target.ok_or(RejectReason::InvalidTarget)?,
        };
        let t = self
            .actors
            .get(&primary)
            .ok_or(RejectReason::NoSuchTarget)?;
        let same_team = t.team == c.team;
        let relation_ok = match def.target_kind {
            TargetKind::Enemy => !same_team,
            TargetKind::Ally => same_team,
            TargetKind::SelfOnly => true,
        };
        if !relation_ok {
            return Err(RejectReason::InvalidTarget);
        }
        if dist_sq(c.pos.0, t.pos.0) > def.range * def.range {
            return Err(RejectReason::OutOfRange);
        }
        Ok(primary)
    }

    /// Select every entity the effects apply to, in deterministic id order.
    fn select_targets(
        &self,
        caster: EntityId,
        primary: EntityId,
        def: &AbilityDef,
    ) -> Vec<EntityId> {
        match def.shape {
            TargetShape::Single => vec![primary],
            TargetShape::Radius(r) => {
                let Some(origin) = self.actors.get(&primary).map(|a| a.pos.0) else {
                    return Vec::new();
                };
                let caster_team = self.actors.get(&caster).map(|a| a.team);
                self.actors
                    .iter()
                    .filter(|(_, a)| dist_sq(a.pos.0, origin) <= r * r)
                    .filter(|(_, a)| match def.target_kind {
                        TargetKind::Enemy => caster_team != Some(a.team),
                        TargetKind::Ally | TargetKind::SelfOnly => caster_team == Some(a.team),
                    })
                    .map(|(&id, _)| id)
                    .collect()
            }
        }
    }

    /// Resolve a full cast deterministically. On success, costs are paid, the
    /// cooldown/GCD armed, effects applied, and threat generated.
    ///
    /// # Errors
    /// A [`RejectReason`] if any validation gate fails; no state changes then.
    pub fn cast(
        &mut self,
        now: Tick,
        caster: EntityId,
        def: &AbilityDef,
        target: Option<EntityId>,
    ) -> Result<(), RejectReason> {
        let primary = self.validate(now, caster, def, target)?;
        let targets = self.select_targets(caster, primary, def);

        // Costs are atomic with the cast: pay + arm cooldown before any effect.
        if let Some(c) = self.actors.get_mut(&caster) {
            c.power.spend(def.power_cost);
            c.cooldowns
                .trigger(def.id, now, def.cooldown_ticks, def.gcd_ticks);
        }

        for id in targets {
            for effect in &def.effects {
                self.apply_effect(now, caster, id, *effect);
            }
        }
        Ok(())
    }

    /// Apply one resolved effect to one target, generating threat for damage.
    fn apply_effect(&mut self, now: Tick, caster: EntityId, target: EntityId, effect: EffectKind) {
        let Some(actor) = self.actors.get_mut(&target) else {
            return;
        };
        match effect {
            EffectKind::Damage(amount) => {
                actor.health.damage(amount);
                actor.threat.add(caster, i64::from(amount));
            }
            EffectKind::Heal(amount) => actor.health.heal(amount),
            EffectKind::ApplyAura(spec) => actor.auras.apply(caster, spec, now),
        }
    }

    /// Advance all auras: fire any due periodic effect, reschedule, and drop
    /// expired auras. Deterministic over the ordered actor set.
    pub fn tick_auras(&mut self, now: Tick) {
        // Collect due firings first so we never mutate while borrowing the map.
        let mut firings: Vec<(EntityId, EntityId, Periodic)> = Vec::new();
        for (&id, actor) in &mut self.actors {
            for aura in &mut actor.auras.active {
                while aura.next_tick <= now && now < aura.expire_tick {
                    firings.push((id, aura.source, aura.spec.periodic));
                    let period = u64::from(aura.spec.period_ticks.max(1));
                    aura.next_tick = Tick(aura.next_tick.0.saturating_add(period));
                }
            }
            actor.auras.prune(now);
        }
        for (target, source, periodic) in firings {
            let Some(actor) = self.actors.get_mut(&target) else {
                continue;
            };
            match periodic {
                Periodic::Damage(amount) => {
                    actor.health.damage(amount);
                    actor.threat.add(source, i64::from(amount));
                }
                Periodic::Heal(amount) => actor.health.heal(amount),
            }
        }
    }
}

#[cfg(test)]
mod tests;
