//! Cascaded Shadow Map (CSM) **cascade split math** — pure, headless, deterministic.
//!
//! Directional shadows cover the whole view frustum, so a single shadow map is
//! either crisp up close or reaches the horizon, never both. CSM slices the view
//! depth range `[near, far]` into up to [`MAX_CASCADES`] slices and renders each
//! into its own map — near slices get resolution, far slices get reach. This
//! module computes *where* the slices fall; the device layer (under the `render`
//! feature) renders them.
//!
//! # Practical split scheme
//! Split distances blend a **logarithmic** distribution (perceptually even —
//! matches how shadow detail should fall off with distance) and a **uniform** one
//! (avoids wasting a cascade on the sliver right in front of the camera), weighted
//! by [`CsmConfig::split_lambda`] ∈ `[0, 1]` — `1.0` all-log, `0.0` all-uniform.
//! This is the Zhang et al. parallel-split scheme every CSM implementation uses.
//! [`CsmConfig::overlap_proportion`] pulls each cascade's near edge back so
//! adjacent cascades overlap, giving the device a band to blend across instead of
//! a hard seam.
//!
//! The computation is pure and total: a validated [`CsmConfig`] in, a
//! [`CascadeSplits`] out — no GPU, no window — so an agent reasons about the shadow
//! cascade layout in a headless harness exactly as the rendered client does at
//! boot. Spec: shadows ship as CSM + contact shadows now, tracking VSM upstream.
//!
//! → `docs/specs/game-engine/rendering/README.md` (Shadows).

use bevy_reflect::{std_traits::ReflectDefault, Reflect};

use crate::error::RenderError;
use crate::tier::RenderTier;

/// Maximum directional shadow cascades. Four is the industry ceiling for CSM: a
/// fifth slice buys no visible quality for its shadow-map memory and extra pass.
pub const MAX_CASCADES: usize = 4;

/// Authored cascade configuration. Pure data — the *input* to
/// [`compute_splits`](CsmConfig::compute_splits) — with no GPU dependency, so it
/// is reflected and tunable from tools/agents. [`Default`] is the native-quality
/// preset (4 cascades, 0.1..1000 m, balanced split, 20 % overlap).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct CsmConfig {
    /// How many cascades to split into, `1..=`[`MAX_CASCADES`].
    pub cascade_count: u8,
    /// View-space near distance where the first cascade begins (metres, `> 0`).
    pub near_distance: f32,
    /// View-space far distance the last cascade reaches (metres, `> near_distance`).
    pub far_distance: f32,
    /// Log↔uniform blend for the split distribution, `0.0..=1.0`. `1.0` = fully
    /// logarithmic (perceptual), `0.0` = fully uniform (linear in depth).
    pub split_lambda: f32,
    /// Fraction each cascade's near edge is pulled back into the previous cascade
    /// for blend bands, `0.0..1.0`. `0.0` = hard seams, `0.2` = a 20 % overlap.
    pub overlap_proportion: f32,
}

impl Default for CsmConfig {
    fn default() -> Self {
        Self {
            cascade_count: 4,
            near_distance: 0.1,
            far_distance: 1000.0,
            split_lambda: 0.5,
            overlap_proportion: 0.2,
        }
    }
}

impl CsmConfig {
    /// The shadow preset for a render [tier](RenderTier). The WebGPU baseline uses
    /// fewer cascades over a shorter range to fit its shadow-map budget; the native
    /// tiers use the full ladder. One config type drives every tier — the tier only
    /// picks the preset, exactly as it does for AA and GI.
    #[must_use]
    pub fn for_tier(tier: RenderTier) -> Self {
        match tier {
            // Native tiers: full cascade ladder to the far plane.
            RenderTier::Ultra | RenderTier::High => Self::default(),
            // WebGPU baseline: half the cascades, shorter shadow reach.
            RenderTier::Web => Self {
                cascade_count: 2,
                far_distance: 300.0,
                ..Self::default()
            },
        }
    }

