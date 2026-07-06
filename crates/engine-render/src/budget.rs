//! Frame **budget instrumentation** — frame time and draw count, measured from day
//! one. Pure, headless, deterministic.
//!
//! The renderer earns its AAA look inside a fixed per-frame budget, the same
//! discipline the netcode applies to bandwidth: a frame that blows the frame-time
//! or draw-call ceiling is a bug to catch, not a stutter to ship. LOD and area-of-
//! interest cap *what is ever submitted*; this module measures *what it cost*.
//!
//! [`FrameBudget`] is the authored ceiling and [`FrameSample`] one frame's
//! measurement — both pure data. [`RenderBudget`] is the reflected ECS resource the
//! device layer feeds each frame and that tools, the inspector and agents read to
//! see live frame cost. The aggregation ([`record`](RenderBudget::record)'s moving
//! average and over-budget tally) is deterministic given a sample sequence, so a
//! headless test drives it exactly as the rendered client does.
//!
//! → `docs/specs/game-engine/rendering/README.md` (Budget-driven).

use bevy_ecs::prelude::*;
use bevy_reflect::{std_traits::ReflectDefault, Reflect};

use crate::error::RenderError;

/// The per-frame ceiling. Pure config — reflected and tunable. [`Default`] targets
/// 60 FPS (16.67 ms) with generous native-tier draw and triangle headroom.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct FrameBudget {
    /// Target frame time in milliseconds. A frame exceeding it missed the refresh
    /// deadline. `16.667` ms = 60 FPS.
    pub target_frame_time_ms: f32,
    /// Maximum draw calls submitted per frame before the CPU submission cost is a
    /// concern. GPU-driven indirect submission keeps this flat as scenes grow.
    pub max_draw_calls: u32,
    /// Maximum triangles rasterized per frame. LOD/imposters and AoI keep the
    /// submitted geometry under this ceiling.
    pub max_triangles: u64,
}

impl Default for FrameBudget {
    fn default() -> Self {
        Self::for_fps(60.0)
    }
}

impl FrameBudget {
    /// A budget targeting `fps` frames per second, with default draw/triangle
    /// ceilings. `fps` must be finite and positive; a non-positive value clamps the
    /// target to `0.0`, which [`validate`](Self::validate) then rejects.
    #[must_use]
    pub fn for_fps(fps: f32) -> Self {
        let target_frame_time_ms = if fps > 0.0 { 1000.0 / fps } else { 0.0 };
        Self {
            target_frame_time_ms,
            max_draw_calls: 8_192,
            max_triangles: 20_000_000,
        }
    }

    /// Fail loud on a budget that can never be met (a non-positive frame-time
    /// target, or a zero draw/triangle ceiling).
    ///
    /// # Errors
    /// [`RenderError::InstrumentationAnomaly`] naming the unusable field.
    pub fn validate(&self) -> Result<(), RenderError> {
        let fail = |detail: String| Err(RenderError::InstrumentationAnomaly { detail });
        if !self.target_frame_time_ms.is_finite() || self.target_frame_time_ms <= 0.0 {
            return fail(format!(
                "target_frame_time_ms must be finite and > 0, got {}",
                self.target_frame_time_ms
            ));
        }
        if self.max_draw_calls == 0 {
            return fail("max_draw_calls must be > 0".to_owned());
        }
        if self.max_triangles == 0 {
            return fail("max_triangles must be > 0".to_owned());
        }
        Ok(())
    }

    /// Whether `sample` stayed within every ceiling.
    #[must_use]
    pub fn is_within(&self, sample: &FrameSample) -> bool {
        sample.frame_time_ms <= self.target_frame_time_ms
            && sample.draw_calls <= self.max_draw_calls
            && sample.triangles <= self.max_triangles
    }

    /// Fail loud when `sample` breaches a ceiling, naming the first one over.
    ///
    /// # Errors
    /// [`RenderError::InstrumentationAnomaly`] describing the breach (frame time,
    /// draw calls, or triangles).
    pub fn check(&self, sample: &FrameSample) -> Result<(), RenderError> {
        let fail = |detail: String| Err(RenderError::InstrumentationAnomaly { detail });
        if sample.frame_time_ms > self.target_frame_time_ms {
            return fail(format!(
                "frame_time {} ms over budget {} ms",
                sample.frame_time_ms, self.target_frame_time_ms
            ));
        }
        if sample.draw_calls > self.max_draw_calls {
            return fail(format!(
                "draw_calls {} over budget {}",
                sample.draw_calls, self.max_draw_calls
            ));
        }
        if sample.triangles > self.max_triangles {
            return fail(format!(
                "triangles {} over budget {}",
                sample.triangles, self.max_triangles
            ));
        }
        Ok(())
    }
}

