//! The authoritative simulation world — the live entity set one shard owns.
//!
//! [`World`] is the mutable heart of the tick loop: it holds every [`Actor`]
//! keyed by a **server-issued** [`EntityId`] and stamps the current [`Tick`].
//! Ids come from a monotonic counter here, never from a client — that is the
//! first anti-spoof gate (a client can name a target, never mint one). A
//! [`BTreeMap`] keeps every traversal — targeting, snapshots, world-state
//! hashing — in deterministic id order, the property replay and anti-cheat
//! re-simulation depend on.
//!
//! Combat resolution lives in [`crate::combat`], hung off this same `World` so
//! the schedule advances one authoritative structure rather than several.

use crate::combat::Actor;
use crate::hash::WorldHash;
use crate::movement::{apply_move, integrate_all};
use omm_ecs_core::{AbilityDef, AbilityId, EntityId};
use omm_protocol::{CharacterId, Intent, Tick};
use std::collections::BTreeMap;

/// One tick's validated intents — at most one per acting entity — keyed by the
/// **server-issued** [`EntityId`] of the caster and sorted by it, so iteration
/// (and therefore the whole tick) is deterministic. The shard builds this from
/// the transport before calling [`World::step`]; the id is the actor the server
/// resolved for a session, never one a client named.
///
/// The id-sorted invariant is the caller's to uphold — it is the canonical order
/// two shards must agree on for cross-shard re-simulation to line up bit-for-bit.
pub type InputBatch = Vec<(EntityId, Intent)>;

/// The authoritative entity set for one shard, advanced one fixed [`Tick`] at a
/// time by the server's schedule.
#[derive(Debug, Clone, Default)]
pub struct World {
    /// Live actors, ordered by id so every traversal is deterministic.
    pub(crate) actors: BTreeMap<EntityId, Actor>,
    /// The tick this world's state is stamped at; advanced by the tick loop.
    now: Tick,
    /// Monotonic id source. Server-issued only — a client can never pick an id.
    next_id: u64,
}

impl World {
    /// An empty world at tick zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit `actor` under a fresh server-issued id and return it.
    ///
    /// Ids strictly increase and are never reused within a world's life, so a
    /// stale handle can never alias a later entity.
    pub fn spawn(&mut self, actor: Actor) -> EntityId {
        self.next_id += 1;
        let id = EntityId::new(self.next_id);
        self.actors.insert(id, actor);
        id
    }

    /// Remove an entity, returning its last state if it was present.
    pub fn despawn(&mut self, id: EntityId) -> Option<Actor> {
        self.actors.remove(&id)
    }

    /// Borrow an actor by id.
    #[must_use]
    pub fn get(&self, id: EntityId) -> Option<&Actor> {
        self.actors.get(&id)
    }

    /// Iterate live actors in server-issued id order.
    ///
    /// Read-only and deterministic (`BTreeMap` order), so the replication egress
    /// can snapshot the whole world without reaching into its internals. Yields
    /// each actor beside the server id a snapshot stamps on the wire.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &Actor)> {
        self.actors.iter().map(|(&id, actor)| (id, actor))
    }

    /// Number of live actors.
    #[must_use]
    pub fn len(&self) -> usize {
        self.actors.len()
    }

    /// Whether the world holds no actors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
    }

    /// The tick this world is currently stamped at.
    #[must_use]
    pub fn now(&self) -> Tick {
        self.now
    }

    /// A deterministic, platform-stable [`WorldHash`] of the authoritative state.
    ///
    /// Folds every actor in server-issued id order through FNV-1a — position and
    /// velocity by `f32::to_bits` (never a float compare), current/max health and
    /// power, and a cooldown/aura summary. Same state → same hash on any box, so
    /// replay and anti-cheat re-simulation detect divergence by comparing one
    /// `u64` per tick instead of the whole world. The tick counter itself is not
    /// folded: two runs compare hashes at matching ticks, so state alone decides.
    #[must_use]
    pub fn state_hash(&self) -> WorldHash {
        crate::hash::hash_world(&self.actors)
    }
}

