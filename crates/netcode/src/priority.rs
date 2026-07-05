//! Per-client bandwidth budget. Naive replication is ~0.5 Mbit/s/client; AoI and
//! delta cut most of it, and this stage caps the rest: fill a fixed per-tick byte
//! budget by priority (self > combat > near > far). Entities that don't fit are
//! dropped *this* tick and picked up when the budget frees — never starving the
//! high-priority set. Deterministic: ties break by entity id.

use crate::snapshot::EntitySnapshot;

/// Replication priority, highest first. Lower discriminant = sent sooner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// The client's own entity — always highest.
    SelfPlayer,
    /// Entities in active combat with the client.
    Combat,
    /// Nearby entities.
    Near,
    /// Distant entities.
    Far,
}

/// A replication candidate: its state plus how it should be prioritized.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    pub snap: EntitySnapshot,
    pub priority: Priority,
}

impl Candidate {
    /// A candidate at the given priority.
    #[must_use]
    pub const fn new(snap: EntitySnapshot, priority: Priority) -> Self {
        Self { snap, priority }
    }
}

/// Select the entities to send this tick, highest priority first, until the byte
/// `budget` is exhausted. A candidate that doesn't fit is skipped (not a hard
/// stop) so a large low-priority entity can't block smaller ones behind it.
/// Output is sorted by id so it feeds straight into [`crate::snapshot::SnapshotFrame::new`].
#[must_use]
pub fn fill_budget(mut candidates: Vec<Candidate>, budget: usize) -> Vec<EntitySnapshot> {
    // Priority ascending (SelfPlayer first), then id for a stable order.
    candidates.sort_by(|a, b| a.priority.cmp(&b.priority).then(a.snap.id.cmp(&b.snap.id)));
    let mut spent = 0usize;
    let mut out = Vec::new();
    for c in candidates {
        let next = spent + EntitySnapshot::WIRE_SIZE;
        if next <= budget {
            spent = next;
            out.push(c.snap);
        }
    }
    out.sort_by_key(|e| e.id);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use omm_protocol::Vec3;

    fn cand(id: u64, p: Priority) -> Candidate {
        Candidate::new(
            EntitySnapshot {
                id,
                pos: Vec3::default(),
                yaw: 0.0,
                health: 100,
            },
            p,
        )
    }

    #[test]
    fn budget_is_never_exceeded() {
        let cs = vec![
            cand(1, Priority::Far),
            cand(2, Priority::SelfPlayer),
            cand(3, Priority::Near),
        ];
        // Room for exactly two entities.
        let out = fill_budget(cs, EntitySnapshot::WIRE_SIZE * 2);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn highest_priority_wins_under_pressure() {
        let cs = vec![cand(1, Priority::Far), cand(2, Priority::SelfPlayer)];
        // Room for one → the self player must be it.
        let out = fill_budget(cs, EntitySnapshot::WIRE_SIZE);
        assert_eq!(out, vec![cand(2, Priority::SelfPlayer).snap]);
    }

    #[test]
    fn zero_budget_sends_nothing() {
        assert!(fill_budget(vec![cand(1, Priority::SelfPlayer)], 0).is_empty());
    }
}
