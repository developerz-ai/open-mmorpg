//! Pure penetration math — closest points and shape/box overlap resolution.
//!
//! Separated from the shape *types* ([`crate::shapes`]) so this module holds only
//! the collision *algorithms*: given two shapes, how deep do they overlap and
//! which way to push them apart. Every function is deterministic `f32` math with
//! degenerate-input guards, the bedrock the move-and-slide controller stands on.

use bevy_math::Vec3;

use crate::shapes::{Aabb3d, Capsule, Sphere};

/// Below this squared distance two points are treated as coincident, so we fall
/// back to an axis-aligned push-out instead of normalising a near-zero vector.
const EPS: f32 = 1e-6;

/// A minimum-translation vector: push a shape `depth` along `normal` (a unit
/// vector pointing *out* of the obstacle) to separate it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Penetration {
    /// Unit separation direction, pointing away from the obstacle.
    pub normal: Vec3,
    /// How far to move along `normal` to just separate (always `>= 0`).
    pub depth: f32,
}

/// Whether `x` is a usable positive, finite radius/extent (also rejects `NaN`
/// and infinities, which `x > 0.0` alone would let slip through in one case).
#[inline]
#[must_use]
pub(crate) fn positive_finite(x: f32) -> bool {
    x.is_finite() && x > 0.0
}

/// The point on segment `a`–`b` nearest to `p`. Degenerate (zero-length)
/// segments collapse to `a`.
#[must_use]
pub fn closest_point_on_segment(a: Vec3, b: Vec3, p: Vec3) -> Vec3 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 <= EPS {
        return a;
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    a + ab * t
}

/// Penetration of a sphere into a box, or `None` if they do not overlap.
///
/// Exact: measures the gap between the sphere centre and the nearest point on
/// the box. When the centre is *inside* the box the gap is zero, so it pushes
/// out along the least-penetrated face instead.
#[must_use]
pub fn sphere_vs_aabb(sphere: &Sphere, aabb: &Aabb3d) -> Option<Penetration> {
    if !positive_finite(sphere.radius) || !sphere.center.is_finite() {
        return None;
    }
    let closest = aabb.closest_point(sphere.center);
    let delta = sphere.center - closest;
    let dist2 = delta.length_squared();
    let r = sphere.radius;
    // `>=` so a shape resting exactly `radius` away (touching, zero depth) reports
    // no penetration rather than a spurious zero-depth contact.
    if dist2 >= r * r {
        return None;
    }
    if dist2 > EPS * EPS {
        let dist = dist2.sqrt();
        Some(Penetration {
            normal: delta / dist,
            depth: r - dist,
        })
    } else {
        // Centre inside the box: separate along the nearest face's outward normal.
        let (normal, face_depth) = least_axis_push(aabb, sphere.center);
        Some(Penetration {
            normal,
            depth: face_depth + r,
        })
    }
}

/// Penetration of a capsule into a box, or `None` if they do not overlap.
///
/// Reduces the capsule to the sphere at the spine point nearest the box: find
/// the spine point closest to the box centre, the box point closest to that,
/// then re-project onto the spine. Exact for a sphere and a close, stable
/// approximation for an upright capsule against an axis-aligned box — the
/// geometry a character controller actually walks through.
#[must_use]
pub fn capsule_vs_aabb(capsule: &Capsule, aabb: &Aabb3d) -> Option<Penetration> {
    if !positive_finite(capsule.radius) {
        return None;
    }
    let on_spine = closest_point_on_segment(capsule.a, capsule.b, aabb.center());
    let on_box = aabb.closest_point(on_spine);
    let spine = closest_point_on_segment(capsule.a, capsule.b, on_box);
    sphere_vs_aabb(&Sphere::new(spine, capsule.radius), aabb)
}

/// For a point inside `aabb`, the nearest face's outward unit normal and the
/// distance to it.
fn least_axis_push(aabb: &Aabb3d, p: Vec3) -> (Vec3, f32) {
    let lo = p - aabb.min; // distance past each min face
    let hi = aabb.max - p; // distance to each max face
    let mut normal = Vec3::Y;
    let mut best = f32::INFINITY;
    for (axis, l, h) in [
        (Vec3::X, lo.x, hi.x),
        (Vec3::Y, lo.y, hi.y),
        (Vec3::Z, lo.z, hi.z),
    ] {
        if l < best {
            best = l;
            normal = -axis; // nearer to the min face → push toward -axis
        }
        if h < best {
            best = h;
            normal = axis; // nearer to the max face → push toward +axis
        }
    }
    (normal, best.max(0.0))
}

#[cfg(test)]
mod tests;
