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
    pub fn step(&mut self, inputs: &InputBatch, abilities: &BTreeMap<AbilityId, AbilityDef>) {
        self.movement(inputs); // IngestIntents + Movement
        self.resolve_casts(inputs, abilities); // ResolveCasts
        self.tick_auras(self.now); // TickAuras
        self.prune_dead(); // Prune
        self.now = self.now.next();
    }

    /// `Move` intents set each actor's velocity, then every actor integrates one
    /// fixed tick. A `Move` naming an absent entity is a no-op.
    fn movement(&mut self, inputs: &InputBatch) {
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
    fn resolve_casts(&mut self, inputs: &InputBatch, abilities: &BTreeMap<AbilityId, AbilityDef>) {
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
/// the shard owns the authoritative entity set and its server-issued ids. Until
/// the session registry lands (client ↔ server-issued entity, a later slice), the
/// two id spaces are bridged by raw value. This stays server-authoritative: the
/// resolved id is still validated inside [`World::cast`], so a client naming an
/// absent or illegal target is rejected, never trusted.
fn resolve_target(target: Option<CharacterId>) -> Option<EntityId> {
    target.map(|c| EntityId::new(c.raw()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_ecs_core::Team;
    use omm_protocol::Vec3;

    fn actor() -> Actor {
        Actor::new(Vec3::default(), Team(1), 100, 100)
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
}
