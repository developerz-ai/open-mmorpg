//! Pure spatial-audio math: **distance attenuation** and **stereo pan** from a
//! listener pose and an emitter position.
//!
//! This is the headless, deterministic core the mixer sits on top of — no audio
//! device, no ECS. The same function the headful client feeds into `bevy_audio`
//! can be re-run server-side or in a replay to reason about what a player could
//! hear. Every attenuation curve is normalised to reach exactly `1.0` at
//! `min_distance` and exactly `0.0` at `max_distance`, so an emitter dropped at
//! the [`aoi`](crate::aoi) boundary fades out cleanly instead of popping.
//!
//! → `docs/specs/game-engine/audio/README.md`.

use bevy_math::Vec3;
use bevy_reflect::Reflect;

use crate::error::AudioError;

/// Coincidence epsilon: a listener/emitter pair closer than this is treated as
/// co-located — centred pan, full gain.
const EPS: f32 = 1e-6;

/// How loudness falls off with distance between [`Attenuation::min_distance`] and
/// [`Attenuation::max_distance`]. Each curve is remapped to `1.0` at the near edge
/// and `0.0` at the far edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum Rolloff {
    /// Straight-line fade — cheapest, least physical.
    Linear,
    /// Inverse-distance (`1/d`) fade. The default; a reasonable approximation of
    /// real acoustic falloff.
    #[default]
    Inverse,
    /// Inverse-square (`1/d²`) fade — steeper near the listener, for tight,
    /// point-like sources.
    InverseSquare,
}

/// Distance-attenuation curve for an emitter: full volume within `min_distance`,
/// silent beyond `max_distance`, and a [`Rolloff`]-shaped fade between.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Attenuation {
    /// Distance (world units) within which the emitter plays at full gain.
    pub min_distance: f32,
    /// Distance (world units) at or beyond which the emitter is silent.
    pub max_distance: f32,
    /// The falloff curve between the two.
    pub rolloff: Rolloff,
}

impl Default for Attenuation {
    fn default() -> Self {
        Self {
            min_distance: 1.0,
            max_distance: 50.0,
            rolloff: Rolloff::Inverse,
        }
    }
}

impl Attenuation {
    /// Build a validated curve. Fails loud unless both distances are finite,
    /// `min_distance` is non-negative, and `min_distance < max_distance` — an
    /// inverted range would attenuate every emitter incorrectly.
    pub fn new(min_distance: f32, max_distance: f32, rolloff: Rolloff) -> Result<Self, AudioError> {
        if !min_distance.is_finite() || !max_distance.is_finite() {
            return Err(AudioError::invalid("attenuation distances must be finite"));
        }
        if min_distance < 0.0 {
            return Err(AudioError::invalid(
                "attenuation min_distance must be non-negative",
            ));
        }
        if min_distance >= max_distance {
            return Err(AudioError::invalid(
                "attenuation min_distance must be < max_distance",
            ));
        }
        Ok(Self {
            min_distance,
            max_distance,
            rolloff,
        })
    }

    /// Gain in `0.0..=1.0` for an emitter at `distance` from the listener. `1.0`
    /// within `min_distance`, `0.0` at/after `max_distance` (and for a non-finite
    /// distance), monotonically decreasing between per the [`Rolloff`].
    #[must_use]
    pub fn gain(&self, distance: f32) -> f32 {
        if !distance.is_finite() {
            return 0.0;
        }
        if distance <= self.min_distance {
            return 1.0;
        }
        if distance >= self.max_distance {
            return 0.0;
        }
        match self.rolloff {
            Rolloff::Linear => {
                (self.max_distance - distance) / (self.max_distance - self.min_distance)
            }
            Rolloff::Inverse => remap(
                self.min_distance / distance,
                self.min_distance / self.max_distance,
            ),
            Rolloff::InverseSquare => {
                let raw = self.min_distance / distance;
                let edge = self.min_distance / self.max_distance;
                remap(raw * raw, edge * edge)
            }
        }
    }
}

/// Remap a raw falloff value — `1.0` at the near edge, `edge` at the far edge —
/// onto a clean `1.0..=0.0` ramp so gain is exactly zero at `max_distance`.
/// `edge < 1.0` always (since `min_distance < max_distance`), so the denominator
/// is never zero.
fn remap(raw: f32, edge: f32) -> f32 {
    ((raw - edge) / (1.0 - edge)).clamp(0.0, 1.0)
}

/// A listener pose: where the ears are and which way they face. Built from a
/// camera/player transform via [`Listener::new`], which normalises the basis and
/// rejects a degenerate one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Listener {
    position: Vec3,
    right: Vec3,
}

impl Listener {
    /// Build a listener at `position` facing `forward` with `up` overhead. Fails
    /// loud on a non-finite input or a degenerate basis (zero or parallel
    /// `forward`/`up`), which would make pan undefined.
    pub fn new(position: Vec3, forward: Vec3, up: Vec3) -> Result<Self, AudioError> {
        if !position.is_finite() || !forward.is_finite() || !up.is_finite() {
            return Err(AudioError::invalid("listener pose must be finite"));
        }
        let f = forward.normalize_or_zero();
        let u = up.normalize_or_zero();
        if f == Vec3::ZERO || u == Vec3::ZERO {
            return Err(AudioError::invalid("listener forward/up must be non-zero"));
        }
        // Right-handed basis (bevy convention: forward × up = right).
        let right = f.cross(u).normalize_or_zero();
        if right == Vec3::ZERO {
            return Err(AudioError::invalid(
                "listener forward and up must not be parallel",
            ));
        }
        Ok(Self { position, right })
    }

    /// The listener's world position.
    #[must_use]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// The listener's unit right vector — the pan axis.
    #[must_use]
    pub fn right(&self) -> Vec3 {
        self.right
    }
}

/// The per-emitter mix parameters the mixer applies: [`gain`](Self::gain) scales
/// amplitude, [`pan`](Self::pan) places the source in the stereo field, and
/// [`distance`](Self::distance) is the raw listener→emitter distance used to order
/// the voice budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialMix {
    /// Amplitude scale, `0.0` (silent) .. `1.0` (full).
    pub gain: f32,
    /// Stereo placement, `-1.0` (left) .. `0.0` (centre) .. `1.0` (right).
    pub pan: f32,
    /// Listener→emitter distance in world units (`>= 0`).
    pub distance: f32,
}

/// Compute the [`SpatialMix`] for an emitter at `emitter` relative to `listener`,
/// using `attenuation` for gain. Pan is the emitter direction projected onto the
/// listener's right axis; a co-located or non-finite emitter is centred and
/// silent. Pure and deterministic.
#[must_use]
pub fn spatialize(listener: &Listener, emitter: Vec3, attenuation: &Attenuation) -> SpatialMix {
    let to = emitter - listener.position;
    let distance = to.length();
    if !distance.is_finite() {
        return SpatialMix {
            gain: 0.0,
            pan: 0.0,
            distance: f32::INFINITY,
        };
    }
    let pan = if distance <= EPS {
        0.0
    } else {
        (to / distance).dot(listener.right).clamp(-1.0, 1.0)
    };
    SpatialMix {
        gain: attenuation.gain(distance),
        pan,
        distance,
    }
}

#[cfg(test)]
mod tests;
