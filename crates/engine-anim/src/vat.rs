//! **Vertex-animation-texture** frame + index math — pure, headless.
//!
//! # What a VAT is
//! For MMO-scale crowds we cannot skin hundreds of skeletons on the CPU. A VAT
//! bakes per-frame vertex positions into a texture: the mesh becomes a static,
//! GPU-instanced object animated entirely in the vertex shader by sampling the
//! texture — no per-character skeleton, blend, or IK. The trade is fixed baked
//! clips, correct for the [far LOD tier](crate::lod::AnimTier::Vat), never the
//! local player. → [animation spec](../../../docs/specs/game-engine/animation/README.md).
//!
//! # What this module owns
//! The GPU baking and shader sampling need a device and are out of scope for
//! headless CI. What *is* pure — and lives here — is the math both sides agree on:
//! * [`VatLayout`] — the row-major texel grid (one texel per vertex×frame): its
//!   dimensions, flat bake index, and sample UVs.
//! * [`VatClip`] + [`VatClip::sample_at`] — which two baked frames to sample at a
//!   playback time and the blend factor between them, under [`PlaybackMode::Loop`],
//!   [`Once`](PlaybackMode::Once), or [`PingPong`](PlaybackMode::PingPong).
//!
//! All of it is deterministic integer/`f32` math with fail-loud validation, so the
//! baker (offline), the shader (device), and any headless test compute identical
//! indices. Non-finite or negative playback times clamp to frame zero rather than
//! producing garbage indices.

use bevy_math::Vec2;
use bevy_reflect::Reflect;

use crate::error::AnimError;

/// How a baked VAT clip advances once playback passes its final frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum PlaybackMode {
    /// Wrap back to frame 0 — the last frame blends into the first. Cyclic gaits.
    #[default]
    Loop,
    /// Clamp at the final frame and hold. One-shot emotes.
    Once,
    /// Bounce forward then backward over the frame range. Subtle idle breathing.
    PingPong,
}

/// The two baked frames plus the blend factor to sample at a playback time.
///
/// Runtime output (like [`Pose`](crate::graph::Pose)), so it is not reflected. The
/// shader samples texture rows `frame_a` and `frame_b` and linearly interpolates by
/// `blend` ∈ `[0, 1)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VatSample {
    /// Lower baked frame index.
    pub frame_a: u32,
    /// Upper baked frame index (wrapped for [`PlaybackMode::Loop`], clamped else).
    pub frame_b: u32,
    /// Interpolation factor from `frame_a` toward `frame_b`, in `[0, 1)`.
    pub blend: f32,
}

/// A baked VAT clip's timing descriptor: how many frames were baked, at what rate,
/// and how playback continues past the end. Authored content, so it is reflected.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct VatClip {
    frame_count: u32,
    fps: f32,
    mode: PlaybackMode,
}

impl VatClip {
    /// Build a validated clip. Fails loud unless at least one frame was baked and
    /// `fps` is finite and positive.
    pub fn new(frame_count: u32, fps: f32, mode: PlaybackMode) -> Result<Self, AnimError> {
        if frame_count == 0 {
            return Err(AnimError::new("VAT clip needs at least 1 frame"));
        }
        if !fps.is_finite() || fps <= 0.0 {
            return Err(AnimError::new("VAT fps must be finite and > 0"));
        }
        Ok(Self {
            frame_count,
            fps,
            mode,
        })
    }

    /// Number of baked frames.
    #[must_use]
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Bake / playback rate in frames per second.
    #[must_use]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// The playback mode.
    #[must_use]
    pub fn mode(&self) -> PlaybackMode {
        self.mode
    }

    /// Forward duration in seconds (`frame_count / fps`). A
    /// [`PingPong`](PlaybackMode::PingPong) round trip is nearly twice this.
    #[must_use]
    pub fn duration_secs(&self) -> f32 {
        self.frame_count as f32 / self.fps
    }

