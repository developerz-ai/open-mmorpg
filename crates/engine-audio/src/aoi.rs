//! AoI-scoped emitter selection and voice budgeting — the audio side of the
//! engine's one relevance filter.
//!
//! # One index, no drift
//! Audio builds no spatial structure of its own. [`AudioAoi::audible_ids`] reuses
//! [`omm_world`]'s quadtree — the exact index the netcode and renderer cull
//! against (CLAUDE.md: one index, no drift) — so a sound is loaded and mixed only
//! for an entity the player can already perceive. A shard's worth of emitters
//! never mixes at once.
//!
//! # Pipeline
//! 1. [`AudioAoi::audible_ids`] — coarse cull: world entity ids within audible
//!    range, straight off the shared quadtree.
//! 2. The caller resolves those ids to transforms (ECS side).
//! 3. [`AudioAoi::mix`] — attenuate each, drop the inaudible, keep the nearest
//!    [`AudioAoi::max_voices`], deterministically ordered.
//!
//! → `docs/specs/game-engine/audio/README.md`.

use bevy_math::Vec3;
use bevy_reflect::Reflect;
use omm_protocol::Vec3 as WorldVec3;
use omm_world::{EntityId, Quadtree};

use crate::error::AudioError;
use crate::spatial::{spatialize, Attenuation, Listener, SpatialMix};

/// An AoI candidate emitter: its world id and current world position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterInput {
    /// World entity id, shared with the spatial index and netcode.
    pub id: EntityId,
    /// World-space position of the emitter.
    pub position: Vec3,
}

/// A selected, mixable voice: the emitter id plus its computed spatial mix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice {
    /// Which emitter this voice plays.
    pub id: EntityId,
    /// The gain/pan/distance to apply.
    pub mix: SpatialMix,
}

/// Audio area-of-interest config: how far sound carries and how many voices the
/// mixer will play at once.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct AudioAoi {
    /// Radius (world units) for the coarse quadtree cull. Keep this `>=` the
    /// emitter attenuation's `max_distance`, or audible emitters are pruned early.
    pub radius: f32,
    /// Hard cap on simultaneously mixed voices — the nearest N survive.
    pub max_voices: usize,
}

impl Default for AudioAoi {
    fn default() -> Self {
        Self {
            radius: 50.0,
            max_voices: 32,
        }
    }
}

impl AudioAoi {
    /// Build a validated config. Fails loud unless `radius` is finite and positive
    /// and `max_voices` is non-zero — a zero budget would mute the world.
    pub fn new(radius: f32, max_voices: usize) -> Result<Self, AudioError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(AudioError::invalid(
                "audio AoI radius must be finite and positive",
            ));
        }
        if max_voices == 0 {
            return Err(AudioError::invalid("audio AoI max_voices must be non-zero"));
        }
        Ok(Self { radius, max_voices })
    }

    /// Build an AoI whose cull radius exactly matches an attenuation's far edge, so
    /// the coarse cull and the per-emitter fade agree on the audible boundary.
    pub fn for_attenuation(
        attenuation: &Attenuation,
        max_voices: usize,
    ) -> Result<Self, AudioError> {
        Self::new(attenuation.max_distance, max_voices)
    }

    /// Coarse cull: world entity ids within [`radius`](Self::radius) of the
    /// listener, straight off the shared world quadtree (sorted by id). `y` is
    /// ignored — the index culls on the ground plane, like every other AoI read.
    #[must_use]
    pub fn audible_ids(&self, world: &Quadtree, listener_pos: Vec3) -> Vec<EntityId> {
        world.interest_set(
            WorldVec3 {
                x: listener_pos.x,
                y: listener_pos.y,
                z: listener_pos.z,
            },
            self.radius,
        )
    }

    /// Final voice set for `candidates`: attenuate each against `listener`, drop
    /// anything inaudible (gain `<= 0`, e.g. past `max_distance` or non-finite),
    /// keep the nearest [`max_voices`](Self::max_voices), ordered by ascending
    /// distance then id.
    ///
    /// Deterministic: distances compare via [`f32::total_cmp`] and ties break on
    /// id, so a client and a re-simulating server pick the same voices.
    #[must_use]
    pub fn mix(
        &self,
        listener: &Listener,
        attenuation: &Attenuation,
        candidates: &[EmitterInput],
    ) -> Vec<Voice> {
        let mut voices: Vec<Voice> = candidates
            .iter()
            .map(|c| Voice {
                id: c.id,
                mix: spatialize(listener, c.position, attenuation),
            })
            .filter(|v| v.mix.gain.is_finite() && v.mix.gain > 0.0)
            .collect();
        voices.sort_by(|a, b| {
            a.mix
                .distance
                .total_cmp(&b.mix.distance)
                .then_with(|| a.id.0.cmp(&b.id.0))
        });
        voices.truncate(self.max_voices);
        voices
    }
}

#[cfg(test)]
mod tests;
