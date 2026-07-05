//! Delta compression against a per-client acked baseline (the Quake3/Source
//! model): send only what changed since the last snapshot a client acknowledged.
//! A dropped packet is never resent — the next delta just references an older
//! baseline. `apply_delta(baseline, diff(baseline, current)) == current` always.

use crate::snapshot::{EntitySnapshot, SnapshotFrame};
use omm_protocol::Tick;
use serde::{Deserialize, Serialize};

/// A compact diff from a baseline frame to a target frame: only the entities
/// that appeared or changed, plus the ids that left the client's view.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DeltaFrame {
    /// Tick of the target (current) frame this delta reconstructs.
    pub tick: Tick,
    /// Tick of the baseline this delta is computed against (the client's last
    /// acked frame); lets the client pick the right baseline to apply onto.
    pub baseline_tick: Tick,
    /// Echoed last processed input sequence (carried through from the target).
    pub last_input_seq: u32,
    /// Entities new or changed versus the baseline, sorted by id.
    pub changed: Vec<EntitySnapshot>,
    /// Ids present in the baseline but gone from the target (left AoI/despawned).
    pub removed: Vec<u64>,
}

/// Compute the delta that turns `baseline` into `current`. Both frames must be
/// id-sorted (they are, via [`SnapshotFrame::new`]); the merge walk is `O(n+m)`.
#[must_use]
pub fn diff(baseline: &SnapshotFrame, current: &SnapshotFrame) -> DeltaFrame {
    let (mut changed, mut removed) = (Vec::new(), Vec::new());
    let (mut i, mut j) = (0, 0);
    let (base, cur) = (&baseline.entities, &current.entities);
    while i < base.len() || j < cur.len() {
        match (base.get(i), cur.get(j)) {
            (Some(b), Some(c)) if b.id == c.id => {
                if !b.state_eq(c) {
                    changed.push(*c);
                }
                i += 1;
                j += 1;
            }
            // Baseline id comes first (or current exhausted) → entity left view.
            (Some(b), None) => {
                removed.push(b.id);
                i += 1;
            }
            (Some(b), Some(c)) if b.id < c.id => {
                removed.push(b.id);
                i += 1;
            }
            // Current id comes first (or baseline exhausted) → new entity.
            (_, Some(c)) => {
                changed.push(*c);
                j += 1;
            }
            (None, None) => break,
        }
    }
    DeltaFrame {
        tick: current.tick,
        baseline_tick: baseline.tick,
        last_input_seq: current.last_input_seq,
        changed,
        removed,
    }
}

/// Reconstruct the target frame by applying `delta` onto `baseline`.
#[must_use]
pub fn apply_delta(baseline: &SnapshotFrame, delta: &DeltaFrame) -> SnapshotFrame {
    let removed = &delta.removed;
    let mut entities: Vec<EntitySnapshot> = baseline
        .entities
        .iter()
        .filter(|e| !removed.contains(&e.id))
        .copied()
        .collect();
    for c in &delta.changed {
        match entities.binary_search_by_key(&c.id, |e| e.id) {
            Ok(idx) => entities[idx] = *c,
            Err(idx) => entities.insert(idx, *c),
        }
    }
    SnapshotFrame {
        tick: delta.tick,
        last_input_seq: delta.last_input_seq,
        entities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_protocol::Vec3;
    use proptest::prelude::*;

    fn snap(id: u64, x: f32, hp: u32) -> EntitySnapshot {
        EntitySnapshot {
            id,
            pos: Vec3 { x, y: 0.0, z: 0.0 },
            yaw: 0.0,
            health: hp,
        }
    }

    #[test]
    fn delta_captures_add_change_remove() {
        let base = SnapshotFrame::new(Tick(1), 0, vec![snap(1, 0.0, 100), snap(2, 0.0, 100)]);
        let cur = SnapshotFrame::new(Tick(2), 7, vec![snap(2, 0.0, 80), snap(3, 5.0, 100)]);
        let d = diff(&base, &cur);
        assert_eq!(d.removed, vec![1]); // 1 left
        assert_eq!(d.changed.iter().map(|e| e.id).collect::<Vec<_>>(), [2, 3]); // 2 changed, 3 added
        assert_eq!(d.last_input_seq, 7);
    }

    #[test]
    fn unchanged_entity_is_omitted() {
        let base = SnapshotFrame::new(Tick(1), 0, vec![snap(1, 0.0, 100)]);
        let cur = SnapshotFrame::new(Tick(2), 0, vec![snap(1, 0.0, 100)]);
        assert!(diff(&base, &cur).changed.is_empty());
    }

    proptest! {
        /// The core invariant: applying the delta reconstructs the target exactly.
        #[test]
        fn apply_of_diff_reconstructs_target(
            base_ids in prop::collection::vec(0u64..8, 0..8),
            cur in prop::collection::vec((0u64..8, 0.0f32..10.0, 0u32..200), 0..8),
        ) {
            // Frames carry unique ids by construction; dedup the arbitrary input.
            let base_unique: std::collections::BTreeMap<u64, EntitySnapshot> =
                base_ids.into_iter().map(|id| (id, snap(id, 1.0, 50))).collect();
            let base = SnapshotFrame::new(Tick(1), 0, base_unique.into_values().collect());
            let cur_unique: std::collections::BTreeMap<u64, EntitySnapshot> =
                cur.into_iter().map(|(id, x, hp)| (id, snap(id, x, hp))).collect();
            let current = SnapshotFrame::new(Tick(2), 3, cur_unique.into_values().collect());
            let d = diff(&base, &current);
            prop_assert_eq!(apply_delta(&base, &d), current);
        }
    }
}
