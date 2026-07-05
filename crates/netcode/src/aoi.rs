//! Area-of-interest filtering. The world-model quadtree (`omm-world`) answers
//! "which entities are near this viewer"; this stage takes that id set and keeps
//! only those entities in a candidate list, so a client only ever receives its
//! visible set. That makes replication bandwidth `O(nearby)`, not `O(world)` —
//! the reason one shard can hold thousands online while each client sees a few
//! hundred (docs/specs/game-server/world-model).

use crate::snapshot::EntitySnapshot;
use std::collections::BTreeSet;

/// Keep only the entities whose id is in the viewer's `interest` set. Preserves
/// input order (frames stay id-sorted if the input was).
#[must_use]
pub fn filter_by_interest(
    entities: &[EntitySnapshot],
    interest: &BTreeSet<u64>,
) -> Vec<EntitySnapshot> {
    entities
        .iter()
        .filter(|e| interest.contains(&e.id))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_protocol::Vec3;

    fn snap(id: u64) -> EntitySnapshot {
        EntitySnapshot {
            id,
            pos: Vec3::default(),
            yaw: 0.0,
            health: 100,
        }
    }

    #[test]
    fn keeps_only_entities_in_interest() {
        let all = [snap(1), snap(2), snap(3), snap(4)];
        let interest: BTreeSet<u64> = [2, 4].into_iter().collect();
        let out = filter_by_interest(&all, &interest);
        assert_eq!(out.iter().map(|e| e.id).collect::<Vec<_>>(), [2, 4]);
    }

    #[test]
    fn empty_interest_yields_nothing() {
        let all = [snap(1)];
        assert!(filter_by_interest(&all, &BTreeSet::new()).is_empty());
    }
}
