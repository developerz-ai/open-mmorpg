//! Scene queries — ray casts against shapes and line-of-sight checks.
//!
//! Used for targeting, interaction probes, and **line of sight**: the client's
//! cosmetic equivalent of the server's authoritative collision checks, both
//! derived from the same open glTF/heightmap geometry. Pure and deterministic —
//! ties break by the lowest collider key so a raycast returns the same hit every
//! run, on client and re-simulating server alike.

use bevy_math::Vec3;

use crate::error::PhysicsError;
use crate::penetration::positive_finite;
use crate::shapes::{Aabb3d, Capsule, Collider, Sphere};

/// Coincidence / parallelism epsilon for query math.
const EPS: f32 = 1e-6;

/// A ray with a **unit** direction. Build via [`Ray::new`], which normalises and
/// rejects a zero-length direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    /// Start point.
    pub origin: Vec3,
    /// Unit direction.
    pub dir: Vec3,
}

impl Ray {
    /// Builds a ray, normalising `dir`. Fails loud on a zero-length or non-finite
    /// direction rather than emitting NaNs downstream.
    pub fn new(origin: Vec3, dir: Vec3) -> Result<Self, PhysicsError> {
        let n = dir.normalize_or_zero();
        if !origin.is_finite() || n == Vec3::ZERO {
            return Err(PhysicsError::InvalidQuery(format!(
                "degenerate ray dir {dir:?}"
            )));
        }
        Ok(Self { origin, dir: n })
    }

    /// The point `t` units along the ray.
    #[must_use]
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }
}

/// A ray hit: time-of-impact, contact point, and surface normal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// Distance along the ray to the contact (`>= 0`).
    pub toi: f32,
    /// World-space contact point.
    pub point: Vec3,
    /// Unit surface normal at the contact.
    pub normal: Vec3,
}

/// Ray vs axis-aligned box (slab method). `max_toi` bounds the search length.
#[must_use]
pub fn ray_vs_aabb(ray: &Ray, aabb: &Aabb3d, max_toi: f32) -> Option<RayHit> {
    let inv = ray.dir.recip();
    let t1 = (aabb.min - ray.origin) * inv;
    let t2 = (aabb.max - ray.origin) * inv;
    let entry = t1.min(t2);
    let exit = t1.max(t2);
    let tmin = entry.max_element();
    let tmax = exit.min_element();
    let t = tmin.max(0.0);
    if tmax < t || t > max_toi {
        return None;
    }
    // The entry axis is whichever slab produced `tmin`.
    let normal = if entry.x >= entry.y && entry.x >= entry.z {
        Vec3::X * -ray.dir.x.signum()
    } else if entry.y >= entry.z {
        Vec3::Y * -ray.dir.y.signum()
    } else {
        Vec3::Z * -ray.dir.z.signum()
    };
    Some(RayHit {
        toi: t,
        point: ray.at(t),
        normal,
    })
}

/// Ray vs sphere. Handles an origin inside the sphere (contact at `t = 0`).
#[must_use]
pub fn ray_vs_sphere(ray: &Ray, sphere: &Sphere, max_toi: f32) -> Option<RayHit> {
    if !positive_finite(sphere.radius) {
        return None;
    }
    let m = ray.origin - sphere.center;
    let b = m.dot(ray.dir);
    let c = m.length_squared() - sphere.radius * sphere.radius;
    // Origin outside and pointing away — no hit.
    if c > 0.0 && b > 0.0 {
        return None;
    }
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()).max(0.0);
    if t > max_toi {
        return None;
    }
    let point = ray.at(t);
    Some(RayHit {
        toi: t,
        point,
        normal: (point - sphere.center).normalize_or_zero(),
    })
}

/// Ray vs capsule: the nearest of the two end-cap spheres and the middle
/// cylinder body. Robust to a ray parallel to the spine (body term degenerates,
/// caps still resolve).
#[must_use]
pub fn ray_vs_capsule(ray: &Ray, capsule: &Capsule, max_toi: f32) -> Option<RayHit> {
    if !positive_finite(capsule.radius) {
        return None;
    }
    let mut best = ray_vs_sphere(ray, &Sphere::new(capsule.a, capsule.radius), max_toi);
    best = nearer(
        best,
        ray_vs_sphere(ray, &Sphere::new(capsule.b, capsule.radius), max_toi),
    );
    nearer(best, ray_vs_cylinder_body(ray, capsule, max_toi))
}

