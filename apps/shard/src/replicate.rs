//! Snapshot egress: turn the authoritative [`World`] into a per-client delta.
//!
//! This is the server→client edge of replication, applied per client each tick
//! in the order netcode prescribes (docs/specs/game-server/netcode):
//! 1. [`world_entities`] snapshots the whole world into the wire shape.
//! 2. [`filter_by_interest`] keeps only the viewer's near-set, so a client's
//!    bandwidth is `O(nearby)`, never `O(world)`.
//! 3. [`fill_budget`] caps the rest to a per-tick byte budget by priority
//!    (self first); entities that don't fit slip to a later tick.
//! 4. [`diff`] against the client's acked baseline sends only what changed
//!    (the Quake3/Source delta model).
//!
//! The chosen frame becomes the client's next baseline. [`snapshot_msg`] then
//! serializes the delta into [`ServerMsg::Snapshot`] — protocol carries the
//! netcode frame opaquely, so it never has to depend on `omm-netcode`.

use std::collections::BTreeSet;

use omm_ecs_core::EntityId;
use omm_netcode::{
    diff, fill_budget, filter_by_interest, Candidate, DeltaFrame, EntitySnapshot, Priority,
    SnapshotFrame,
};
use omm_protocol::{ServerMsg, Tick};
use omm_sim::World;

/// Default per-client per-tick replication budget, in bytes. Comfortably below a
/// UDP MTU-sized burst; the priority filler keeps the high-priority set whole and
/// slips the tail to later ticks when a crowd blows past it.
pub const DEFAULT_BUDGET_BYTES: usize = 1200;

/// Snapshot the whole authoritative world into wire entities (raw ids).
///
/// The sim→netcode bridge: it reads `omm-sim` actors and emits the `omm-netcode`
/// shape, so neither crate need depend on the other. Facing (`yaw`) is derived
/// from the actor's velocity heading here — a presentation value produced only at
/// egress, never folded into the deterministic world hash — so a moving actor
/// faces its heading without the sim having to model facing.
#[must_use]
pub fn world_entities(world: &World) -> Vec<EntitySnapshot> {
    world
        .iter()
        .map(|(id, actor)| EntitySnapshot {
            id: id.raw(),
            pos: actor.pos.0,
            yaw: facing_yaw(actor.vel.0.x, actor.vel.0.z),
            health: actor.health.current,
        })
        .collect()
}

/// Yaw about the vertical axis from a ground-plane heading. `atan2(0, 0)` is `0`,
/// so a stationary actor faces zero with no special case.
fn facing_yaw(x: f32, z: f32) -> f32 {
    x.atan2(z)
}

/// Per-client replication cursor: the delta baseline (the last frame the shard
/// sent this client) plus its self id and byte budget. The baseline starts empty,
/// so a client's first snapshot is a full frame diffed against nothing.
#[derive(Debug)]
pub struct ClientReplication {
    /// The client's own entity (raw id) — always the highest priority.
    self_id: u64,
    /// Per-tick byte budget; entities past it slip to a later tick.
    budget: usize,
    /// The last frame sent — the baseline the next delta is computed against.
    baseline: SnapshotFrame,
}

impl ClientReplication {
    /// A cursor for the client driving `self_id`, with `budget` bytes per tick.
    #[must_use]
    pub fn new(self_id: EntityId, budget: usize) -> Self {
        Self {
            self_id: self_id.raw(),
            budget,
            baseline: SnapshotFrame::default(),
        }
    }

    /// The tick of this client's current baseline — [`Tick`] zero before its
    /// first send.
    #[must_use]
    pub fn baseline_tick(&self) -> Tick {
        self.baseline.tick
    }

    /// Build the delta to send this client this tick and advance its baseline.
    ///
    /// `all` is the world's full wire set from [`world_entities`]; `interest` is
    /// the near-set ids the spatial index returned for this viewer. Runs the
    /// AoI → budget → diff pipeline and adopts the chosen frame as the new
    /// baseline, so the next call diffs against exactly what this client last saw.
    pub fn delta_for(
        &mut self,
        tick: Tick,
        last_input_seq: u32,
        all: &[EntitySnapshot],
        interest: &BTreeSet<u64>,
    ) -> DeltaFrame {
        let candidates = filter_by_interest(all, interest)
            .into_iter()
            .map(|snap| Candidate::new(snap, self.priority_of(snap.id)))
            .collect();
        let chosen = fill_budget(candidates, self.budget);
        let current = SnapshotFrame::new(tick, last_input_seq, chosen);
        let delta = diff(&self.baseline, &current);
        self.baseline = current;
        delta
    }

    /// The client's own entity is [`Priority::SelfPlayer`]; every other visible
    /// entity is [`Priority::Near`]. Combat/Far banding needs threat and distance
    /// context and lands in a later slice — until then the near-set shares a tier.
    fn priority_of(&self, id: u64) -> Priority {
        if id == self.self_id {
            Priority::SelfPlayer
        } else {
            Priority::Near
        }
    }
}