/// One frame's measured cost. Pure data the device layer fills in each frame.
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct FrameSample {
    /// Wall-clock time this frame took, in milliseconds.
    pub frame_time_ms: f32,
    /// Draw calls submitted this frame.
    pub draw_calls: u32,
    /// Triangles rasterized this frame.
    pub triangles: u64,
}

impl FrameSample {
    /// Instantaneous frames-per-second this sample implies (`0.0` for a zero/
    /// non-positive frame time).
    #[must_use]
    pub fn fps(&self) -> f32 {
        if self.frame_time_ms > 0.0 {
            1000.0 / self.frame_time_ms
        } else {
            0.0
        }
    }
}

/// Live frame-budget instrumentation resource. Holds the [`FrameBudget`] ceiling,
/// the most recent [`FrameSample`], an exponential moving average of frame time,
/// and running frame/over-budget tallies. Reflected as a resource so the inspector
/// and MCP editor read live frame cost like any other type; inserted headless-safe
/// by [`EngineRenderPlugin`](crate::EngineRenderPlugin).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Resource, Debug, Default, PartialEq)]
pub struct RenderBudget {
    /// The ceiling every recorded sample is checked against.
    pub budget: FrameBudget,
    /// The most recently recorded frame.
    pub last_sample: FrameSample,
    /// Exponential moving average of frame time (ms), smoothing per-frame jitter.
    pub average_frame_time_ms: f32,
    /// Total frames recorded since the last [`reset`](Self::reset).
    pub frames_recorded: u64,
    /// How many recorded frames breached the budget.
    pub over_budget_frames: u64,
}

impl RenderBudget {
    /// EMA smoothing factor: each sample contributes 10 %, damping spikes while
    /// still tracking sustained regressions.
    const EMA_ALPHA: f32 = 0.1;

    /// A fresh instrumentation resource for `budget`, with no samples recorded.
    #[must_use]
    pub fn new(budget: FrameBudget) -> Self {
        Self {
            budget,
            ..Self::default()
        }
    }

    /// Record one frame: update the last sample and moving average, and tally the
    /// frame (and whether it breached the budget). Deterministic — the same sample
    /// sequence always yields the same aggregates.
    pub fn record(&mut self, sample: FrameSample) {
        self.average_frame_time_ms = if self.frames_recorded == 0 {
            // Seed the average with the first sample so it never lags up from zero.
            sample.frame_time_ms
        } else {
            Self::EMA_ALPHA * sample.frame_time_ms
                + (1.0 - Self::EMA_ALPHA) * self.average_frame_time_ms
        };
        if !self.budget.is_within(&sample) {
            self.over_budget_frames = self.over_budget_frames.saturating_add(1);
        }
        self.last_sample = sample;
        self.frames_recorded = self.frames_recorded.saturating_add(1);
    }

    /// Average FPS implied by the moving-average frame time (`0.0` before any
    /// sample is recorded).
    #[must_use]
    pub fn average_fps(&self) -> f32 {
        if self.average_frame_time_ms > 0.0 {
            1000.0 / self.average_frame_time_ms
        } else {
            0.0
        }
    }

    /// Fraction of recorded frames that breached the budget, in `0.0..=1.0`
    /// (`0.0` before any sample).
    #[must_use]
    pub fn over_budget_ratio(&self) -> f32 {
        if self.frames_recorded == 0 {
            0.0
        } else {
            self.over_budget_frames as f32 / self.frames_recorded as f32
        }
    }

    /// Whether the most recently recorded frame stayed within budget. `true` before
    /// any sample (an empty history is not over budget).
    #[must_use]
    pub fn last_within_budget(&self) -> bool {
        self.frames_recorded == 0 || self.budget.is_within(&self.last_sample)
    }

    /// Clear the recorded history, keeping the configured [`budget`](Self::budget).
    pub fn reset(&mut self) {
        let budget = self.budget;
        *self = Self::new(budget);
    }
}

#[cfg(test)]
mod tests;