/// The hit with the smaller time-of-impact (preferring an existing hit on ties).
fn nearer(best: Option<RayHit>, hit: Option<RayHit>) -> Option<RayHit> {
    match (best, hit) {
        (b, None) => b,
        (None, h) => h,
        (Some(b), Some(h)) => Some(if h.toi < b.toi { h } else { b }),
    }
}

/// Ray vs the finite cylinder between the capsule's spine endpoints (no caps).
fn ray_vs_cylinder_body(ray: &Ray, capsule: &Capsule, max_toi: f32) -> Option<RayHit> {
    let ba = capsule.b - capsule.a;
    let oa = ray.origin - capsule.a;
    let baba = ba.dot(ba);
    if baba <= EPS {
        return None; // degenerate spine → caps cover it
    }
    let bard = ba.dot(ray.dir);
    let baoa = ba.dot(oa);
    let a_coef = baba - bard * bard;
    if a_coef.abs() <= EPS {
        return None; // ray parallel to the spine → caps cover it
    }
    let b_coef = baba * oa.dot(ray.dir) - baoa * bard;
    let c_coef = baba * oa.dot(oa) - baoa * baoa - capsule.radius * capsule.radius * baba;
    let disc = b_coef * b_coef - a_coef * c_coef;
    if disc < 0.0 {
        return None;
    }
    let t = (-b_coef - disc.sqrt()) / a_coef;
    let y = baoa + t * bard; // projection along the spine, scaled by baba
    if t < 0.0 || t > max_toi || y < 0.0 || y > baba {
        return None; // behind, too far, or beyond the cap band
    }
    let point = ray.at(t);
    let axis_point = capsule.a + ba * (y / baba);
    Some(RayHit {
        toi: t,
        point,
        normal: (point - axis_point).normalize_or_zero(),
    })
}

impl Collider {
    /// Cast `ray` against this collider, up to `max_toi`.
    #[must_use]
    pub fn raycast(&self, ray: &Ray, max_toi: f32) -> Option<RayHit> {
        match self {
            Collider::Aabb(a) => ray_vs_aabb(ray, a, max_toi),
            Collider::Sphere(s) => ray_vs_sphere(ray, s, max_toi),
            Collider::Capsule(c) => ray_vs_capsule(ray, c, max_toi),
        }
    }
}

/// The nearest collider hit by `ray` within `max_toi`, with its key. Ties on
/// time-of-impact break by the lowest key for replay-stable output.
#[must_use]
pub fn raycast_nearest(
    ray: &Ray,
    max_toi: f32,
    colliders: &[(u64, Collider)],
) -> Option<(u64, RayHit)> {
    let mut best: Option<(u64, RayHit)> = None;
    for (key, collider) in colliders {
        if let Some(hit) = collider.raycast(ray, max_toi) {
            let better = match best {
                None => true,
                Some((bk, bh)) => hit.toi < bh.toi - EPS || (hit.toi <= bh.toi + EPS && *key < bk),
            };
            if better {
                best = Some((*key, hit));
            }
        }
    }
    best
}

/// Whether the segment `from`–`to` is unobstructed by any collider.
///
/// A zero-length segment is trivially clear. Order-independent: the result is a
/// boolean over the same set regardless of iteration order.
#[must_use]
pub fn line_of_sight(from: Vec3, to: Vec3, colliders: &[(u64, Collider)]) -> bool {
    let seg = to - from;
    let dist = seg.length();
    if dist <= EPS {
        return true;
    }
    let Ok(ray) = Ray::new(from, seg) else {
        return true;
    };
    for (_, collider) in colliders {
        if let Some(hit) = collider.raycast(&ray, dist) {
            if hit.toi < dist - EPS {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests;
