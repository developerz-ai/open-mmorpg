//! Collision shape types — AABB, sphere, capsule.
//!
//! All shapes are plain `f32` geometry with no I/O and no GPU: an agent (or the
//! server, for movement validation) reasons about them in a headless harness
//! exactly as the client does. Every method is deterministic — given the same
//! inputs it returns bit-identical output, the property the sim relies on for
//! replay and anti-cheat re-simulation.
//!
//! This module holds the shape *definitions* and their inherent geometry; the
//! shape-vs-shape *overlap algorithms* (`sphere_vs_aabb`, `capsule_vs_aabb`,
//! closest-point) live in [`crate::penetration`]. The [`Collider`] enum is a
//! reflected ECS component so the MCP editor and agents can author collision
//! volumes as data.

use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_math::Vec3;
use bevy_reflect::Reflect;

use crate::error::PhysicsError;

/// A 3D axis-aligned bounding box (`min <= max` on every axis).
///
/// The world quadtree ([`omm_world`]) partitions only the `x`/`z` ground plane;
/// this box carries the `y` (height) axis the broadphase filters on and the
/// controller resolves against.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Aabb3d {
    /// Corner with the smallest coordinate on every axis.
    pub min: Vec3,
    /// Corner with the largest coordinate on every axis.
    pub max: Vec3,
}

impl Aabb3d {
    /// Builds a box, normalising so `min <= max` on every axis.
    #[must_use]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// Builds a box from a centre and (absolute) half-extents.
    #[must_use]
    pub fn from_center_half(center: Vec3, half: Vec3) -> Self {
        let h = half.abs();
        Self {
            min: center - h,
            max: center + h,
        }
    }

    /// The centre point.
    #[must_use]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Half the size on each axis (always non-negative).
    #[must_use]
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Whether the point lies inside the box (edges inclusive).
    #[must_use]
    pub fn contains_point(&self, p: Vec3) -> bool {
        p.cmpge(self.min).all() && p.cmple(self.max).all()
    }

    /// Whether this box shares any volume with `other` (touching faces count).
    #[must_use]
    pub fn intersects(&self, other: &Aabb3d) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    /// The point on or inside the box nearest to `p` (clamp into the box).
    #[must_use]
    pub fn closest_point(&self, p: Vec3) -> Vec3 {
        p.clamp(self.min, self.max)
    }

    /// A copy grown outward by `margin` on each axis (absolute).
    #[must_use]
    pub fn expanded(&self, margin: Vec3) -> Self {
        let m = margin.abs();
        Self {
            min: self.min - m,
            max: self.max + m,
        }
    }

    /// A copy shifted by `offset`.
    #[must_use]
    pub fn translated(&self, offset: Vec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// The smallest box enclosing both `self` and `other`.
    #[must_use]
    pub fn union(&self, other: &Aabb3d) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

/// A sphere: centre plus radius.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Sphere {
    /// Centre point.
    pub center: Vec3,
    /// Radius (must be positive to collide).
    pub radius: f32,
}

impl Sphere {
    /// Builds a sphere.
    #[must_use]
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// The axis-aligned box enclosing the sphere.
    #[must_use]
    pub fn bounding_aabb(&self) -> Aabb3d {
        Aabb3d::from_center_half(self.center, Vec3::splat(self.radius.max(0.0)))
    }
}

/// A capsule: a segment `a`–`b` (the spine) inflated by `radius`.
///
/// A character capsule is upright, with `a` the lower spine end and `b` the
/// upper; the round caps extend `radius` past each end.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Capsule {
    /// Lower spine endpoint.
    pub a: Vec3,
    /// Upper spine endpoint.
    pub b: Vec3,
    /// Radius of the swept sphere.
    pub radius: f32,
}

impl Capsule {
    /// Builds a capsule.
    #[must_use]
    pub fn new(a: Vec3, b: Vec3, radius: f32) -> Self {
        Self { a, b, radius }
    }

    /// A copy shifted by `offset`.
    #[must_use]
    pub fn translated(&self, offset: Vec3) -> Self {
        Self {
            a: self.a + offset,
            b: self.b + offset,
            radius: self.radius,
        }
    }

    /// The axis-aligned box enclosing the capsule.
    #[must_use]
    pub fn bounding_aabb(&self) -> Aabb3d {
        let r = Vec3::splat(self.radius.max(0.0));
        Aabb3d::new(self.a.min(self.b) - r, self.a.max(self.b) + r)
    }
}

/// A collision volume, authorable as data and reflected for the editor/agents.
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Component)]
#[reflect(Component)]
pub enum Collider {
    /// Axis-aligned box.
    Aabb(Aabb3d),
    /// Sphere.
    Sphere(Sphere),
    /// Capsule.
    Capsule(Capsule),
}

impl Collider {
    /// The axis-aligned box enclosing this collider — the broadphase key.
    #[must_use]
    pub fn bounding_aabb(&self) -> Aabb3d {
        match self {
            Collider::Aabb(a) => *a,
            Collider::Sphere(s) => s.bounding_aabb(),
            Collider::Capsule(c) => c.bounding_aabb(),
        }
    }

    /// A copy shifted by `offset` (used to place a local collider into world
    /// space from its entity's translation).
    #[must_use]
    pub fn translated(&self, offset: Vec3) -> Self {
        match self {
            Collider::Aabb(a) => Collider::Aabb(a.translated(offset)),
            Collider::Sphere(s) => Collider::Sphere(Sphere::new(s.center + offset, s.radius)),
            Collider::Capsule(c) => Collider::Capsule(c.translated(offset)),
        }
    }

    /// Fail loud on an illegal shape (non-finite fields, non-positive radius, or a
    /// degenerate box). Reflection/deserialization can produce invalid data, so
    /// authored colliders are validated on load rather than trusted.
    pub fn validate(&self) -> Result<(), PhysicsError> {
        let ok = match self {
            Collider::Aabb(a) => {
                a.min.is_finite() && a.max.is_finite() && (a.max - a.min).min_element() >= 0.0
            }
            Collider::Sphere(s) => s.center.is_finite() && s.radius.is_finite() && s.radius > 0.0,
            Collider::Capsule(c) => {
                c.a.is_finite() && c.b.is_finite() && c.radius.is_finite() && c.radius > 0.0
            }
        };
        if ok {
            Ok(())
        } else {
            Err(PhysicsError::InvalidShape(format!("{self:?}")))
        }
    }
}

#[cfg(test)]
mod tests;