    /// Fail loud on a nonsensical config rather than emit `NaN` splits or a
    /// degenerate cascade the device would render as garbage.
    ///
    /// # Errors
    /// [`RenderError::InvalidCsmConfig`] naming the first offending parameter:
    /// cascade count outside `1..=`[`MAX_CASCADES`], a non-positive or non-finite
    /// near, a far not strictly beyond near, or a split/overlap outside its range.
    pub fn validate(&self) -> Result<(), RenderError> {
        let fail = |detail: String| Err(RenderError::InvalidCsmConfig { detail });
        let count = self.cascade_count as usize;
        if count == 0 || count > MAX_CASCADES {
            return fail(format!(
                "cascade_count {} out of bounds (1..={MAX_CASCADES})",
                self.cascade_count
            ));
        }
        if !self.near_distance.is_finite() || self.near_distance <= 0.0 {
            return fail(format!(
                "near_distance must be finite and > 0, got {}",
                self.near_distance
            ));
        }
        if !self.far_distance.is_finite() || self.far_distance <= self.near_distance {
            return fail(format!(
                "far_distance must be finite and > near_distance ({}), got {}",
                self.near_distance, self.far_distance
            ));
        }
        if !self.split_lambda.is_finite() || !(0.0..=1.0).contains(&self.split_lambda) {
            return fail(format!(
                "split_lambda must be in 0.0..=1.0, got {}",
                self.split_lambda
            ));
        }
        if !self.overlap_proportion.is_finite() || !(0.0..1.0).contains(&self.overlap_proportion) {
            return fail(format!(
                "overlap_proportion must be in 0.0..1.0, got {}",
                self.overlap_proportion
            ));
        }
        Ok(())
    }

    /// The practical-split far bound for `index` (`1..=cascade_count`): a lambda
    /// blend of the logarithmic and uniform distributions over `[near, far]`.
    fn practical_split(&self, index: usize) -> f32 {
        let ratio = index as f32 / self.cascade_count as f32;
        let logarithmic = self.near_distance * (self.far_distance / self.near_distance).powf(ratio);
        let uniform = self.near_distance + (self.far_distance - self.near_distance) * ratio;
        self.split_lambda * logarithmic + (1.0 - self.split_lambda) * uniform
    }

    /// Compute the per-cascade `[near, far]` depth intervals.
    ///
    /// Far bounds follow the practical split scheme; the last is pinned to exactly
    /// [`far_distance`](Self::far_distance). Each cascade after the first has its
    /// near edge pulled back by [`overlap_proportion`](Self::overlap_proportion) of
    /// the previous far bound, so adjacent cascades overlap for blending. The
    /// returned intervals are strictly ordered and finite.
    ///
    /// # Errors
    /// [`RenderError::InvalidCsmConfig`] if [`validate`](Self::validate) fails.
    pub fn compute_splits(&self) -> Result<CascadeSplits, RenderError> {
        self.validate()?;
        let count = self.cascade_count as usize;
        let mut intervals = [CascadeInterval::default(); MAX_CASCADES];
        let mut previous_far = self.near_distance;
        for (index, slot) in intervals.iter_mut().enumerate().take(count) {
            let far = if index + 1 == count {
                // Pin the last cascade to the exact far plane — no rounding drift.
                self.far_distance
            } else {
                self.practical_split(index + 1)
            };
            let near = if index == 0 {
                self.near_distance
            } else {
                previous_far * (1.0 - self.overlap_proportion)
            };
            *slot = CascadeInterval { near, far };
            previous_far = far;
        }
        Ok(CascadeSplits {
            intervals,
            count: self.cascade_count,
        })
    }
}

/// One cascade's view-space depth coverage, `[near, far]` in metres.
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub struct CascadeInterval {
    /// Nearest depth this cascade covers (metres).
    pub near: f32,
    /// Farthest depth this cascade covers (metres).
    pub far: f32,
}

impl CascadeInterval {
    /// Depth extent (`far - near`) this cascade spans, in metres.
    #[must_use]
    pub fn depth_range(&self) -> f32 {
        self.far - self.near
    }
}

/// Computed cascade split result: up to [`MAX_CASCADES`] ordered intervals. The
/// trailing array slots are unused; iterate via [`intervals`](Self::intervals).
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
pub struct CascadeSplits {
    intervals: [CascadeInterval; MAX_CASCADES],
    count: u8,
}

impl CascadeSplits {
    /// The active cascade intervals, near→far, exactly [`cascade_count`] long.
    ///
    /// [`cascade_count`]: CsmConfig::cascade_count
    #[must_use]
    pub fn intervals(&self) -> &[CascadeInterval] {
        &self.intervals[..self.count as usize]
    }
}

#[cfg(test)]
mod tests;
