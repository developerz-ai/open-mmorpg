//! Deterministic, platform-stable hashing of the authoritative world state.
//!
//! [`WorldHash`] is a single `u64` fingerprint of every live actor. Two runs of
//! the sim from the same start produce the same hash on any machine — the cheap
//! oracle that server-side replay and anti-cheat re-simulation compare each tick
//! instead of diffing the whole world.
//!
//! Stability, not speed, is the point, so the fold is spelled out by hand:
//! - **FNV-1a** over little-endian bytes (mirrors [`fnv1a`] in the gateway
//!   router) — dependency-free and identical across builds, unlike the stdlib
//!   `DefaultHasher`, whose seed is unspecified.
//! - Floats fold by [`f32::to_bits`] (like [`crate::combat`]'s wire path), never
//!   a `==` compare — the same reason the sim avoids `sqrt` on the tick path.
//! - Actors fold in server-issued id order (the world's `BTreeMap`), and every
//!   variable-length part (cooldowns, auras) is length-prefixed so no two
//!   distinct states can serialize to the same byte stream.
//!
//! [`fnv1a`]: ../../gateway/src/routing.rs

use crate::combat::Actor;
use omm_ecs_core::{Auras, Cooldowns, EntityId, Periodic};
use omm_protocol::Vec3;
use std::collections::BTreeMap;
use std::fmt;

/// A deterministic, platform-stable fingerprint of a [`World`]'s state.
///
/// Equal hashes mean two worlds hold bit-identical actor state; a single changed
/// position, hit point, or armed cooldown flips it. Displays as zero-padded hex
/// for logs.
///
/// [`World`]: crate::World
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct WorldHash(pub u64);

impl WorldHash {
    /// The underlying raw value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for WorldHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Fold the id-ordered actor set into a [`WorldHash`]. Driven by
/// [`World::state_hash`](crate::World::state_hash).
pub(crate) fn hash_world(actors: &BTreeMap<EntityId, Actor>) -> WorldHash {
    let mut h = Fnv1a::new();
    for (&id, actor) in actors {
        write_actor(&mut h, id, actor);
    }
    WorldHash(h.finish())
}

/// Fold one actor: id, then the state fields the spec pins. Team and threat are
/// deliberately excluded — team is set once at spawn and never mutates, and
/// threat is fully derived from the damage already reflected in health, so
/// neither can make two runs from the same start diverge.
fn write_actor(h: &mut Fnv1a, id: EntityId, actor: &Actor) {
    h.write_u64(id.raw());
    h.write_vec3(actor.pos.0);
    h.write_vec3(actor.vel.0);
    h.write_u32(actor.health.current);
    h.write_u32(actor.health.max);
    h.write_u32(actor.power.current);
    h.write_u32(actor.power.max);
    write_cooldowns(h, &actor.cooldowns);
    write_auras(h, &actor.auras);
}

/// Fold the GCD deadline plus the id-ordered armed cooldowns, length-prefixed.
fn write_cooldowns(h: &mut Fnv1a, cooldowns: &Cooldowns) {
    h.write_u64(cooldowns.gcd_until().0);
    let armed: Vec<_> = cooldowns.ready_ticks().collect();
    h.write_u64(armed.len() as u64);
    for (ability, ready_at) in armed {
        h.write_u32(ability.0);
        h.write_u64(ready_at.0);
    }
}

/// Fold the active auras in application order, length-prefixed. Order is stable
/// because the same input stream applies them in the same sequence.
fn write_auras(h: &mut Fnv1a, auras: &Auras) {
    h.write_u64(auras.active.len() as u64);
    for aura in &auras.active {
        h.write_u64(aura.source.raw());
        h.write_u32(aura.spec.period_ticks);
        h.write_u32(aura.spec.duration_ticks);
        write_periodic(h, aura.spec.periodic);
        h.write_u64(aura.next_tick.0);
        h.write_u64(aura.expire_tick.0);
    }
}

/// Fold a periodic payload: a discriminant byte then its amount.
fn write_periodic(h: &mut Fnv1a, periodic: Periodic) {
    match periodic {
        Periodic::Damage(amount) => {
            h.write_u8(0);
            h.write_u32(amount);
        }
        Periodic::Heal(amount) => {
            h.write_u8(1);
            h.write_u32(amount);
        }
    }
}

/// An incremental FNV-1a hasher over little-endian bytes. Chosen for stability
/// across builds and machines (the stdlib hasher is seeded and is not).
struct Fnv1a {
    state: u64,
}

impl Fnv1a {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self {
            state: Self::OFFSET_BASIS,
        }
    }

    fn write_u8(&mut self, byte: u8) {
        self.state ^= u64::from(byte);
        self.state = self.state.wrapping_mul(Self::PRIME);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_u8(byte);
        }
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Fold a float by its exact IEEE-754 bit pattern — never a `==` compare, and
    /// stable across platforms since `to_bits` is a fixed reinterpretation.
    fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    fn write_vec3(&mut self, v: Vec3) {
        self.write_f32(v.x);
        self.write_f32(v.y);
        self.write_f32(v.z);
    }

    const fn finish(&self) -> u64 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use omm_ecs_core::{ActiveAura, AuraSpec, Health, Position, Power, Team, Threat, Velocity};
    use omm_protocol::Tick;
    use proptest::prelude::*;