/// A snapshot delta could not be encoded for the wire.
#[derive(Debug, thiserror::Error)]
pub enum ReplicateError {
    /// The delta frame failed to serialize. Internal detail only — the client
    /// simply receives no snapshot this tick, never this message.
    #[error("failed to encode snapshot delta")]
    Encode(#[from] serde_json::Error),
}

/// Encode a delta into the wire [`ServerMsg::Snapshot`].
///
/// Protocol carries the netcode frame opaquely — it never depends on
/// `omm-netcode` — so the shard serializes it here and the client deserializes
/// the mirror. The frame's `tick` is mirrored onto the envelope so the
/// reliability layer can drop a stale snapshot without decoding the payload.
pub fn snapshot_msg(delta: &DeltaFrame) -> Result<ServerMsg, ReplicateError> {
    let bytes = serde_json::to_vec(delta)?;
    Ok(ServerMsg::Snapshot {
        tick: delta.tick,
        delta: bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_ecs_core::Team;
    use omm_protocol::{Intent, Vec3};
    use omm_sim::combat::Actor;
    use std::collections::BTreeMap;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn actor_at(x: f32, z: f32) -> Actor {
        Actor::new(at(x, z), Team(1), 100, 100)
    }

    fn interest(ids: &[u64]) -> BTreeSet<u64> {
        ids.iter().copied().collect()
    }

    #[test]
    fn world_entities_maps_pos_health_and_zero_yaw_when_still() {
        let mut w = World::new();
        let id = w.spawn(actor_at(3.0, 4.0));
        let ents = world_entities(&w);
        assert_eq!(ents.len(), 1);
        assert_eq!(ents[0].id, id.raw());
        assert_eq!(ents[0].pos, at(3.0, 4.0));
        assert_eq!(ents[0].health, 100);
        assert_eq!(ents[0].yaw, 0.0);
    }

    #[test]
    fn world_entities_derives_yaw_from_velocity_heading() {
        let mut w = World::new();
        let id = w.spawn(actor_at(0.0, 0.0));
        // A Move sets velocity, so the egress-derived facing follows the heading.
        w.step(
            &[(id, Intent::Move { dir: at(5.0, 0.0) })],
            &BTreeMap::new(),
        );
        let vel = w.get(id).unwrap().vel.0;
        let ents = world_entities(&w);
        assert_eq!(ents[0].yaw, vel.x.atan2(vel.z));
        assert_ne!(ents[0].yaw, 0.0, "a moving actor faces a non-zero heading");
    }

    #[test]
    fn world_entities_empty_world_is_empty() {
        assert!(world_entities(&World::new()).is_empty());
    }

    #[test]
    fn first_delta_is_a_full_frame() {
        let mut w = World::new();
        let me = w.spawn(actor_at(0.0, 0.0));
        let other = w.spawn(actor_at(1.0, 0.0));
        let all = world_entities(&w);
        let mut client = ClientReplication::new(me, DEFAULT_BUDGET_BYTES);
        let d = client.delta_for(Tick(1), 0, &all, &interest(&[me.raw(), other.raw()]));
        // Empty baseline → everything visible is "changed", nothing removed.
        assert_eq!(d.changed.len(), 2);
        assert!(d.removed.is_empty());
        assert_eq!(d.tick, Tick(1));
        assert_eq!(client.baseline_tick(), Tick(1));
    }

    #[test]
    fn unchanged_second_delta_is_empty() {
        let mut w = World::new();
        let me = w.spawn(actor_at(0.0, 0.0));
        let all = world_entities(&w);
        let ids = interest(&[me.raw()]);
        let mut client = ClientReplication::new(me, DEFAULT_BUDGET_BYTES);
        let _ = client.delta_for(Tick(1), 0, &all, &ids);
        let d2 = client.delta_for(Tick(2), 0, &all, &ids);
        assert!(d2.changed.is_empty());
        assert!(d2.removed.is_empty());
        assert_eq!(d2.baseline_tick, Tick(1), "delta references the last frame");
    }

    #[test]
    fn delta_excludes_entities_outside_interest() {
        let mut w = World::new();
        let me = w.spawn(actor_at(0.0, 0.0));
        let far = w.spawn(actor_at(500.0, 500.0));
        let all = world_entities(&w);
        let mut client = ClientReplication::new(me, DEFAULT_BUDGET_BYTES);
        // Interest holds only self; the far entity must never appear.
        let d = client.delta_for(Tick(1), 0, &all, &interest(&[me.raw()]));
        let ids: Vec<u64> = d.changed.iter().map(|e| e.id).collect();
        assert_eq!(ids, vec![me.raw()]);
        assert!(!ids.contains(&far.raw()));
    }

    #[test]
    fn self_is_prioritized_under_budget_pressure() {
        let mut w = World::new();
        // Spawn others first so self gets a higher raw id — proving priority, not
        // id order, decides who makes the cut under pressure.
        let other_a = w.spawn(actor_at(1.0, 0.0));
        let other_b = w.spawn(actor_at(2.0, 0.0));
        let me = w.spawn(actor_at(0.0, 0.0));
        let all = world_entities(&w);
        // Budget for exactly one entity.
        let mut client = ClientReplication::new(me, EntitySnapshot::WIRE_SIZE);
        let ids = interest(&[me.raw(), other_a.raw(), other_b.raw()]);
        let d = client.delta_for(Tick(1), 0, &all, &ids);
        assert_eq!(d.changed.len(), 1);
        assert_eq!(d.changed[0].id, me.raw(), "self outranks higher-id others");
    }

    #[test]
    fn snapshot_msg_encodes_a_decodable_delta() {
        let mut w = World::new();
        let me = w.spawn(actor_at(0.0, 0.0));
        let all = world_entities(&w);
        let mut client = ClientReplication::new(me, DEFAULT_BUDGET_BYTES);
        let delta = client.delta_for(Tick(7), 3, &all, &interest(&[me.raw()]));
        let msg = snapshot_msg(&delta).unwrap();
        match msg {
            ServerMsg::Snapshot { tick, delta: bytes } => {
                assert_eq!(tick, Tick(7));
                // The bytes decode straight back to the netcode frame — the
                // opaque-payload contract holds end to end.
                let back: DeltaFrame = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(back, delta);
            }
            other => panic!("expected Snapshot, got {other:?}"),
        }
    }
}
