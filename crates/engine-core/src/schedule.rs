//! Fixed-timestep constant and the schedule **ordering scaffold**.
//!
//! Bevy runs systems in parallel by data-access signature; explicit ordering is
//! declared only where it matters. This module declares that ordering *once*, as
//! named [system sets], so every plugin slots its systems into a shared,
//! deterministic pipeline without any plugin needing to know about the others.
//!
//! Two schedules, two set families:
//! * [`SimSet`] orders the **deterministic** [`FixedUpdate`] simulation — the
//!   schedule the authoritative server re-runs for replay and anti-cheat. Its
//!   phases mirror the shard tick loop: ingest input → step the sim → finalize
//!   (→ `docs/specs/game-server/tick-loop/README.md`).
//! * [`FrameSet`] orders the per-frame [`Update`] work — input sampling, frame
//!   logic, then render prep. Non-deterministic by design and decoupled from the
//!   fixed timestep, so headless results equal headful results.
//!
//! [system sets]: bevy_ecs::schedule::SystemSet

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

/// Seconds advanced per fixed simulation step — the reciprocal of
/// [`TICK_HZ`](crate::TICK_HZ).
///
/// Kept numerically equal to `omm_ecs_core::TICK_DT` so the engine's
/// `FixedUpdate` steps and the shared sim integrate over the *same* timestep;
/// the `tick_dt_matches_ecs_core` test fails loud if the two ever drift apart.
/// Expressed in `f64` here to match `bevy_time`'s clock, while the shared sim
/// integrates in `f32` — both denote 1/30 s.
pub const TICK_DT: f64 = 1.0 / crate::TICK_HZ;

/// Ordered phases of the deterministic [`FixedUpdate`] simulation.
///
/// Chained `Input → Simulate → PostSim`: each set runs strictly after the
/// previous one, while systems *within* a set still parallelize freely. Mirrors
/// the shard tick loop (ingest → step → snapshot) so the same discipline holds
/// on client and server.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// Ingest external intent/input into ECS state before the sim reads it.
    Input,
    /// Advance deterministic world state — movement, gameplay, physics.
    Simulate,
    /// Finalize the tick: cleanup, bookkeeping, snapshot preparation.
    PostSim,
}

/// Ordered phases of the per-frame [`Update`] schedule.
///
/// Chained `Input → Update → Render`. Per-frame and non-deterministic —
/// decoupled from [`SimSet`] so rendering never perturbs the simulation. A
/// headless build simply has no systems in [`FrameSet::Render`].
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameSet {
    /// Sample raw device input for this frame.
    Input,
    /// Per-frame logic: cameras, interpolation, UI state.
    Update,
    /// Prepare render/present work.
    Render,
}

/// Install the set ordering into both schedules. Called once by
/// [`EngineCorePlugin`](crate::EngineCorePlugin); plugins added on top attach
/// their systems to these sets with `.in_set(..)`.
pub(crate) fn configure(app: &mut App) {
    app.configure_sets(
        FixedUpdate,
        (SimSet::Input, SimSet::Simulate, SimSet::PostSim).chain(),
    );
    app.configure_sets(
        Update,
        (FrameSet::Input, FrameSet::Update, FrameSet::Render).chain(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records the order phases fired in, so ordering is asserted by observed
    /// behaviour rather than by trusting the graph builder.
    #[derive(Resource, Default)]
    struct Order(Vec<u8>);

    fn record<const N: u8>(mut order: ResMut<Order>) {
        order.0.push(N);
    }

    #[test]
    fn sim_sets_run_in_declared_order() {
        let mut app = App::new();
        configure(&mut app);
        app.init_resource::<Order>();
        // Added out of declared order on purpose: the sets — not add-order —
        // must impose Input → Simulate → PostSim.
        app.add_systems(
            FixedUpdate,
            (
                record::<2>.in_set(SimSet::PostSim),
                record::<0>.in_set(SimSet::Input),
                record::<1>.in_set(SimSet::Simulate),
            ),
        );
        app.world_mut().run_schedule(FixedUpdate);
        assert_eq!(app.world().resource::<Order>().0, vec![0, 1, 2]);
    }

    #[test]
    fn frame_sets_run_in_declared_order() {
        let mut app = App::new();
        configure(&mut app);
        app.init_resource::<Order>();
        app.add_systems(
            Update,
            (
                record::<2>.in_set(FrameSet::Render),
                record::<0>.in_set(FrameSet::Input),
                record::<1>.in_set(FrameSet::Update),
            ),
        );
        app.world_mut().run_schedule(Update);
        assert_eq!(app.world().resource::<Order>().0, vec![0, 1, 2]);
    }

    #[test]
    fn tick_dt_is_reciprocal_of_tick_hz() {
        assert!((TICK_DT * crate::TICK_HZ - 1.0).abs() < 1e-12);
        assert!((TICK_DT - 1.0 / 30.0).abs() < 1e-12);
    }

    /// The engine's fixed timestep must equal the shared sim's `TICK_DT`, or a
    /// headless re-simulation would integrate over a different `dt` than the
    /// client — silent desync. Drift here is a compile-and-test failure.
    #[test]
    fn tick_dt_matches_ecs_core() {
        let delta = (TICK_DT - f64::from(omm_ecs_core::TICK_DT)).abs();
        assert!(
            delta < 1e-6,
            "engine TICK_DT ({TICK_DT}) drifted from omm_ecs_core::TICK_DT ({})",
            omm_ecs_core::TICK_DT
        );
    }

    /// The `Time<Fixed>` clock the core installs must advance by exactly
    /// [`TICK_DT`] each fixed step.
    #[test]
    fn installed_fixed_timestep_equals_tick_dt() {
        use bevy_time::{Fixed, Time};
        let app = crate::headless_app();
        let step = app.world().resource::<Time<Fixed>>().timestep();
        // `Time<Fixed>` stores a ns-quantized `Duration`, so compare within a
        // sub-nanosecond epsilon rather than bit-for-bit.
        assert!((step.as_secs_f64() - TICK_DT).abs() < 1e-9);
    }
}
