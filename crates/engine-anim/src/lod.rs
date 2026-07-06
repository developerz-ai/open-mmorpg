//! Distance-tiered animation **level of detail** — pure, headless selection of
//! how much animation an entity gets by how far it is from the viewer.
//!
//! # Three tiers, one budget
//! An MMO renders a handful of nearby heroes and a city full of distant NPCs; it
//! cannot afford full skeletal blending on all of them. Selection is a pure
//! function of distance against two thresholds ([`spec`]):
//! * **near** → [`AnimTier::Skeletal`]: full skeletal + blend graph + IK.
//! * **mid** → [`AnimTier::Reduced`]: skeletal with a reduced bone budget.
//! * **far** → [`AnimTier::Vat`]: a baked vertex-animation-texture instance —
//!   animated entirely in the shader, no per-character CPU skinning, no blend/IK.
//!
//! # The one hard rule
//! **The local player is never VAT.** The entity the user controls always gets
//! full skeletal fidelity — it needs per-frame blend and IK — regardless of its
//! distance to the camera. [`LodThresholds::select`] enforces this before it
//! even looks at distance.
//!
//! [`spec`]: <../../../docs/specs/game-engine/animation/README.md>

use bevy_reflect::Reflect;

use crate::error::AnimError;

/// Which animation fidelity an entity is rendered at, chosen by distance.
///
/// Ordered cheapest-last: `Skeletal` (full) → `Reduced` → `Vat` (baked).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AnimTier {
    /// Full skeletal animation: blend graph + IK, all bones. Near the camera and
    /// always the local player.
    Skeletal,
    /// Skeletal animation with a reduced bone budget — still CPU/GPU skinned and
    /// graph-driven, but coarser. Mid distance.
    Reduced,
    /// Baked vertex-animation-texture instance: animated in the shader, GPU
    /// instanced, no per-character skinning or blend/IK. Far distance, crowds.
    Vat,
}

impl AnimTier {
    /// Whether this tier evaluates the blend graph (weighted/additive/masked).
    /// VAT plays fixed baked clips, so it does not.
    #[must_use]
    pub const fn runs_graph(self) -> bool {
        matches!(self, Self::Skeletal | Self::Reduced)
    }

    /// Whether this tier runs the IK pass. Only full skeletal does.
    #[must_use]
    pub const fn runs_ik(self) -> bool {
        matches!(self, Self::Skeletal)
    }

    /// Whether this tier deforms a skinned mesh on the CPU/GPU (as opposed to
    /// VAT's shader-only playback).
    #[must_use]
    pub const fn needs_skinning(self) -> bool {
        matches!(self, Self::Skeletal | Self::Reduced)
    }

    /// Whether this tier is the baked vertex-animation-texture path.
    #[must_use]
    pub const fn is_vat(self) -> bool {
        matches!(self, Self::Vat)
    }
}

/// Distance thresholds and bone budget that drive [`AnimTier`] selection.
///
/// Shares the same LOD/AoI distance discipline as the asset streamer: `mid` and
/// `far` are the world-unit radii at which fidelity steps down.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct LodThresholds {
    /// Distance (world units) at or beyond which an entity drops to
    /// [`AnimTier::Reduced`].
    pub mid_distance: f32,
    /// Distance (world units) at or beyond which an entity drops to
    /// [`AnimTier::Vat`].
    pub far_distance: f32,
    /// Bone budget for the [`AnimTier::Reduced`] tier — the skeleton is decimated
    /// to at most this many joints at mid distance.
    pub reduced_bone_budget: u16,
}

impl Default for LodThresholds {
    fn default() -> Self {
        Self {
            mid_distance: 30.0,
            far_distance: 100.0,
            reduced_bone_budget: 24,
        }
    }
}

impl LodThresholds {
    /// Build validated thresholds. Fails loud unless both distances are finite,
    /// non-negative, and `mid_distance < far_distance` — a mis-ordered ladder
    /// would silently misclassify every entity.
    pub fn new(
        mid_distance: f32,
        far_distance: f32,
        reduced_bone_budget: u16,
    ) -> Result<Self, AnimError> {
        if !mid_distance.is_finite() || !far_distance.is_finite() {
            return Err(AnimError::new("LOD distances must be finite"));
        }
        if mid_distance < 0.0 {
            return Err(AnimError::new("LOD mid_distance must be non-negative"));
        }
        if mid_distance >= far_distance {
            return Err(AnimError::new("LOD mid_distance must be < far_distance"));
        }
        Ok(Self {
            mid_distance,
            far_distance,
            reduced_bone_budget,
        })
    }

    /// Select the animation tier for an entity at `distance` from the viewer.
    ///
    /// The local player is **always** [`AnimTier::Skeletal`] — never VAT —
    /// checked before distance. For everyone else the ladder is
    /// `< mid → Skeletal`, `< far → Reduced`, else `Vat`; a non-finite distance
    /// (NaN) falls through to the cheapest tier, never the local player's.
    #[must_use]
    pub fn select(&self, distance: f32, is_local_player: bool) -> AnimTier {
        if is_local_player {
            return AnimTier::Skeletal;
        }
        if distance < self.mid_distance {
            AnimTier::Skeletal
        } else if distance < self.far_distance {
            AnimTier::Reduced
        } else {
            AnimTier::Vat
        }
    }

    /// The joint budget for a tier given the skeleton's `full_bone_count`:
    /// full for `Skeletal`, capped at [`reduced_bone_budget`] for `Reduced`, and
    /// `0` for `Vat` (baked, no live skeleton).
    ///
    /// [`reduced_bone_budget`]: LodThresholds::reduced_bone_budget
    #[must_use]
    pub fn bone_budget(&self, tier: AnimTier, full_bone_count: u16) -> u16 {
        match tier {
            AnimTier::Skeletal => full_bone_count,
            AnimTier::Reduced => full_bone_count.min(self.reduced_bone_budget),
            AnimTier::Vat => 0,
        }
    }
}