/// The deterministic tick schedule — the systems of [`crate::schedule::System`],
/// run in [`ORDER`](crate::schedule::System::ORDER). This block is the executable
/// form of that determinism contract.
impl World {
    /// Advance the world exactly one fixed tick under `inputs`.
    ///
    /// Runs the fixed schedule — ingest intents, move, resolve casts, tick auras,
    /// prune the dead — then stamps the next tick. `abilities` is the content
    /// ability table (definitions are data; the engine only resolves them). The
    /// same `inputs` and `abilities` always produce the same resulting state:
    /// that is what replay and anti-cheat re-simulation rely on.
    ///
    /// A rejected cast is not an error here — it is an ordinary outcome the
    /// caller reports to the client; the tick advances regardless, never panics.
    pub fn step(
        &mut self,
        inputs: &[(EntityId, Intent)],
        abilities: &BTreeMap<AbilityId, AbilityDef>,
    ) {
        self.movement(inputs); // IngestIntents + Movement
        self.resolve_casts(inputs, abilities); // ResolveCasts
        self.tick_auras(self.now); // TickAuras
        self.prune_dead(); // Prune
        self.now = self.now.next();
    }

    /// `Move` intents set each actor's velocity, then every actor integrates one
    /// fixed tick. A `Move` naming an absent entity is a no-op.
    fn movement(&mut self, inputs: &[(EntityId, Intent)]) {
        for (id, intent) in inputs {
            if let Intent::Move { dir } = intent {
                if let Some(actor) = self.actors.get_mut(id) {
                    apply_move(actor, dir);
                }
            }
        }
        integrate_all(self);
    }

    /// `UseAbility` intents resolve through [`World::cast`]. The ability id is
    /// looked up in the content `abilities` table; an unknown id is dropped, and
    /// a rejected cast leaves state untouched — neither stalls the tick.
    fn resolve_casts(
        &mut self,
        inputs: &[(EntityId, Intent)],
        abilities: &BTreeMap<AbilityId, AbilityDef>,
    ) {
        let now = self.now;
        for (caster, intent) in inputs {
            let Intent::UseAbility { id, target } = intent else {
                continue;
            };
            let Some(def) = abilities.get(&AbilityId(*id)) else {
                continue;
            };
            let _ = self.cast(now, *caster, def, resolve_target(*target));
        }
    }

    /// Drop actors reduced to zero health from the live set, in id order.
    fn prune_dead(&mut self) {
        self.actors.retain(|_, actor| !actor.health.is_dead());
    }
}

