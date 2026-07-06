//! [`SceneQuery`] — the single scene-query entry point the client and the
//! server world-model share.
//!
//! Every query takes a plain `&[(u64, Collider)]`, never an ECS `World` or a
//! GPU handle, so the *same* code answers a probe on either side of the wire:
//! the server wraps colliders derived from the open glTF/heightmap VMaps, the
//! client wraps its ECS colliders, and both get bit-identical, replay-stable
//! answers (ties to the lowest key). Callers that maintain a spatial index
//! prune first, then wrap the surviving candidates here for the exact
//! narrowphase — see [`crate::broadphase::Broadphase`].

use bevy_math::Vec3;

use crate::query::{line_of_sight, raycast_nearest, Ray, RayHit};
use crate::shapecast::{sphere_cast_nearest, thick_line_of_sight};
use crate::shapes::Collider;

/// A borrowed, read-only view over a collider set that answers scene queries.
///
/// Zero-copy: holds only the slice reference. Construct one per query batch from
/// whatever collider source the caller owns.
#[derive(Debug, Clone, Copy)]
pub struct SceneQuery<'a> {
    colliders: &'a [(u64, Collider)],
}

impl<'a> SceneQuery<'a> {
    /// Wraps a keyed collider set. Keys are opaque `u64`s (the controller uses
    /// `Entity::to_bits()`; the server can use its own world-object ids).
    #[must_use]
    pub fn new(colliders: &'a [(u64, Collider)]) -> Self {
        Self { colliders }
    }

    /// Number of colliders in view.
    #[must_use]
    pub fn len(&self) -> usize {
        self.colliders.len()
    }

    /// Whether the view holds no colliders.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.colliders.is_empty()
    }

    /// The nearest collider hit by `ray` within `max_toi` — targeting / aim.
    #[must_use]
    pub fn raycast(&self, ray: &Ray, max_toi: f32) -> Option<(u64, RayHit)> {
        raycast_nearest(ray, max_toi, self.colliders)
    }

    /// The nearest collider swept by a sphere of `radius` along `ray` — thick
    /// targeting / aim assist / interaction reach.
    #[must_use]
    pub fn sphere_cast(&self, ray: &Ray, radius: f32, max_toi: f32) -> Option<(u64, RayHit)> {
        sphere_cast_nearest(ray, radius, self.colliders, max_toi)
    }

    /// Whether the segment `from`–`to` is unobstructed (thin line-of-sight).
    #[must_use]
    pub fn line_of_sight(&self, from: Vec3, to: Vec3) -> bool {
        line_of_sight(from, to, self.colliders)
    }

    /// Whether a sphere of `radius` swept `from`–`to` is unobstructed — line-of-
    /// sight with width (cover / projectile clearance).
    #[must_use]
    pub fn thick_line_of_sight(&self, from: Vec3, to: Vec3, radius: f32) -> bool {
        thick_line_of_sight(from, to, radius, self.colliders)
    }
}

#[cfg(test)]
mod tests;
