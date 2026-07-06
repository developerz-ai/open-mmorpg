//! Swept-shape scene queries — sphere casts (a "thick ray") for line-of-sight,
//! targeting, and interaction / projectile-clearance probes.
//!
//! A sphere cast is the shape query the physics spec names: the same
//! deterministic geometry the client uses to *preview* a probe is reused by the
//! server world-model to *validate* it, both derived from the open
//! glTF/heightmap source. Every cast reduces to the exact ray kernels in
//! [`crate::query`] by inflating the target by the probe radius (a Minkowski
//! sum), so results are bit-identical on client and re-simulating server, with
//! ties broken by the lowest collider key ([`crate::query::better_keyed_hit`]).
//!
//! **Scope.** Casting a *box* or *capsule* as the moving shape is intentionally
//! omitted: character movement is resolved by [`crate::slide::move_and_slide`]
//! (penetration, not casts), and LOS / targeting / interaction all use a sphere
//! cast — a ray is its zero-radius limit. One exact caster beats the approximate
//! box/capsule sweeps a general solver would need, matching the crate's
//! deterministic-first contract.

use bevy_math::Vec3;

use crate::penetration::positive_finite;
use crate::query::{better_keyed_hit, nearer, ray_vs_aabb, ray_vs_capsule, ray_vs_sphere};
use crate::query::{Ray, RayHit};
use crate::shapes::{Aabb3d, Capsule, Collider, Sphere};

/// Coincidence epsilon, matching [`crate::query`].
const EPS: f32 = 1e-6;

/// Sweep a sphere of `radius`, centred at `ray.origin`, along `ray.dir` up to
/// `max_toi`, against one collider. Exact for every shape.
///
/// Returns the contact on the **target** surface and the outward target normal,
/// or `None` on a miss. A non-positive (or non-finite) `radius` degenerates to
/// [`Collider::raycast`] — a ray is a zero-radius sphere cast.
#[must_use]
pub fn sphere_cast(ray: &Ray, radius: f32, target: &Collider, max_toi: f32) -> Option<RayHit> {
    if !positive_finite(radius) {
        return target.raycast(ray, max_toi);
    }
    // First contact of the moving centre is a raycast against the target grown by
    // `radius` (its configuration-space obstacle).
    let inflated = match target {
        Collider::Sphere(s) => {
            ray_vs_sphere(ray, &Sphere::new(s.center, s.radius + radius), max_toi)
        }
        Collider::Capsule(c) => {
            ray_vs_capsule(ray, &Capsule::new(c.a, c.b, c.radius + radius), max_toi)
        }
        Collider::Aabb(a) => ray_vs_rounded_aabb(ray, a, radius, max_toi),
    }?;
    // The reduction hits the inflated surface; the true contact lies one radius
    // back along the (parallel) surface normal.
    Some(RayHit {
        toi: inflated.toi,
        point: inflated.point - inflated.normal * radius,
        normal: inflated.normal,
    })
}

/// Ray vs the box grown by `r` into a rounded box (the sphere cast's
/// configuration-space obstacle). Returns the hit on the inflated surface.
///
/// Exact: a flat-face contact resolves on the expanded slab; an edge or corner
/// contact refines against the edge capsule(s) the entry point falls into — the
/// caps of the edge capsules cover the corner sphere (Ericson, *RTCD* §5.5.7).
fn ray_vs_rounded_aabb(ray: &Ray, aabb: &Aabb3d, r: f32, max_toi: f32) -> Option<RayHit> {
    let hit = ray_vs_aabb(ray, &aabb.expanded(Vec3::splat(r)), max_toi)?;
    let (p, mn, mx) = (
        hit.point.to_array(),
        aabb.min.to_array(),
        aabb.max.to_array(),
    );
    // Bit i set = the entry point sits beyond the *original* box on axis i.
    let mut mask = 0u8;
    for i in 0..3 {
        if p[i] < mn[i] - EPS || p[i] > mx[i] + EPS {
            mask |= 1 << i;
        }
    }
    // Zero or one axis outside → a genuine flat-face contact; the slab hit stands.
    if mask.count_ones() <= 1 {
        return Some(hit);
    }
    let vtx = hit.point.clamp(aabb.min, aabb.max); // nearest box feature point
    if mask == 0b111 {
        // Corner region: the three edges meeting at the vertex; nearest wins.
        (0..3).fold(None, |best, axis| {
            nearer(
                best,
                ray_vs_capsule(
                    ray,
                    &Capsule::new(vtx, flip_axis(vtx, aabb, axis), r),
                    max_toi,
                ),
            )
        })
    } else {
        // Edge region: the in-range axis is the one missing from the mask.
        let in_axis = (0..3).find(|&i| mask & (1 << i) == 0)?;
        let edge = Capsule::new(
            edge_end(vtx, aabb, in_axis, false),
            edge_end(vtx, aabb, in_axis, true),
            r,
        );
        ray_vs_capsule(ray, &edge, max_toi)
    }
}