    fn v(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    /// A fully-specified actor so every hashed field has a distinct value.
    fn rich_actor() -> Actor {
        Actor {
            pos: Position(v(1.0, 2.0, 3.0)),
            vel: Velocity(v(4.0, 5.0, 6.0)),
            health: Health {
                current: 90,
                max: 100,
            },
            power: Power {
                current: 40,
                max: 50,
            },
            team: Team(7),
            cooldowns: Cooldowns::default(),
            auras: Auras::default(),
            threat: Threat::default(),
        }
    }

    #[test]
    fn empty_world_hashes_to_the_fnv_offset_basis() {
        // No actors folded → the raw seed. Pins the seed and the empty case to a
        // known constant, the anchor of platform stability.
        assert_eq!(
            hash_world(&BTreeMap::new()),
            WorldHash(0xcbf2_9ce4_8422_2325)
        );
    }

    #[test]
    fn hash_is_a_known_platform_stable_vector() {
        // Cross-check the production fold against an independent, hand-written
        // byte layout for one known actor. If the field order or width drifts,
        // this fails — the canary for the wire-stable contract.
        let mut w = World::new();
        w.spawn(rich_actor());

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u64.to_le_bytes()); // first server-issued id
        for f in [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0] {
            bytes.extend_from_slice(&f.to_bits().to_le_bytes()); // pos then vel
        }
        bytes.extend_from_slice(&90u32.to_le_bytes()); // health.current
        bytes.extend_from_slice(&100u32.to_le_bytes()); // health.max
        bytes.extend_from_slice(&40u32.to_le_bytes()); // power.current
        bytes.extend_from_slice(&50u32.to_le_bytes()); // power.max
        bytes.extend_from_slice(&0u64.to_le_bytes()); // gcd_until
        bytes.extend_from_slice(&0u64.to_le_bytes()); // cooldown count
        bytes.extend_from_slice(&0u64.to_le_bytes()); // aura count

        let mut expected = Fnv1a::new();
        expected.write_bytes(&bytes);
        assert_eq!(w.state_hash(), WorldHash(expected.finish()));
    }

    #[test]
    fn identical_worlds_hash_equal() {
        let build = || {
            let mut w = World::new();
            w.spawn(rich_actor());
            w.spawn(Actor::new(v(9.0, 0.0, -2.0), Team(2), 200, 0));
            w
        };
        assert_eq!(build().state_hash(), build().state_hash());
    }

    #[test]
    fn each_folded_field_changes_the_hash() {
        let base = {
            let mut w = World::new();
            w.spawn(rich_actor());
            w.state_hash()
        };
        // Perturb one field at a time; each must move the hash.
        let mutate = |f: &dyn Fn(&mut Actor)| {
            let mut a = rich_actor();
            f(&mut a);
            let mut w = World::new();
            w.spawn(a);
            w.state_hash()
        };
        assert_ne!(base, mutate(&|a| a.pos.0.x += 1.0), "position must count");
        assert_ne!(base, mutate(&|a| a.vel.0.z -= 1.0), "velocity must count");
        assert_ne!(
            base,
            mutate(&|a| a.health.current = 91),
            "health must count"
        );
        assert_ne!(
            base,
            mutate(&|a| a.health.max = 101),
            "health max must count"
        );
        assert_ne!(base, mutate(&|a| a.power.current = 41), "power must count");
        assert_ne!(base, mutate(&|a| a.power.max = 51), "power max must count");
    }

    #[test]
    fn team_and_threat_are_excluded_from_the_hash() {
        let base = {
            let mut w = World::new();
            w.spawn(rich_actor());
            w.state_hash()
        };
        let with_team = {
            let mut a = rich_actor();
            a.team = Team(999);
            a.threat.add(EntityId::new(3), 500);
            let mut w = World::new();
            w.spawn(a);
            w.state_hash()
        };
        // Static/derived fields don't drive divergence — they stay out of the hash.
        assert_eq!(base, with_team);
    }

    #[test]
    fn cooldowns_and_auras_change_the_hash() {
        let base = {
            let mut w = World::new();
            w.spawn(rich_actor());
            w.state_hash()
        };
        let with_cd = {
            let mut a = rich_actor();
            a.cooldowns
                .trigger(omm_ecs_core::AbilityId(1), Tick(0), 30, 3);
            let mut w = World::new();
            w.spawn(a);
            w.state_hash()
        };
        let with_aura = {
            let mut a = rich_actor();
            a.auras.active.push(ActiveAura {
                source: EntityId::new(1),
                spec: AuraSpec {
                    period_ticks: 3,
                    duration_ticks: 9,
                    periodic: Periodic::Damage(5),
                },
                next_tick: Tick(3),
                expire_tick: Tick(9),
            });
            let mut w = World::new();
            w.spawn(a);
            w.state_hash()
        };
        assert_ne!(base, with_cd, "an armed cooldown must count");
        assert_ne!(base, with_aura, "an active aura must count");
        assert_ne!(with_cd, with_aura);
    }

    proptest! {
        /// Same actors, same order → identical hash, every run. The determinism
        /// the replay/anti-cheat oracle stands on.
        #[test]
        fn hash_is_deterministic_over_spawns(
            spawns in prop::collection::vec(
                (-50.0f32..50.0, -50.0f32..50.0, 1u32..500), 0..24)
        ) {
            let build = || {
                let mut w = World::new();
                for &(x, z, hp) in &spawns {
                    w.spawn(Actor::new(v(x, 0.0, z), Team(1), hp, hp));
                }
                w.state_hash()
            };
            prop_assert_eq!(build(), build());
        }

        /// Any single actor at a distinct position changes the fold — a moved
        /// entity can never hide behind an unchanged hash.
        #[test]
        fn moving_one_actor_diverges(dx in 0.01f32..10.0) {
            let base = {
                let mut w = World::new();
                w.spawn(Actor::new(v(0.0, 0.0, 0.0), Team(1), 100, 100));
                w.state_hash()
            };
            let moved = {
                let mut w = World::new();
                w.spawn(Actor::new(v(dx, 0.0, 0.0), Team(1), 100, 100));
                w.state_hash()
            };
            prop_assert_ne!(base, moved);
        }
    }
}
