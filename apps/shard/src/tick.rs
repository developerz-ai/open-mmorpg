//! Fixed-timestep accumulator — the wall-clock ↔ sim-tick decoupler.
//!
//! The simulation is deterministic only if it advances in fixed [`TICK_DT`]
//! steps, never by however much wall-clock a wake-up happened to span. This
//! accumulator banks real elapsed time and hands back a whole number of fixed
//! steps to run, so `World::step` always sees the same `dt` on every box — the
//! property replay and anti-cheat re-simulation depend on
//! (docs/specs/game-server/tick-loop/README.md).
//!
//! Two invariants stop a slow tick from cascading into a stall:
//! - **Bounded catch-up.** At most [`MAX_CATCHUP_STEPS`] steps come out of one
//!   [`FixedTimestep::advance`], so a hitch can never demand unbounded work —
//!   the "spiral of death" is impossible by construction.
//! - **No blocking.** `advance` is pure arithmetic; it never sleeps or waits.
//!   The async loop owns pacing, this type only counts.

use std::time::Duration;

use omm_ecs_core::TICK_DT;

/// Upper bound on fixed steps produced by a single [`FixedTimestep::advance`].
/// Past this the backlog is dropped rather than simulated, trading a one-off
/// time skip for the guarantee that a shard never blocks catching up.
pub const MAX_CATCHUP_STEPS: u32 = 5;

/// The outcome of banking one wall-clock slice: how many fixed steps to run
/// now, and any sim time discarded because the catch-up clamp was hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Steps {
    /// Fixed steps to run this wake — always `<= MAX_CATCHUP_STEPS`.
    pub count: u32,
    /// Sim time dropped to honour the clamp; zero on a healthy tick. A non-zero
    /// value means the shard fell behind and skipped ahead rather than block.
    pub dropped: Duration,
}

impl Steps {
    /// Whether the catch-up clamp discarded backlog this wake (an overrun).
    #[must_use]
    pub fn overran(&self) -> bool {
        !self.dropped.is_zero()
    }
}

/// Banks real elapsed time and emits whole fixed [`TICK_DT`] steps.
#[derive(Debug, Clone)]
pub struct FixedTimestep {
    /// Unspent wall-clock time carried between wakes. Kept below `step` on exit
    /// (the sub-step remainder), except the clamp drains whole steps from it.
    accumulator: Duration,
    /// One fixed tick as a `Duration`, cached from [`TICK_DT`].
    step: Duration,
}

impl Default for FixedTimestep {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedTimestep {
    /// A fresh accumulator with no banked time, stepping at [`TICK_DT`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            accumulator: Duration::ZERO,
            step: Duration::from_secs_f32(TICK_DT),
        }
    }

    /// One fixed simulation step as a wall-clock [`Duration`].
    #[must_use]
    pub fn period(&self) -> Duration {
        self.step
    }

    /// Bank `elapsed` real time and report the fixed steps to run now.
    ///
    /// Drains one `step` per emitted tick up to [`MAX_CATCHUP_STEPS`]. If a
    /// whole-step backlog remains after the clamp, those steps are dropped
    /// (reported via [`Steps::dropped`]) so the caller never runs unbounded
    /// work — the spiral-of-death guard. The sub-step remainder is always kept,
    /// so the tick phase stays stable across an overrun.
    pub fn advance(&mut self, elapsed: Duration) -> Steps {
        self.accumulator = self.accumulator.saturating_add(elapsed);
        let mut count = 0;
        while count < MAX_CATCHUP_STEPS && self.accumulator >= self.step {
            self.accumulator -= self.step;
            count += 1;
        }
        let dropped = self.drop_backlog();
        Steps { count, dropped }
    }

    /// After the clamp, discard any whole-step backlog still banked, keeping the
    /// sub-step remainder. Returns the time dropped (zero on a healthy tick).
    fn drop_backlog(&mut self) -> Duration {
        if self.accumulator < self.step {
            return Duration::ZERO;
        }
        // `remainder` is strictly below `step`, so the cast is lossless.
        let remainder =
            Duration::from_nanos((self.accumulator.as_nanos() % self.step.as_nanos()) as u64);
        let dropped = self.accumulator - remainder;
        self.accumulator = remainder;
        dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> FixedTimestep {
        FixedTimestep::new()
    }

    #[test]
    fn new_and_default_agree_on_period() {
        assert_eq!(
            FixedTimestep::new().period(),
            FixedTimestep::default().period()
        );
        assert_eq!(ts().period(), Duration::from_secs_f32(TICK_DT));
    }

    #[test]
    fn sub_step_elapsed_yields_no_tick_but_banks_it() {
        let mut t = ts();
        let half = t.period() / 2;
        let out = t.advance(half);
        assert_eq!(out.count, 0);
        assert!(!out.overran());
        // The banked half pushes the next full period to exactly one step.
        assert_eq!(t.advance(t.period()).count, 1);
    }

    #[test]
    fn exact_period_yields_one_step() {
        let mut t = ts();
        let out = t.advance(t.period());
        assert_eq!(out.count, 1);
        assert_eq!(out.dropped, Duration::ZERO);
        assert!(!out.overran());
    }

    #[test]
    fn multiple_periods_catch_up_within_clamp() {
        let mut t = ts();
        let out = t.advance(t.period() * 3);
        assert_eq!(out.count, 3);
        assert!(!out.overran());
    }

    #[test]
    fn backlog_beyond_clamp_is_dropped_and_flagged() {
        let mut t = ts();
        let out = t.advance(t.period() * 50);
        assert_eq!(out.count, MAX_CATCHUP_STEPS);
        assert!(out.overran());
        assert!(
            out.dropped >= t.period(),
            "at least one whole step must be dropped"
        );
    }

    #[test]
    fn phase_survives_an_overrun() {
        let mut t = ts();
        // A huge stall clamps and drops the whole-step backlog...
        assert!(t.advance(t.period() * 50).overran());
        // ...yet the very next full period still yields exactly one clean step,
        // proving the sub-step remainder (not the whole accumulator) was kept.
        let out = t.advance(t.period());
        assert_eq!(out.count, 1);
        assert!(!out.overran());
    }

    #[test]
    fn advance_is_deterministic_across_instances() {
        let elapsed = [
            Duration::from_millis(10),
            Duration::from_millis(40),
            Duration::from_millis(200),
            Duration::from_millis(33),
        ];
        let run = || {
            let mut t = ts();
            elapsed.iter().map(|&e| t.advance(e)).collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }
}