/// Resolve the target id a client named onto a sim [`EntityId`].
///
/// A client can only *name* a target by the [`CharacterId`] it sees on the wire;
/// the shard owns the authoritative entity set and its server-issued ids. The
/// *caster* is already resolved server-side — the shard's session registry binds
/// each connection to the entity it drives — so the id keying every intent is
/// never one a client asserted. The *target* wire id is still bridged by raw
/// value here; resolving it against the shard's live entity set moves to the
/// input-batch boundary in a later slice. Either way this stays
/// server-authoritative: the resolved id is validated inside [`World::cast`], so
/// a client naming an absent or illegal target is rejected, never trusted.
fn resolve_target(target: Option<CharacterId>) -> Option<EntityId> {
    target.map(|c| EntityId::new(c.raw()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_ecs_core::{AbilityId, EffectKind, Position, TargetKind, TargetShape, Team, TICK_DT};
    use omm_protocol::{CharacterId, Vec3};
    use proptest::prelude::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn actor() -> Actor {
        Actor::new(Vec3::default(), Team(1), 100, 100)
    }

    /// A simple instant nuke — no cooldown, no GCD, range 999 so it always hits.
    fn instant_nuke() -> AbilityDef {
        AbilityDef {
            id: AbilityId(1),
            power_cost: 0,
            cooldown_ticks: 0,
            gcd_ticks: 0,
            range: 999.0,
            target_kind: TargetKind::Enemy,
            shape: TargetShape::Single,
            effects: vec![EffectKind::Damage(20)],
        }
    }

    fn ability_table(def: AbilityDef) -> BTreeMap<AbilityId, AbilityDef> {
        let mut m = BTreeMap::new();
        m.insert(def.id, def);
        m
    }

    #[test]
    fn spawn_issues_monotonic_server_ids() {
        let mut w = World::new();
        let a = w.spawn(actor());
        let b = w.spawn(actor());
        assert_eq!(a, EntityId::new(1));
        assert_eq!(b, EntityId::new(2));
        assert_eq!(w.len(), 2);
        assert!(!w.is_empty());
    }

    #[test]
    fn get_borrows_the_spawned_actor() {
        let mut w = World::new();
        let id = w.spawn(actor());
        assert!(w.get(id).is_some());
        assert!(w.get(EntityId::new(999)).is_none());
    }

    #[test]
    fn iter_yields_actors_in_server_id_order() {
        let mut w = World::new();
        let a = w.spawn(Actor::new(at(1.0, 0.0), Team(1), 100, 100));
        let b = w.spawn(Actor::new(at(2.0, 0.0), Team(1), 100, 100));
        let ids: Vec<_> = w.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![a, b]);
        // The borrow reads real actor state, not a placeholder.
        let found = w.iter().find(|(id, _)| *id == a).unwrap();
        assert_eq!(found.1.pos.0, at(1.0, 0.0));
    }

    #[test]
    fn despawn_removes_and_returns_actor() {
        let mut w = World::new();
        let id = w.spawn(actor());
        assert!(w.despawn(id).is_some());
        assert!(w.get(id).is_none());
        // Removing an absent id is a no-op, not a panic.
        assert!(w.despawn(id).is_none());
        assert!(w.is_empty());
    }

    #[test]
    fn ids_are_never_reused_after_despawn() {
        let mut w = World::new();
        let a = w.spawn(actor());
        w.despawn(a);
        let b = w.spawn(actor());
        assert_ne!(a, b);
        assert_eq!(b, EntityId::new(2));
    }

    #[test]
    fn new_world_is_empty_at_tick_zero() {
        let w = World::new();
        assert!(w.is_empty());
        assert_eq!(w.len(), 0);
        assert_eq!(w.now(), Tick(0));
    }

    // ── World::step determinism ──────────────────────────────────────────────

    #[test]
    fn step_advances_tick_counter() {
        let mut w = World::new();
        w.step(&[], &BTreeMap::new());
        assert_eq!(w.now(), Tick(1));
        w.step(&[], &BTreeMap::new());
        assert_eq!(w.now(), Tick(2));
    }

    #[test]
    fn empty_batch_is_movement_noop() {
        let mut w = World::new();
        let id = w.spawn(Actor::new(at(3.0, 7.0), Team(1), 100, 100));
        let before = w.get(id).unwrap().pos;
        w.step(&[], &BTreeMap::new());
        // No Move intent → velocity stays zero → position unchanged.
        assert_eq!(w.get(id).unwrap().pos, before);
    }

    #[test]
    fn move_intent_integrates_exactly_vel_times_tick_dt() {
        let mut w = World::new();
        let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
        let dir = at(3.0, -6.0);
        let inputs: InputBatch = vec![(id, Intent::Move { dir })];
        w.step(&inputs, &BTreeMap::new());
        assert_eq!(
            w.get(id).unwrap().pos,
            Position(at(3.0 * TICK_DT, -6.0 * TICK_DT)),
        );
    }

    #[test]
    fn use_ability_routes_to_cast_and_deals_damage() {
        let nuke = instant_nuke();
        let table = ability_table(nuke.clone());
        let mut w = World::new();
        let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
        let target = w.spawn(Actor::new(at(1.0, 0.0), Team(2), 100, 100));
        let inputs: InputBatch = vec![(
            caster,
            Intent::UseAbility {
                id: nuke.id.0,
                target: Some(CharacterId::new(target.raw())),
            },
        )];
        w.step(&inputs, &table);
        // 20 damage landed → target health reduced.
        assert_eq!(w.get(target).unwrap().health.current, 80);
    }

    #[test]
    fn use_ability_unknown_id_is_silently_dropped() {
        let table: BTreeMap<AbilityId, AbilityDef> = BTreeMap::new();
        let mut w = World::new();
        let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
        let target = w.spawn(Actor::new(at(1.0, 0.0), Team(2), 100, 100));
        let inputs: InputBatch = vec![(
            caster,
            Intent::UseAbility {
                id: 999, // not in table
                target: Some(CharacterId::new(target.raw())),
            },
        )];
        // Must not panic; target is untouched.
        w.step(&inputs, &table);
        assert_eq!(w.get(target).unwrap().health.current, 100);
    }

    #[test]
    fn step_prunes_dead_actors() {
        let nuke = instant_nuke();
        // Kill the target in one shot — spawn it with 20 hp.
        let table = ability_table(AbilityDef {
            effects: vec![EffectKind::Damage(20)],
            ..nuke
        });
        let mut w = World::new();
        let caster = w.spawn(Actor::new(at(0.0, 0.0), Team(1), 100, 100));
        let target = w.spawn(Actor::new(at(1.0, 0.0), Team(2), 20, 20));
        let inputs: InputBatch = vec![(
            caster,
            Intent::UseAbility {
                id: AbilityId(1).0,
                target: Some(CharacterId::new(target.raw())),
            },
        )];
        w.step(&inputs, &table);
        // Target is at zero health → prune_dead removed it.
        assert!(w.get(target).is_none());
        assert_eq!(w.len(), 1);
    }

    // ── World::state_hash ─────────────────────────────────────────────────────

    #[test]
    fn state_hash_ignores_the_tick_counter() {
        // Only actor state is folded, so an idle world hashes the same however
        // many empty ticks have advanced it.
        let mut w = World::new();
        w.spawn(Actor::new(at(2.0, 5.0), Team(1), 100, 100));
        let before = w.state_hash();
        w.step(&[], &BTreeMap::new());
        w.step(&[], &BTreeMap::new());
        assert_eq!(w.now(), Tick(2));
        assert_eq!(w.state_hash(), before, "empty ticks must not move the hash");
    }

    #[test]
    fn state_hash_tracks_a_move() {
        let mut w = World::new();
        let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
        let before = w.state_hash();
        w.step(
            &[(id, Intent::Move { dir: at(1.0, 0.0) })],
            &BTreeMap::new(),
        );
        assert_ne!(
            w.state_hash(),
            before,
            "an integrated move must change the hash"
        );
    }

    #[test]
    fn re_simulation_reproduces_the_same_hash() {
        // The anti-cheat contract: re-running the same inputs on a fresh world
        // reproduces the authoritative hash exactly, tick for tick.
        let run = || {
            let mut w = World::new();
            let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
            let mut hashes = Vec::new();
            for step in 0..8u32 {
                let dir = at(step as f32, -(step as f32));
                w.step(&[(id, Intent::Move { dir })], &BTreeMap::new());
                hashes.push(w.state_hash());
            }
            hashes
        };
        assert_eq!(run(), run());
    }

    // ── proptest: step determinism ───────────────────────────────────────────

    proptest! {
        /// Two independent step streams driven by the same input sequence yield
        /// bit-identical world state: position, health, tick counter.
        #[test]
        fn step_is_deterministic(
            moves in prop::collection::vec((-5.0f32..5.0, -5.0f32..5.0), 0..30)
        ) {
            let build = || {
                let mut w = World::new();
                let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
                for &(x, z) in &moves {
                    let inputs = vec![(id, Intent::Move { dir: at(x, z) })];
                    w.step(&inputs, &BTreeMap::new());
                }
                (w.get(id).map(|a| a.pos), w.now())
            };
            let (pos_a, tick_a) = build();
            let (pos_b, tick_b) = build();
            prop_assert_eq!(pos_a, pos_b);
            prop_assert_eq!(tick_a, tick_b);
        }

        /// A Move intent always integrates exactly `vel * TICK_DT` for arbitrary
        /// velocities — no floating-point branch or special-case.
        #[test]
        fn move_integrates_exactly(vx in -10.0f32..10.0, vz in -10.0f32..10.0) {
            let mut w = World::new();
            let id = w.spawn(Actor::new(Vec3::default(), Team(1), 100, 100));
            let inputs = vec![(id, Intent::Move { dir: at(vx, vz) })];
            w.step(&inputs, &BTreeMap::new());
            prop_assert_eq!(
                w.get(id).unwrap().pos,
                Position(at(vx * TICK_DT, vz * TICK_DT)),
            );
        }
    }
}
