//! The per-tick snapshot: the authoritative state a shard sends its clients.
//!
//! A [`SnapshotFrame`] is the full replicated set for one client at one tick;
//! the [`crate::delta`] encoder turns two frames into a compact diff. Entity ids
//! are raw `u64` on the wire (the shard maps its `EntityId` to/from this at the
//! socket edge) so this crate stays independent of the ECS.

use omm_protocol::{Tick, Vec3};
use serde::{Deserialize, Serialize};

/// One entity's replicated state in a snapshot. Kept small: position, facing,
/// and health are what a nearby client needs to render and predict.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Server-issued entity id (raw).
    pub id: u64,
    /// World position.
    pub pos: Vec3,
    /// Facing angle in radians about the vertical axis.
    pub yaw: f32,
    /// Current hit points.
    pub health: u32,
}

impl EntitySnapshot {
    /// Bytes this entity costs on the wire once quantized: id(8) + pos(3×4,
    /// mm-quantized `i32`) + yaw(2, packed `i16`) + health(4). Used by the
    /// [`crate::priority`] budget filler without actually serializing.
    pub const WIRE_SIZE: usize = 8 + 12 + 2 + 4;

    /// Whether two snapshots of the same entity carry identical replicated state
    /// (so the delta encoder can omit an unchanged entity).
    #[must_use]
    pub fn state_eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.pos == other.pos
            && self.yaw.to_bits() == other.yaw.to_bits()
            && self.health == other.health
    }
}

/// A full replicated frame for one client at one tick. Entities are kept sorted
/// by id so diffs and iteration are deterministic (replay/anti-cheat re-sim).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SnapshotFrame {
    /// The tick that produced this frame — every snapshot carries its tick so
    /// the client's interpolation and the server's re-sim line up.
    pub tick: Tick,
    /// The last client input sequence the server had processed at this tick —
    /// the client reconciles by replaying its unacked inputs past this.
    pub last_input_seq: u32,
    /// The replicated entities, sorted ascending by [`EntitySnapshot::id`].
    pub entities: Vec<EntitySnapshot>,
}

impl SnapshotFrame {
    /// Build a frame from an unordered entity set, sorting for determinism.
    #[must_use]
    pub fn new(tick: Tick, last_input_seq: u32, mut entities: Vec<EntitySnapshot>) -> Self {
        entities.sort_by_key(|e| e.id);
        Self {
            tick,
            last_input_seq,
            entities,
        }
    }

    /// Look up an entity by id (frames are sorted, so this binary-searches).
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&EntitySnapshot> {
        self.entities
            .binary_search_by_key(&id, |e| e.id)
            .ok()
            .map(|i| &self.entities[i])
    }

    /// Total wire size of the frame's entities, for bandwidth accounting.
    #[must_use]
    pub fn wire_size(&self) -> usize {
        self.entities.len() * EntitySnapshot::WIRE_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(id: u64, x: f32, hp: u32) -> EntitySnapshot {
        EntitySnapshot {
            id,
            pos: Vec3 { x, y: 0.0, z: 0.0 },
            yaw: 0.0,
            health: hp,
        }
    }

    #[test]
    fn new_sorts_entities_by_id() {
        let f = SnapshotFrame::new(Tick(1), 0, vec![snap(3, 0.0, 1), snap(1, 0.0, 1)]);
        assert_eq!(f.entities.iter().map(|e| e.id).collect::<Vec<_>>(), [1, 3]);
    }

    #[test]
    fn get_binary_searches() {
        let f = SnapshotFrame::new(Tick(1), 0, vec![snap(1, 0.0, 1), snap(9, 0.0, 1)]);
        assert_eq!(f.get(9).map(|e| e.id), Some(9));
        assert!(f.get(4).is_none());
    }

    #[test]
    fn state_eq_detects_change() {
        assert!(snap(1, 5.0, 100).state_eq(&snap(1, 5.0, 100)));
        assert!(!snap(1, 5.0, 100).state_eq(&snap(1, 5.0, 99)));
        assert!(!snap(1, 5.0, 100).state_eq(&snap(1, 6.0, 100)));
    }
}
