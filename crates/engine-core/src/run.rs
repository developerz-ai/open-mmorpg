//! Headless run-N-ticks-and-exit runner.
//!
//! The rendered client runs an open-ended event loop; CI, tests and AI agents
//! instead want to advance the simulation a *known* number of fixed ticks and
//! then inspect the world. [`run_ticks`] does exactly that: because
//! [`headless_app`](crate::headless_app) pins a manual time step equal to the
//! fixed timestep, each `App::update()` runs exactly one `FixedUpdate`. The very
//! first update only establishes the clock baseline (zero delta, no fixed step),
//! so the runner primes it once up front — after that, `n` updates == `n`
//! deterministic simulation ticks.

use bevy_app::prelude::*;
use bevy_app::PluginsState;

/// Advance a headless [`App`] by exactly `ticks` fixed simulation steps, then
/// return — the run-N-ticks-and-exit runner.
///
/// Finalizes plugins on first use (mirrors `App::run`), so it is safe to call on
/// a freshly built [`headless_app`](crate::headless_app) and safe to call again
/// to continue from where a previous call left off.
pub fn run_ticks(app: &mut App, ticks: u64) {
    finalize(app);
    for _ in 0..ticks {
        app.update();
    }
}

/// Build a [`headless_app`](crate::headless_app), advance it `ticks` fixed steps,
/// and return it for inspection. Convenience for tests and agent harnesses.
pub fn headless_run(ticks: u64) -> App {
    let mut app = crate::headless_app();
    run_ticks(&mut app, ticks);
    app
}

/// Run plugin `finish`/`cleanup` once and prime the clock, matching what
/// `App::run` does before its first useful update. Idempotent: a second call
/// (e.g. a follow-up `run_ticks`) is a no-op. The engine's headless plugins are
/// synchronous, so plugins are `Ready` immediately and no busy-wait is needed.
fn finalize(app: &mut App) {
    if app.plugins_state() == PluginsState::Cleaned {
        return;
    }
    app.finish();
    app.cleanup();
    // bevy_time's first update establishes the clock baseline and reports zero
    // delta, so it runs no fixed step. Prime it once here so every subsequent
    // update() maps to exactly one deterministic fixed tick.
    app.update();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FixedTick;

    #[test]
    fn run_ticks_advances_one_fixed_tick_per_update() {
        let app = headless_run(10);
        assert_eq!(app.world().resource::<FixedTick>().0, 10);
    }

    #[test]
    fn zero_ticks_is_a_noop() {
        let app = headless_run(0);
        assert_eq!(app.world().resource::<FixedTick>().0, 0);
    }

    #[test]
    fn run_is_deterministic() {
        // Same tick count in, same state out — the headless == headful contract.
        let a = headless_run(64);
        let b = headless_run(64);
        assert_eq!(
            a.world().resource::<FixedTick>().0,
            b.world().resource::<FixedTick>().0
        );
    }

    #[test]
    fn ticks_accumulate_across_calls() {
        let mut app = crate::headless_app();
        run_ticks(&mut app, 3);
        run_ticks(&mut app, 4);
        assert_eq!(app.world().resource::<FixedTick>().0, 7);
    }
}
