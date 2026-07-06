//! Pure move-and-slide: the deterministic core of the kinematic character
//! controller, free of ECS so it is unit-testable in isolation.
//!
//! Given a capsule, a desired motion, and the static world boxes near it, it
//! returns where the capsule ends up and whether it is on the ground. The
//! algorithm is the classic three-step kinematic controller:
//!
//! 1. **Horizontal pass** — move along the ground plane, then depenetrate,
//!    sliding along walls instead of stopping dead.
//! 2. **Step-up** — if a wall blocked the horizontal move, retry it lifted by
//!    `step_height` and settle back down; accept only if it climbed onto a
//!    surface and got further. This is what walks the character up stairs.
//! 3. **Vertical pass** — apply gravity/jump, depenetrate; landing on a
//!    floor-facing surface marks the capsule grounded.
//! 4. **Floor snap** — when descending or idle and left just above the ground,
//!    pull down within `snap_distance` so the character sticks to steps and
//!    slopes instead of launching off every lip.
//!
//! Depenetration is Gauss–Seidel: repeatedly resolve the single deepest contact
//! (ties by input order, which the broadphase sorts by key), which converges for
//! the convex boxes the character walks through and keeps the result stable.

use bevy_math::Vec3;

use crate::penetration::{capsule_vs_aabb, Penetration};
use crate::shapes::{Aabb3d, Capsule};

/// Small slop so a capsule rests flush without jittering against a face.
const EPS: f32 = 1e-5;

/// Tuning for [`move_and_slide`].
#[derive(Debug, Clone, Copy)]
pub struct SlideParams {
    /// World up axis (gravity opposes this). Normalised internally.
    pub up: Vec3,
    /// Maximum ledge height the character auto-climbs.
    pub step_height: f32,
    /// How far below the feet to snap onto ground when descending/idle.
    pub snap_distance: f32,
    /// Contacts shallower than this are ignored (rest slop).
    pub skin: f32,
    /// Depenetration iterations per pass.
    pub max_iterations: usize,
    /// Minimum `normal · up` for a contact to count as a walkable floor
    /// (`cos(max_slope)`).
    pub floor_min_dot: f32,
}

impl Default for SlideParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            step_height: 0.3,
            snap_distance: 0.3,
            skin: 0.02,
            max_iterations: 4,
            floor_min_dot: 0.7,
        }
    }
}

/// Result of a move: the resolved capsule and whether it ended grounded.
#[derive(Debug, Clone, Copy)]
pub struct SlideResult {
    /// The capsule after collision resolution.
    pub capsule: Capsule,
    /// Whether the capsule is resting on a walkable floor.
    pub grounded: bool,
}

/// Move `capsule` by `motion` against static `colliders`, sliding, step-climbing
/// and floor-snapping. Deterministic in the collider order given.
#[must_use]
pub fn move_and_slide(
    capsule: Capsule,
    motion: Vec3,
    colliders: &[Aabb3d],
    params: &SlideParams,
) -> SlideResult {
    let up = normalized_up(params.up);
    let mut cur = capsule;
    // Unstick first, in case the capsule spawned inside geometry.
    resolve(&mut cur, colliders, params, up);

    let vertical = up * motion.dot(up);
    let horizontal = motion - vertical;

    // 1. Horizontal move + slide.
    let mut moved = cur.translated(horizontal);
    resolve(&mut moved, colliders, params, up);

    // 2. Step-up when an obstacle impeded the horizontal move. A rounded cap
    //    against a low ledge yields an angled contact — neither clean floor nor
    //    wall — so we trigger on lost forward progress, not on the contact kind.
    let requested = horizontal.length();
    let impeded = requested > EPS && along(cur, moved, horizontal) < requested - params.skin;
    if params.step_height > 0.0 && impeded {
        if let Some(stepped) = try_step(cur, horizontal, colliders, params, up) {
            if along(cur, stepped, horizontal) > along(cur, moved, horizontal) {
                moved = stepped;
            }
        }
    }
    cur = moved;

    // 3. Vertical move (gravity / jump).
    let mut dropped = cur.translated(vertical);
    let floor_hit = resolve(&mut dropped, colliders, params, up);
    cur = dropped;
    let descending = motion.dot(up) <= EPS;
    let mut grounded = floor_hit && descending;

    // 4. Floor snap.
    if !grounded && descending && params.snap_distance > 0.0 {
        let mut snapped = cur.translated(-up * params.snap_distance);
        if resolve(&mut snapped, colliders, params, up) {
            cur = snapped;
            grounded = true;
        }
    }

    SlideResult {
        capsule: cur,
        grounded,
    }
}

/// Resolve the deepest overlap repeatedly, pushing the capsule out each time.
/// Returns whether any resolved contact was a walkable floor.
fn resolve(cap: &mut Capsule, colliders: &[Aabb3d], params: &SlideParams, up: Vec3) -> bool {
    let mut floor = false;
    for _ in 0..params.max_iterations.max(1) {
        let mut deepest: Option<Penetration> = None;
        for b in colliders {
            if let Some(pen) = capsule_vs_aabb(cap, b) {
                if pen.depth <= params.skin {
                    continue;
                }
                if deepest.is_none_or(|d| pen.depth > d.depth) {
                    deepest = Some(pen);
                }
            }
        }
        let Some(pen) = deepest else { break };
        *cap = cap.translated(pen.normal * pen.depth);
        if pen.normal.dot(up) >= params.floor_min_dot {
            floor = true;
        }
    }
    floor
}

/// Attempt a stair step: lift, move forward past the lip, drop back onto the
/// ledge. Returns the stepped capsule only if it landed on a walkable floor.
///
/// The forward probe extends past the per-tick move by the capsule radius so a
/// slowly-walking character actually clears the lip of a step in one attempt
/// (moving only `velocity * dt` would stall against the ledge every tick).
fn try_step(
    base: Capsule,
    horizontal: Vec3,
    colliders: &[Aabb3d],
    params: &SlideParams,
    up: Vec3,
) -> Option<Capsule> {
    let forward = horizontal + horizontal.normalize_or_zero() * base.radius;
    // Lift by the step height (respecting overhead obstacles).
    let mut lifted = base.translated(up * params.step_height);
    resolve(&mut lifted, colliders, params, up);
    // Move forward at the raised height, clearing the lip.
    let mut ahead = lifted.translated(forward);
    resolve(&mut ahead, colliders, params, up);
    // Settle back down onto the ledge.
    let mut settled = ahead.translated(-up * params.step_height);
    if resolve(&mut settled, colliders, params, up) {
        Some(settled)
    } else {
        None
    }
}

/// Signed progress of `to` over `from` along `dir`.
fn along(from: Capsule, to: Capsule, dir: Vec3) -> f32 {
    (to.a - from.a).dot(dir.normalize_or_zero())
}

/// The up axis, normalised, falling back to `+Y` if degenerate.
fn normalized_up(up: Vec3) -> Vec3 {
    let n = up.normalize_or_zero();
    if n == Vec3::ZERO {
        Vec3::Y
    } else {
        n
    }
}

#[cfg(test)]
mod tests;
