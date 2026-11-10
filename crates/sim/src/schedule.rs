//! The tick schedule — the fixed, ordered set of systems [`World::step`] runs
//! once per tick, and the determinism contract they uphold.
//!
//! # Determinism contract
//!
//! `omm-sim` promises that **the same ordered inputs produce bit-identical state
//! on any machine, every run**. Server-side replay, lockstep validation, and
//! anti-cheat re-simulation all stand on that promise. Three invariants keep it,
//! and this module is where the first is written down:
//!
//! 1. **Fixed system order.** Every tick runs [`System::ORDER`] — the same
//!    systems in the same sequence. Reordering them changes outcomes, so the
//!    order lives in one named constant, pinned by a test. [`World::step`]
//!    iterates it and dispatches each [`System`] through an exhaustive `match`,
//!    so a new variant is a compile error until wired; a `world.rs` test
//!    records the executed systems and asserts they equal [`System::ORDER`].
//! 2. **Deterministic iteration.** Every system walks the world in server-issued
//!    [`EntityId`] order (the world's `BTreeMap`), so no result depends on a hash
//!    seed or insertion order.
//! 3. **No ambient inputs.** No wall-clock, no RNG, no I/O on the tick path — the
//!    only inputs are the [`InputBatch`] and the content ability table.
//!
//! # The order, and why
//!
//! `IngestIntents → Movement → ResolveCasts → TickAuras → Prune`
//!
//! - **IngestIntents** applies the id-sorted input batch's `Move` intents to
//!   actor velocity (absent entities are skipped); nothing moves yet.
//! - **Movement** integrates every actor one fixed tick from the velocity
//!   ingest set, server-authoritative — the client asks, the server places.
//! - **ResolveCasts** resolves `UseAbility` intents through the data-driven cast
//!   pipeline against the freshly-integrated world, so range and target checks
//!   see this tick's committed positions.
//! - **TickAuras** advances periodic auras — due DoTs/HoTs fire, reschedule, and
//!   expired auras drop — after casts, so an aura applied this tick fires next.
//! - **Prune** removes actors reduced to zero health, last, so every earlier
//!   system in the tick still observed them.
//!
//! [`World::step`]: crate::World::step
//! [`InputBatch`]: crate::InputBatch
//! [`EntityId`]: omm_ecs_core::EntityId

/// One system in the fixed tick [`ORDER`](System::ORDER). Variants are declared
/// in execution order; see the [module docs](self) for the determinism contract.
///
/// The enum is the machine-readable form of that contract: [`World::step`]
/// iterates [`System::ORDER`] and dispatches each variant through an exhaustive
/// `match`, so a new system cannot be added without wiring it. Tests pin the
/// order and the executed sequence so a careless reordering fails the build
/// rather than silently breaking replay.
///
/// [`World::step`]: crate::World::step
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum System {
    /// Apply the id-sorted batch's `Move` intents to actor velocity.
    IngestIntents,
    /// Integrate every actor one fixed tick from its velocity.
    Movement,
    /// `UseAbility` intents → the deterministic `cast` pipeline.
    ResolveCasts,
    /// Advance periodic auras: fire due ticks, reschedule, drop expired.
    TickAuras,
    /// Remove actors reduced to zero health from the live set.
    Prune,
}

impl System {
    /// The fixed per-tick execution order — the heart of the determinism
    /// contract. [`World::step`](crate::World::step) iterates this constant, so
    /// it runs exactly these, in exactly this sequence, every tick.
    pub const ORDER: [System; 5] = [
        System::IngestIntents,
        System::Movement,
        System::ResolveCasts,
        System::TickAuras,
        System::Prune,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn order_is_the_documented_contract() {
        assert_eq!(
            System::ORDER,
            [
                System::IngestIntents,
                System::Movement,
                System::ResolveCasts,
                System::TickAuras,
                System::Prune,
            ],
            "reordering the schedule is a breaking change to the determinism contract"
        );
    }

    #[test]
    fn every_system_appears_exactly_once() {
        let mut seen = BTreeSet::new();
        for system in System::ORDER {
            assert!(seen.insert(system), "duplicate system in ORDER: {system:?}");
        }
        assert_eq!(seen.len(), System::ORDER.len());
    }
}