/// `vtx` with axis `i` moved to the opposite box extreme — the far end of the
/// box edge running along axis `i` from the corner `vtx`.
fn flip_axis(vtx: Vec3, aabb: &Aabb3d, i: usize) -> Vec3 {
    let (mut v, mn, mx) = (vtx.to_array(), aabb.min.to_array(), aabb.max.to_array());
    v[i] = if (v[i] - mn[i]).abs() <= (v[i] - mx[i]).abs() {
        mx[i]
    } else {
        mn[i]
    };
    Vec3::from_array(v)
}

/// `vtx` with axis `i` pinned to the box's max (`hi`) or min face — an endpoint
/// of the box edge spanning axis `i`.
fn edge_end(vtx: Vec3, aabb: &Aabb3d, i: usize, hi: bool) -> Vec3 {
    let mut v = vtx.to_array();
    v[i] = if hi {
        aabb.max.to_array()[i]
    } else {
        aabb.min.to_array()[i]
    };
    Vec3::from_array(v)
}

/// The nearest collider swept by a sphere of `radius` along `ray` within
/// `max_toi`, with its key. Ties break by the lowest key — the same
/// replay-stable policy as [`crate::query::raycast_nearest`].
#[must_use]
pub fn sphere_cast_nearest(
    ray: &Ray,
    radius: f32,
    colliders: &[(u64, Collider)],
    max_toi: f32,
) -> Option<(u64, RayHit)> {
    let mut best: Option<(u64, RayHit)> = None;
    for (key, collider) in colliders {
        if let Some(hit) = sphere_cast(ray, radius, collider, max_toi) {
            if better_keyed_hit(hit.toi, *key, best.map(|(k, h)| (k, h.toi))) {
                best = Some((*key, hit));
            }
        }
    }
    best
}

/// Whether a sphere of `radius` swept from `from` to `to` is unobstructed —
/// line-of-sight with width (a projectile or interaction clearance check).
///
/// A zero-length segment is trivially clear; a non-positive `radius` matches
/// [`crate::query::line_of_sight`]. Order-independent over the collider set.
#[must_use]
pub fn thick_line_of_sight(
    from: Vec3,
    to: Vec3,
    radius: f32,
    colliders: &[(u64, Collider)],
) -> bool {
    let seg = to - from;
    let dist = seg.length();
    if dist <= EPS {
        return true;
    }
    let Ok(ray) = Ray::new(from, seg) else {
        return true;
    };
    for (_, collider) in colliders {
        if let Some(hit) = sphere_cast(&ray, radius, collider, dist) {
            if hit.toi < dist - EPS {
                return false;
            }
        }
    }
    true
}

impl Collider {
    /// Sweep a sphere of `radius` (centred at `ray.origin`) along `ray` against
    /// this collider, up to `max_toi`. Mirrors [`Collider::raycast`].
    #[must_use]
    pub fn sphere_cast(&self, ray: &Ray, radius: f32, max_toi: f32) -> Option<RayHit> {
        sphere_cast(ray, radius, self, max_toi)
    }
}

#[cfg(test)]
mod tests;
