//! Deterministic movement systems: intent → velocity, then velocity → position.
//!
//! The server owns position. A client sends only the *direction* it wants to
//! travel (an [`omm_protocol::Intent::Move`]); the server turns that into the
//! actor's velocity ([`apply_move`]) and, once per fixed tick, advances every
//! actor's position under its current velocity ([`integrate_all`]). Splitting
//! the two mirrors an ECS schedule — an input system then an integration
//! system — and keeps intent→velocity and velocity→position each under one
//! reason to change.
//!
//! Both systems are pure over their inputs: no wall-clock, no RNG, no I/O.
//! [`integrate_all`] walks the world's actors in server-issued id order, so the
//! integrated state is bit-identical across runs and machines — the property
//! replay and anti-cheat re-simulation depend on.

use crate::combat::Actor;
use crate::World;
use omm_ecs_core::{integrate, Velocity};
use omm_protocol::Vec3;

/// Set `actor`'s velocity from a requested move direction.
///
/// The direction is taken **raw** — the requested vector becomes the velocity
/// verbatim. The server does not yet clamp it to a legal per-class speed; that
/// authority gate is a later slice, deliberately kept out of this system so the
/// intent→velocity step stays single-purpose. Because a client only ever sends
/// this intent and never a position, movement stays server-authoritative: the
/// client asks to move, the server decides where the actor ends up.
pub fn apply_move(actor: &mut Actor, dir: &Vec3) {
    actor.vel = Velocity(*dir);
}

/// Advance every actor's position by one fixed tick under its current velocity.
///
/// Traversal is in server-issued id order (the world's `BTreeMap`), so the
/// resulting world state is identical across runs and machines. An actor with
/// zero velocity stays put; integration is applied uniformly, so callers never
/// special-case the idle path.
pub fn integrate_all(world: &mut World) {
    for actor in world.actors.values_mut() {
        actor.pos = integrate(actor.pos, actor.vel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_ecs_core::{EntityId, Position, Team, TICK_DT};
    use omm_protocol::Vec3;
    use proptest::prelude::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn actor_at(x: f32, z: f32) -> Actor {
        Actor::new(at(x, z), Team(1), 100, 100)
    }

    #[test]
    fn apply_move_sets_velocity_raw() {
        let mut a = actor_at(0.0, 0.0);
        apply_move(&mut a, &at(1.0, -1.0));
        assert_eq!(a.vel, Velocity(at(1.0, -1.0)));
    }

    #[test]
    fn apply_move_does_not_clamp_speed() {
        // Speed clamping is a later slice — an over-fast request is applied raw.
        let mut a = actor_at(0.0, 0.0);
        apply_move(&mut a, &at(1000.0, 0.0));
        assert_eq!(a.vel, Velocity(at(1000.0, 0.0)));
    }

    #[test]
    fn apply_move_overwrites_prior_velocity() {
        let mut a = actor_at(0.0, 0.0);
        apply_move(&mut a, &at(1.0, 0.0));
        apply_move(&mut a, &at(0.0, 1.0));
        assert_eq!(a.vel, Velocity(at(0.0, 1.0)));
    }

    #[test]
    fn integrate_all_advances_position_one_tick() {
        let mut w = World::new();
        let id = w.spawn(actor_at(0.0, 0.0));
        apply_move(w.actors.get_mut(&id).unwrap(), &at(3.0, -6.0));
        integrate_all(&mut w);
        let p = w.get(id).unwrap().pos;
        assert_eq!(p, Position(at(3.0 * TICK_DT, -6.0 * TICK_DT)));
    }

    #[test]
    fn integrate_all_leaves_idle_actor_in_place() {
        let mut w = World::new();
        let id = w.spawn(actor_at(5.0, 5.0));
        integrate_all(&mut w);
        assert_eq!(w.get(id).unwrap().pos, Position(at(5.0, 5.0)));
    }

    #[test]
    fn integrate_all_moves_every_actor_independently() {
        let mut w = World::new();
        let a = w.spawn(actor_at(0.0, 0.0));
        let b = w.spawn(actor_at(10.0, 0.0));
        apply_move(w.actors.get_mut(&a).unwrap(), &at(1.0, 0.0));
        apply_move(w.actors.get_mut(&b).unwrap(), &at(0.0, 2.0));
        integrate_all(&mut w);
        assert_eq!(w.get(a).unwrap().pos, Position(at(TICK_DT, 0.0)));
        assert_eq!(w.get(b).unwrap().pos, Position(at(10.0, 2.0 * TICK_DT)));
    }

    #[test]
    fn repeated_integration_accumulates_over_ticks() {
        let mut w = World::new();
        let id = w.spawn(actor_at(0.0, 0.0));
        apply_move(w.actors.get_mut(&id).unwrap(), &at(1.0, 0.0));
        for _ in 0..3 {
            integrate_all(&mut w);
        }
        assert_eq!(w.get(id).unwrap().pos, Position(at(3.0 * TICK_DT, 0.0)));
    }

    #[test]
    fn integrate_all_on_empty_world_is_a_noop() {
        let mut w = World::new();
        integrate_all(&mut w);
        assert!(w.is_empty());
    }

    proptest! {
        /// Determinism: the same ordered move+integrate stream yields identical
        /// positions across two independent runs.
        #[test]
        fn movement_is_deterministic(
            dirs in prop::collection::vec((-1.0f32..1.0, -1.0f32..1.0), 0..64)
        ) {
            let build = || {
                let mut w = World::new();
                let id = w.spawn(actor_at(0.0, 0.0));
                for &(x, z) in &dirs {
                    apply_move(w.actors.get_mut(&id).unwrap(), &at(x, z));
                    integrate_all(&mut w);
                }
                w.get(id).map(|a| a.pos)
            };
            prop_assert_eq!(build(), build());
        }

        /// Order-independence of spawn ids: integrating two actors moves each by
        /// exactly its own velocity, regardless of traversal order.
        #[test]
        fn each_actor_integrates_by_its_own_velocity(
            vx in -5.0f32..5.0, vz in -5.0f32..5.0,
        ) {
            let mut w = World::new();
            let a = w.spawn(actor_at(0.0, 0.0));
            let b = w.spawn(actor_at(100.0, 100.0));
            apply_move(w.actors.get_mut(&a).unwrap(), &at(vx, vz));
            // b stays idle.
            integrate_all(&mut w);
            prop_assert_eq!(w.get(a).unwrap().pos, Position(at(vx * TICK_DT, vz * TICK_DT)));
            prop_assert_eq!(w.get(b).unwrap().pos, Position(at(100.0, 100.0)));
            // Ids are distinct and server-issued.
            prop_assert_ne!(a, b);
            prop_assert_eq!(a, EntityId::new(1));
        }
    }
}
