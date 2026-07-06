//! Level of detail — pick a mesh tier from an object's on-screen size, drop to an
//! **octahedral imposter** for the far tier, and cull when it is too small to see.
//!
//! Bevy has no built-in LOD/imposter, so this is the app-layer policy the client
//! drives every frame. It is pure and deterministic: screen size in, a
//! [`LodSelection`] out — no GPU, no ECS, so the same call decides detail in a
//! headless test and in the rendered client. → `docs/specs/game-engine/assets/README.md`.
//!
//! # Screen size
//! "Screen size" is the object's projected radius as a fraction of the viewport
//! half-height — bigger (nearer) ⇒ finer tier. Tier `0` is the finest and has the
//! largest threshold; each coarser tier triggers at a strictly smaller size. Below
//! the coarsest mesh tier the object becomes a single-quad octahedral imposter
//! (an atlas of pre-rendered views), and below the cull size it is not drawn.

use core::num::NonZeroU32;

use bevy_math::{Vec2, Vec3};

use crate::error::AssetError;

/// What to draw for a given on-screen size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodSelection {
    /// Draw discrete mesh tier `index` (`0` = finest).
    Mesh(usize),
    /// Draw the octahedral imposter — a single billboarded quad sampling the
    /// pre-rendered view atlas ([`ImposterAtlas`]).
    Imposter,
    /// Too small to be worth drawing — cull entirely.
    Culled,
}

/// A discrete LOD chain: descending screen-size thresholds selecting a mesh tier,
/// an optional imposter far tier, and a cull size.
#[derive(Debug, Clone, PartialEq)]
pub struct LodChain {
    /// Strictly descending, all finite. `thresholds[i]` is the minimum screen size
    /// at which mesh tier `i` is used; `thresholds[0]` is the largest.
    thresholds: Vec<f32>,
    /// Whether an octahedral imposter fills the range below the coarsest mesh tier
    /// down to `cull`. When `false`, the coarsest mesh tier holds until `cull`.
    imposter: bool,
    /// Screen size below which nothing is drawn. In `[0, coarsest_threshold)`.
    cull: f32,
}

impl LodChain {
    /// Build a validated LOD chain.
    ///
    /// # Errors
    /// - [`AssetError::EmptyLodChain`] if `thresholds` is empty.
    /// - [`AssetError::LodNotDescending`] if the thresholds are not finite and
    ///   strictly descending.
    /// - [`AssetError::LodCull`] if `cull` is not finite or not in
    ///   `[0, coarsest_threshold)`.
    pub fn new(thresholds: Vec<f32>, imposter: bool, cull: f32) -> Result<Self, AssetError> {
        let Some(&coarsest) = thresholds.last() else {
            return Err(AssetError::EmptyLodChain);
        };
        for (index, pair) in thresholds.windows(2).enumerate() {
            let [hi, lo] = [pair[0], pair[1]];
            if !hi.is_finite() || !lo.is_finite() || hi <= lo {
                return Err(AssetError::LodNotDescending { index });
            }
        }
        // A single-tier chain skips the loop above, so range-check the sole value.
        if !coarsest.is_finite() {
            return Err(AssetError::LodNotDescending { index: 0 });
        }
        if !cull.is_finite() || cull < 0.0 || cull >= coarsest {
            return Err(AssetError::LodCull {
                cull,
                min_threshold: coarsest,
            });
        }
        Ok(Self {
            thresholds,
            imposter,
            cull,
        })
    }

    /// Select what to draw for `screen_size` (fraction of the viewport half-height;
    /// see [`projected_screen_size`]).
    #[must_use]
    pub fn select(&self, screen_size: f32) -> LodSelection {
        // NaN culls; `-inf` culls via `< cull`; `+inf` (a near/behind-camera object,
        // see `projected_screen_size`) falls through to the finest tier.
        if screen_size.is_nan() || screen_size < self.cull {
            return LodSelection::Culled;
        }
        for (index, &threshold) in self.thresholds.iter().enumerate() {
            if screen_size >= threshold {
                return LodSelection::Mesh(index);
            }
        }
        // Between `cull` and the coarsest mesh tier: imposter if enabled, else the
        // coarsest mesh tier holds.
        if self.imposter {
            LodSelection::Imposter
        } else {
            LodSelection::Mesh(self.thresholds.len() - 1)
        }
    }

