//! Deterministic simulation core.
//!
//! Feed the same ordered inputs into [`simulate`] and you get bit-identical
//! state every time — on any box, any run. That property is the foundation for
//! three things at once: server-side replay, lockstep validation, and anti-cheat
//! re-simulation of a suspicious client. No wall-clock, no RNG, no I/O here.

pub mod combat;
pub mod movement;
pub mod world;

pub use movement::{apply_move, integrate_all};
pub use world::World;

use omm_ecs_core::{integrate, Health, Position, Velocity};
use omm_protocol::{Intent, Vec3};

/// The minimal authoritative state the sim owns for one entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityState {
    pub pos: Position,
    pub vel: Velocity,
    pub health: Health,
}

impl Default for EntityState {
    fn default() -> Self {
        Self {
            pos: Position(Vec3::default()),
            vel: Velocity(Vec3::default()),
            health: Health::full(100),
        }
    }
}

/// Apply a single validated intent, then advance one fixed tick.
///
/// Movement intents set velocity (the server's rules would clamp speed here);
/// ability use is a placeholder cost against health to exercise conservation.
#[must_use]
pub fn step(mut state: EntityState, intent: &Intent) -> EntityState {
    match intent {
        Intent::Move { dir } => state.vel = Velocity(*dir),
        Intent::UseAbility { .. } => state.health.damage(1),
    }
    state.pos = integrate(state.pos, state.vel);
    state
}

/// Run a full input sequence from a starting state. Deterministic.
#[must_use]
pub fn simulate(start: EntityState, inputs: &[Intent]) -> EntityState {
    inputs.iter().fold(start, step)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_protocol::CharacterId;
    use proptest::prelude::*;

    fn arb_intent() -> impl Strategy<Value = Intent> {
        prop_oneof![
            (-1.0f32..1.0, -1.0f32..1.0).prop_map(|(x, z)| Intent::Move {
                dir: Vec3 { x, y: 0.0, z }
            }),
            any::<u32>().prop_map(|id| Intent::UseAbility {
                id,
                target: Some(CharacterId::new(1)),
            }),
        ]
    }

    proptest! {
        /// Replay determinism: identical inputs → identical final state.
        #[test]
        fn simulation_is_deterministic(inputs in prop::collection::vec(arb_intent(), 0..64)) {
            let a = simulate(EntityState::default(), &inputs);
            let b = simulate(EntityState::default(), &inputs);
            prop_assert_eq!(a, b);
        }

        /// Health is conserved within bounds no matter the input stream.
        #[test]
        fn health_never_exceeds_max(inputs in prop::collection::vec(arb_intent(), 0..256)) {
            let s = simulate(EntityState::default(), &inputs);
            prop_assert!(s.health.current <= s.health.max);
        }
    }
}
