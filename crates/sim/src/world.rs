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
use omm_ecs_core::EntityId;
use omm_protocol::Tick;
use std::collections::BTreeMap;

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
