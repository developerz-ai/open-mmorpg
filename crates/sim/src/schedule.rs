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
//!    order lives in one named constant guarded by a test, not implicit in the
//!    body of [`World::step`].
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
//! - **IngestIntents** routes the id-sorted input batch: each entity's single
//!   intent is dispatched to the system that applies it.
//! - **Movement** turns `Move` intents into velocity and integrates every actor
//!   one fixed tick, server-authoritative — the client asks, the server places.
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
/// The enum is the machine-readable form of that contract: [`World::step`] runs
/// exactly these systems in exactly this sequence, and a test pins the order so a
/// careless reordering fails the build rather than silently breaking replay.
///
/// [`World::step`]: crate::World::step
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum System {
    /// Route the id-sorted input batch to the systems that apply each intent.
    IngestIntents,
    /// `Move` intents → velocity, then integrate every actor one fixed tick.
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
    /// contract. [`World::step`](crate::World::step) runs exactly these, in
    /// exactly this sequence, every tick.
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