    /// The two baked frames and blend factor to sample at `time_secs`.
    ///
    /// A single-frame clip always samples frame 0. Non-finite or negative times
    /// clamp to the start. The frame ladder honors [`mode`](Self::mode): `Loop`
    /// wraps the upper frame back to 0, `Once` holds the last frame, `PingPong`
    /// mirrors at both ends.
    #[must_use]
    pub fn sample_at(&self, time_secs: f32) -> VatSample {
        let frames = self.frame_count;
        if frames <= 1 {
            return VatSample {
                frame_a: 0,
                frame_b: 0,
                blend: 0.0,
            };
        }
        let last = frames - 1;
        let t = if time_secs.is_finite() && time_secs > 0.0 {
            time_secs
        } else {
            0.0
        };
        let fpos = t * self.fps;

        match self.mode {
            PlaybackMode::Loop => {
                let pos = fpos.rem_euclid(frames as f32); // [0, frames)
                let floor = pos.floor();
                let a = (floor as u32) % frames;
                VatSample {
                    frame_a: a,
                    frame_b: (a + 1) % frames,
                    blend: pos - floor,
                }
            }
            PlaybackMode::Once => split_clamped(fpos.min(last as f32), last),
            PlaybackMode::PingPong => {
                let period = (2 * last) as f32; // 2·(frames − 1)
                let x = fpos.rem_euclid(period);
                let pos = if x <= last as f32 { x } else { period - x };
                split_clamped(pos, last)
            }
        }
    }
}

/// Split a clamped, non-wrapping frame position into `(a, b, blend)` with the upper
/// frame held at `last` — shared by [`PlaybackMode::Once`] and `PingPong`.
fn split_clamped(pos: f32, last: u32) -> VatSample {
    let floor = pos.floor();
    let a = (floor as u32).min(last);
    VatSample {
        frame_a: a,
        frame_b: (a + 1).min(last),
        blend: pos - floor,
    }
}

/// Row-major VAT texture layout: one texel per (vertex, frame), vertices across the
/// width, frames down the height. Authored alongside the baked texture, so it is
/// reflected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct VatLayout {
    vertex_count: u32,
    frame_count: u32,
}

impl VatLayout {
    /// Build a validated layout. Fails loud unless both the vertex and frame counts
    /// are non-zero.
    pub fn new(vertex_count: u32, frame_count: u32) -> Result<Self, AnimError> {
        if vertex_count == 0 {
            return Err(AnimError::new("VAT layout needs at least 1 vertex"));
        }
        if frame_count == 0 {
            return Err(AnimError::new("VAT layout needs at least 1 frame"));
        }
        Ok(Self {
            vertex_count,
            frame_count,
        })
    }

    /// Texture width in texels — one column per vertex.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.vertex_count
    }

    /// Texture height in texels — one row per baked frame.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.frame_count
    }

    /// Number of vertices per frame.
    #[must_use]
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    /// Number of baked frames.
    #[must_use]
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Total texel count (`width × height`), as `u64` since a large crowd bake can
    /// exceed `u32`.
    #[must_use]
    pub fn texel_count(&self) -> u64 {
        u64::from(self.vertex_count) * u64::from(self.frame_count)
    }

    /// Whether both texture dimensions are within the device's max texture size —
    /// checked before a bake so an over-wide mesh fails loud instead of at upload.
    #[must_use]
    pub fn fits(&self, max_texture_size: u32) -> bool {
        self.width() <= max_texture_size && self.height() <= max_texture_size
    }

    /// Flat row-major texel index for `(vertex, frame)` = `frame·width + vertex`.
    /// Fails loud if either coordinate is out of range.
    pub fn texel_index(&self, vertex: u32, frame: u32) -> Result<u64, AnimError> {
        self.check_bounds(vertex, frame)?;
        Ok(u64::from(frame) * u64::from(self.vertex_count) + u64::from(vertex))
    }

    /// Texel-center sample UV for `(vertex, frame)`: `((v+0.5)/w, (f+0.5)/h)`. Fails
    /// loud if either coordinate is out of range.
    pub fn texel_uv(&self, vertex: u32, frame: u32) -> Result<Vec2, AnimError> {
        self.check_bounds(vertex, frame)?;
        Ok(Vec2::new(
            (vertex as f32 + 0.5) / self.width() as f32,
            (frame as f32 + 0.5) / self.height() as f32,
        ))
    }

    /// Row-center V coordinate for `frame`: `(f+0.5)/height`. The shader samples the
    /// two rows from a [`VatSample`] at their `frame_v` and lerps by its `blend`.
    /// Fails loud if `frame` is out of range.
    pub fn frame_v(&self, frame: u32) -> Result<f32, AnimError> {
        if frame >= self.frame_count {
            return Err(AnimError::new("VAT frame index out of range"));
        }
        Ok((frame as f32 + 0.5) / self.height() as f32)
    }

    fn check_bounds(&self, vertex: u32, frame: u32) -> Result<(), AnimError> {
        if vertex >= self.vertex_count {
            return Err(AnimError::new("VAT vertex index out of range"));
        }
        if frame >= self.frame_count {
            return Err(AnimError::new("VAT frame index out of range"));
        }
        Ok(())
    }
}
