//! The session registry: the shard's authoritative map from a live transport
//! [`ConnId`] to the one server-issued [`EntityId`] that connection drives.
//!
//! This is the seam that keeps the server authoritative over *identity*. A
//! client names things by the ids it sees on the wire; it can never mint one.
//! On accept the shard **spawns** an actor into the [`World`] under a fresh
//! server id ([`World::spawn`]) and records the binding here; on disconnect it
//! **despawns** ([`World::despawn`]) and drops the binding. Every gameplay
//! intent a connection sends is then keyed by [`SessionRegistry::entity_of`] —
//! the caster the *server* chose, never one the client asserted. That is what
//! supersedes the by-raw-value stopgap the pure sim used before a real registry
//! existed: the id keying an input is now a real lookup, not a coincidence of
//! two id spaces happening to line up.
//!
//! A [`BTreeMap`] keeps iteration in deterministic [`ConnId`] order, matching
//! the replay-stable ordering the rest of the server relies on.

use std::collections::BTreeMap;

use omm_ecs_core::EntityId;
use omm_sim::{combat::Actor, World};
use omm_transport::ConnId;

/// Maps each live connection to the single entity it controls this session.
///
/// The registry owns the *mapping*; the [`World`] owns the *entities*. Every
/// mutating method takes the `World` so the two never drift — a binding without
/// a live actor (or an orphaned actor with no binding) cannot arise.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    /// Connection → server-issued entity, ordered for deterministic iteration.
    bindings: BTreeMap<ConnId, EntityId>,
}

impl SessionRegistry {
    /// An empty registry with no bound connections.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind `conn` to a freshly spawned `actor`, returning its server id.
    ///
    /// The actor is admitted to `world` under a monotonic, server-issued id (the
    /// client never picks it) and the binding is recorded. If `conn` was already
    /// bound — a duplicate accept on a live connection — its previous entity is
    /// despawned first, so a reconnect can never orphan an actor on the shard.
    pub fn bind(&mut self, conn: ConnId, world: &mut World, actor: Actor) -> EntityId {
        if let Some(stale) = self.bindings.remove(&conn) {
            world.despawn(stale);
        }
        let id = world.spawn(actor);
        self.bindings.insert(conn, id);
        id
    }

    /// Drop `conn`'s binding and despawn its entity, returning the removed actor.
    ///
    /// Returns `None` — a no-op — if `conn` was never bound (a disconnect for a
    /// connection that never finished its handshake), so a double-close can never
    /// panic or double-despawn.
    pub fn unbind(&mut self, conn: ConnId, world: &mut World) -> Option<Actor> {
        let id = self.bindings.remove(&conn)?;
        world.despawn(id)
    }

    /// The server entity `conn` drives, if it is bound.
    ///
    /// This is the caster the shard stamps onto every intent from `conn` when it
    /// builds the tick's input batch — the server-resolved id, never one named
    /// by the client.
    #[must_use]
    pub fn entity_of(&self, conn: ConnId) -> Option<EntityId> {
        self.bindings.get(&conn).copied()
    }

    /// Whether `conn` currently drives an entity.
    #[must_use]
    pub fn is_bound(&self, conn: ConnId) -> bool {
        self.bindings.contains_key(&conn)
    }

    /// The number of live sessions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether no connection is currently bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
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

    fn conn(raw: u64) -> ConnId {
        ConnId::new(raw)
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = SessionRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(!reg.is_bound(conn(1)));
        assert_eq!(reg.entity_of(conn(1)), None);
    }

    #[test]
    fn bind_spawns_actor_and_records_binding() {
        let mut reg = SessionRegistry::new();
        let mut world = World::new();
        let id = reg.bind(conn(7), &mut world, actor());
        // The actor is live in the world under the returned server id...
        assert!(world.get(id).is_some());
        assert_eq!(world.len(), 1);
        // ...and the connection resolves to it.
        assert_eq!(reg.entity_of(conn(7)), Some(id));
        assert!(reg.is_bound(conn(7)));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn unbind_despawns_and_removes_binding() {
        let mut reg = SessionRegistry::new();
        let mut world = World::new();
        let id = reg.bind(conn(3), &mut world, actor());
        let removed = reg.unbind(conn(3), &mut world);
        assert!(removed.is_some(), "the despawned actor is returned");
        assert!(world.get(id).is_none(), "entity is gone from the world");
        assert!(world.is_empty());
        assert_eq!(reg.entity_of(conn(3)), None);
        assert!(reg.is_empty());
    }

    #[test]
    fn unbind_unknown_conn_is_a_noop() {
        let mut reg = SessionRegistry::new();
        let mut world = World::new();
        // A disconnect for a connection that never bound must not panic and must
        // not touch the world.
        assert!(reg.unbind(conn(99), &mut world).is_none());
        assert!(world.is_empty());
    }

    #[test]
    fn distinct_conns_get_distinct_entities() {
        let mut reg = SessionRegistry::new();
        let mut world = World::new();
        let a = reg.bind(conn(1), &mut world, actor());
        let b = reg.bind(conn(2), &mut world, actor());
        assert_ne!(a, b);
        assert_eq!(world.len(), 2);
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.entity_of(conn(1)), Some(a));
        assert_eq!(reg.entity_of(conn(2)), Some(b));
    }

    #[test]
    fn rebinding_a_conn_despawns_the_stale_entity() {
        let mut reg = SessionRegistry::new();
        let mut world = World::new();
        let first = reg.bind(conn(5), &mut world, actor());
        // A second accept on the same live connection replaces the entity...
        let second = reg.bind(conn(5), &mut world, actor());
        assert_ne!(first, second, "a fresh server id is issued");
        // ...leaving no orphan: exactly one live actor, the new one.
        assert!(world.get(first).is_none(), "stale entity is despawned");
        assert!(world.get(second).is_some());
        assert_eq!(world.len(), 1);
        assert_eq!(reg.entity_of(conn(5)), Some(second));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn unbinding_one_conn_leaves_others_bound() {
        let mut reg = SessionRegistry::new();
        let mut world = World::new();
        let a = reg.bind(conn(1), &mut world, actor());
        let b = reg.bind(conn(2), &mut world, actor());
        reg.unbind(conn(1), &mut world);
        assert_eq!(reg.entity_of(conn(1)), None);
        assert_eq!(reg.entity_of(conn(2)), Some(b));
        assert!(world.get(a).is_none());
        assert!(world.get(b).is_some());
        assert_eq!(world.len(), 1);
    }
}
