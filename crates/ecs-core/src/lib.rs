//! Shared ECS components and the pure systems that operate on them.
//!
//! This crate is **pure**: types + logic, no I/O, no async, no globals. The
//! server (`shard`) and the client run the same component model, so movement
//! looks identical on both sides — the client predicts, the server decides.
//!
//! Systems here are deterministic: same components in, same components out. That
//! is what makes replay and anti-cheat re-simulation possible (see `omm-sim`).
//!
//! The scaffold models components as plain structs. When the engine grows we
//! adopt Bevy ECS storage (docs/architecture/05-ecs-and-scripting.md); the
//! component *shapes* below are chosen to port cleanly.

pub mod combat;

pub use combat::{
    AbilityDef, AbilityId, ActiveAura, AuraSpec, Auras, Cooldowns, EffectKind, EntityId, Periodic,
    Power, TargetKind, TargetShape, Team, Threat,
};

use omm_protocol::Vec3;

/// Fixed simulation timestep in seconds. Fixed, not wall-clock, so the sim is
/// deterministic regardless of frame rate.
pub const TICK_DT: f32 = 1.0 / 30.0;

/// World position of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position(pub Vec3);

/// Velocity in units/second.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity(pub Vec3);

/// Current and maximum hit points. `current` is clamped to `0..=max`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    /// A full-health pool.
    #[must_use]
    pub const fn full(max: u32) -> Self {
        Self { current: max, max }
    }

    /// Apply `amount` of damage, saturating at 0. Never underflows.
    pub fn damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }

    /// Heal by `amount`, never exceeding `max`.
    pub fn heal(&mut self, amount: u32) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }

    /// Whether the entity is dead.
    #[must_use]
    pub const fn is_dead(&self) -> bool {
        self.current == 0
    }
}

/// Advance a position by one fixed tick under a velocity. Deterministic.
#[must_use]
pub fn integrate(pos: Position, vel: Velocity) -> Position {
    Position(Vec3 {
        x: pos.0.x + vel.0.x * TICK_DT,
        y: pos.0.y + vel.0.y * TICK_DT,
        z: pos.0.z + vel.0.z * TICK_DT,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrate_is_deterministic() {
        let p = Position(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let v = Velocity(Vec3 {
            x: 3.0,
            y: 0.0,
            z: -6.0,
        });
        let a = integrate(p, v);
        let b = integrate(p, v);
        assert_eq!(a, b, "same inputs must yield identical output");
        assert!((a.0.x - 0.1).abs() < 1e-6);
        assert!((a.0.z + 0.2).abs() < 1e-6);
    }

    #[test]
    fn health_saturates_and_reports_death() {
        let mut h = Health::full(100);
        h.damage(30);
        assert_eq!(h.current, 70);
        h.heal(1000);
        assert_eq!(h.current, 100);
        h.damage(9999);
        assert!(h.is_dead());
    }
}