    /// Number of discrete mesh tiers.
    #[must_use]
    pub fn tiers(&self) -> usize {
        self.thresholds.len()
    }

    /// Whether this chain drops to an octahedral imposter below the coarsest tier.
    #[must_use]
    pub fn has_imposter(&self) -> bool {
        self.imposter
    }

    /// The cull screen size — below this, [`LodSelection::Culled`].
    #[must_use]
    pub fn cull(&self) -> f32 {
        self.cull
    }
}

/// Project a bounding sphere to a screen size: its radius as a fraction of the
/// viewport half-height, the metric [`LodChain::select`] consumes.
///
/// `tan_half_fov` is `tan(vertical_fov / 2)`. Nearer/bigger ⇒ larger result. An
/// object at or behind the camera (`distance <= 0`) returns [`f32::INFINITY`] so
/// it always resolves to the finest tier rather than a bogus tiny size.
#[must_use]
pub fn projected_screen_size(radius: f32, distance: f32, tan_half_fov: f32) -> f32 {
    if !distance.is_finite() || distance <= 0.0 || tan_half_fov <= 0.0 {
        return f32::INFINITY;
    }
    (radius / (distance * tan_half_fov)).max(0.0)
}

/// One cell in an octahedral imposter atlas — the pre-rendered view to sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImposterCell {
    /// Column, in `0..grid`.
    pub x: u32,
    /// Row, in `0..grid`.
    pub y: u32,
}

/// An octahedral imposter atlas: a `grid × grid` sheet of views baked over the
/// sphere of directions via octahedral mapping. Given the direction from the
/// object to the camera, [`Self::cell_for_view`] picks the view to billboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImposterAtlas {
    grid: NonZeroU32,
}

impl ImposterAtlas {
    /// A `grid × grid` atlas. `grid` is [`NonZeroU32`] so an empty atlas is
    /// unrepresentable — no runtime "grid == 0" check needed.
    #[must_use]
    pub fn new(grid: NonZeroU32) -> Self {
        Self { grid }
    }

    /// Atlas resolution (cells per side).
    #[must_use]
    pub fn grid(&self) -> u32 {
        self.grid.get()
    }

    /// Total number of baked views (`grid²`).
    #[must_use]
    pub fn cell_count(&self) -> u32 {
        self.grid.get().saturating_mul(self.grid.get())
    }

    /// The atlas cell whose baked view best matches looking along `view_dir`
    /// (object→camera). A near-zero direction is degenerate and maps to the
    /// centre cell rather than producing NaN.
    #[must_use]
    pub fn cell_for_view(&self, view_dir: Vec3) -> ImposterCell {
        let grid = self.grid.get();
        let normalized = view_dir.normalize_or_zero();
        if normalized == Vec3::ZERO {
            let mid = grid / 2;
            return ImposterCell { x: mid, y: mid };
        }
        let uv = oct_encode(normalized);
        ImposterCell {
            x: cell_index(uv.x, grid),
            y: cell_index(uv.y, grid),
        }
    }
}

/// Map a UV in `[0, 1]` to a cell index in `0..grid`, clamping the `uv == 1.0`
/// edge back into range.
fn cell_index(uv: f32, grid: u32) -> u32 {
    let scaled = (uv * grid as f32).floor() as i64;
    scaled.clamp(0, i64::from(grid) - 1) as u32
}

/// Octahedral encode a unit vector to `[0, 1]²` — the standard mapping that lays
/// the sphere of view directions flat across the imposter atlas.
fn oct_encode(dir: Vec3) -> Vec2 {
    let n = dir / (dir.x.abs() + dir.y.abs() + dir.z.abs());
    let folded = if n.z >= 0.0 {
        Vec2::new(n.x, n.y)
    } else {
        Vec2::new(
            (1.0 - n.y.abs()) * n.x.signum(),
            (1.0 - n.x.abs()) * n.y.signum(),
        )
    };
    folded * 0.5 + Vec2::splat(0.5)
}

#[cfg(test)]
mod tests;
