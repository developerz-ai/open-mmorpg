//! In-house **inverse kinematics** — an analytic two-bone limb solver plus a
//! general FABRIK chain solver, both pure, headless, and deterministic.
//!
//! # Why in-house
//! Bevy core ships no IK. The [animation spec](../../../docs/specs/game-engine/animation/README.md)
//! originally named the community crates `bevy_mod_inverse_kinematics` (FABRIK)
//! and `bevy_animation_graph` (two-bone) to fill the gap. Per ADR-0001 we avoid
//! ecosystem crates that may lag Bevy 0.19 and break CI, so this module implements
//! the two solvers we actually need as a few dozen lines of pure math we own.
//!
//! # Two solvers, two jobs
//! * [`solve_two_bone`] — the **limb** solver: an exact, single-pass, law-of-cosines
//!   solution for a three-joint chain (shoulder→elbow→wrist, hip→knee→ankle). A
//!   **pole** hint fixes the bend plane so the elbow/knee never flips. This is the
//!   correct tool for arms and legs and what an "Two Bone IK" node does elsewhere.
//! * [`IkChain::solve`] — the **general** solver: iterative FABRIK over an N-joint
//!   chain (a spine, a tail, a rope) with fixed segment lengths. Converges in a
//!   bounded number of iterations; the two-bone case is exact via `solve_two_bone`.
//!
//! # Deterministic where shared
//! Both solvers are pure `f32` math with fixed iteration order and count, no
//! randomness and no allocation-order dependence, so the server and every client
//! that runs them produce bit-identical output — a hard requirement for anything
//! reused in the shared sim (foot placement that affects a hitbox, say). Degenerate
//! inputs (coincident joints, target on the root, pole on the aim line) are guarded
//! to a stable fallback rather than producing `NaN`, and truly invalid skeletons
//! (a zero-length bone) fail loud with an [`AnimError`].

use bevy_math::{Quat, Vec3};
use bevy_reflect::Reflect;

use crate::error::AnimError;

/// Guard epsilon for near-zero lengths — below this a vector is treated as having
/// no usable direction and a stable fallback is used instead of dividing by ~0.
const EPS: f32 = 1e-6;

/// Iteration budget and convergence tolerance for the [`IkChain`] FABRIK solve.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct IkParams {
    /// Maximum backward/forward passes before giving up (must be ≥ 1).
    pub max_iterations: u32,
    /// End-effector distance to `target` at or below which the solve is done.
    pub tolerance: f32,
}

impl Default for IkParams {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            tolerance: 1e-3,
        }
    }
}

impl IkParams {
    /// Build validated params. Fails loud unless at least one iteration is allowed
    /// and the tolerance is finite and positive.
    pub fn new(max_iterations: u32, tolerance: f32) -> Result<Self, AnimError> {
        if max_iterations == 0 {
            return Err(AnimError::new("IK max_iterations must be >= 1"));
        }
        if !tolerance.is_finite() || tolerance <= 0.0 {
            return Err(AnimError::new("IK tolerance must be finite and > 0"));
        }
        Ok(Self {
            max_iterations,
            tolerance,
        })
    }
}

/// Solved end positions of a [`solve_two_bone`] call.
///
/// Runtime output rather than authored content (like [`Pose`](crate::graph::Pose)),
/// so it is deliberately not reflected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoBoneSolution {
    /// New mid-joint (elbow / knee) position, bent toward the pole.
    pub mid: Vec3,
    /// New end-effector (wrist / ankle) position — equals `target` when reachable,
    /// else the closest fully-extended point toward it.
    pub end: Vec3,
    /// Whether `target` was within the chain's reach (no clamping applied).
    pub reached: bool,
}

/// Analytic **two-bone IK** for a limb: place the mid and end joints so the end
/// reaches `target`, bending in the plane defined by the `pole` hint.
///
/// `root`/`mid`/`end` are the current joint positions; the upper (`root`→`mid`)
/// and lower (`mid`→`end`) bone lengths are derived from them and preserved. The
/// solution is exact and single-pass via the law of cosines. When `target` is out
/// of reach the limb straightens toward it and [`TwoBoneSolution::reached`] is
/// `false`. Fails loud on a non-finite input or a zero-length bone.
pub fn solve_two_bone(
    root: Vec3,
    mid: Vec3,
    end: Vec3,
    target: Vec3,
    pole: Vec3,
) -> Result<TwoBoneSolution, AnimError> {
    for v in [root, mid, end, target, pole] {
        if !v.is_finite() {
            return Err(AnimError::new("two-bone IK input is not finite"));
        }
    }
    let upper = mid.distance(root);
    let lower = end.distance(mid);
    if upper <= EPS || lower <= EPS {
        return Err(AnimError::new("two-bone IK bone length is ~zero"));
    }

    let to_target = target - root;
    let dist_raw = to_target.length();
    let reached = dist_raw <= upper + lower && dist_raw >= (upper - lower).abs();

    // Aim direction root→target, with a stable fallback if the target sits on the
    // root (no direction) — reuse the current upper-bone direction, then world-up.
    let dir = if dist_raw > EPS {
        to_target / dist_raw
    } else {
        let fallback = (mid - root).normalize_or_zero();
        if fallback.length_squared() > EPS {
            fallback
        } else {
            Vec3::Y
        }
    };

    // Clamp the reach into the valid triangle so `acos` stays in range.
    let dist = dist_raw.clamp((upper - lower).abs(), upper + lower);
    // Law of cosines: interior angle at the root between `dir` and the upper bone.
    let cos_a =
        ((upper * upper + dist * dist - lower * lower) / (2.0 * upper * dist)).clamp(-1.0, 1.0);
    let sin_a = (1.0 - cos_a * cos_a).max(0.0).sqrt();

    // Bend toward the pole: the component of (pole − root) perpendicular to `dir`.
    // Degenerate pole (on the aim line) falls back to any stable orthonormal axis.
    let pole_vec = pole - root;
    let perp = pole_vec - dir * pole_vec.dot(dir);
    let bend = {
        let b = perp.normalize_or_zero();
        if b.length_squared() > EPS {
            b
        } else {
            dir.any_orthonormal_vector()
        }
    };

    let new_mid = root + dir * (upper * cos_a) + bend * (upper * sin_a);
    let new_end = root + dir * dist;
    Ok(TwoBoneSolution {
        mid: new_mid,
        end: new_end,
        reached,
    })
}

/// Solved joint positions of an [`IkChain::solve`] call.
#[derive(Debug, Clone, PartialEq)]
pub struct IkSolution {
    /// Joint positions after the solve, root first, end-effector last.
    pub joints: Vec<Vec3>,
    /// Backward/forward passes actually run (`0` when already converged / stretched).
    pub iterations: u32,
    /// Whether the end effector landed within `tolerance` of `target`.
    pub reached: bool,
}

/// A fixed-length joint chain solved by **FABRIK** (Forward And Backward Reaching
/// Inverse Kinematics). Segment lengths are captured once from a rest pose and
/// preserved by every solve — bones never stretch.
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct IkChain {
    /// Length of each bone, ordered root→tip. `len() == joint_count() - 1`.
    lengths: Vec<f32>,
    /// Total reach = sum of `lengths`.
    reach: f32,
}

impl IkChain {
    /// Build a chain from rest-pose joint positions, deriving one bone length per
    /// adjacent pair. Needs at least two joints; fails loud on a non-finite or
    /// zero-length segment.
    pub fn from_positions(joints: &[Vec3]) -> Result<Self, AnimError> {
        if joints.len() < 2 {
            return Err(AnimError::new("IK chain needs at least 2 joints"));
        }
        let lengths = joints
            .windows(2)
            .map(|w| w[1].distance(w[0]))
            .collect::<Vec<_>>();
        Self::from_lengths(lengths)
    }

    /// Build a chain directly from bone lengths. Needs at least one bone; fails
    /// loud on a non-finite or ~zero length.
    pub fn from_lengths(lengths: Vec<f32>) -> Result<Self, AnimError> {
        if lengths.is_empty() {
            return Err(AnimError::new("IK chain needs at least 1 bone"));
        }
        for &l in &lengths {
            if !l.is_finite() || l <= EPS {
                return Err(AnimError::new("IK bone length must be finite and > 0"));
            }
        }
        let reach = lengths.iter().sum();
        Ok(Self { lengths, reach })
    }

    /// Number of bones (segments) in the chain.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.lengths.len()
    }

    /// Number of joints (`segment_count() + 1`).
    #[must_use]
    pub fn joint_count(&self) -> usize {
        self.lengths.len() + 1
    }

    /// Total reach — the sum of all bone lengths.
    #[must_use]
    pub fn reach(&self) -> f32 {
        self.reach
    }

    /// Solve the chain toward `target` with FABRIK, keeping `joints[0]` pinned as
    /// the fixed base. `joints` must have exactly [`joint_count`](Self::joint_count)
    /// entries. When the target is beyond [`reach`](Self::reach) the chain
    /// straightens toward it in a single pass (`reached = false`); otherwise it
    /// iterates up to [`IkParams::max_iterations`], stopping early at `tolerance`.
    /// Fails loud on a joint-count mismatch or a non-finite input.
    pub fn solve(
        &self,
        joints: &[Vec3],
        target: Vec3,
        params: &IkParams,
    ) -> Result<IkSolution, AnimError> {
        if joints.len() != self.joint_count() {
            return Err(AnimError::new(format!(
                "IK solve: expected {} joints, got {}",
                self.joint_count(),
                joints.len()
            )));
        }
        if !target.is_finite() || joints.iter().any(|j| !j.is_finite()) {
            return Err(AnimError::new("IK solve input is not finite"));
        }

        let n = self.lengths.len();
        let last = n;
        let base = joints[0];
        let mut p = joints.to_vec();

        // Unreachable: lay every joint out along the base→target ray, one bone at a
        // time. Single pass, exact, no iteration.
        if base.distance(target) >= self.reach {
            for i in 0..n {
                let r = target.distance(p[i]).max(EPS);
                let lambda = self.lengths[i] / r;
                p[i + 1] = p[i].lerp(target, lambda);
            }
            return Ok(IkSolution {
                joints: p,
                iterations: 1,
                reached: false,
            });
        }

        let mut iterations = 0;
        while iterations < params.max_iterations {
            if p[last].distance(target) <= params.tolerance {
                break;
            }
            iterations += 1;

            // Backward pass: pin the end to the target, walk toward the base.
            p[last] = target;
            for i in (0..n).rev() {
                let r = p[i + 1].distance(p[i]).max(EPS);
                let lambda = self.lengths[i] / r;
                p[i] = p[i + 1].lerp(p[i], lambda);
            }
            // Forward pass: pin the base back, walk toward the tip.
            p[0] = base;
            for i in 0..n {
                let r = p[i + 1].distance(p[i]).max(EPS);
                let lambda = self.lengths[i] / r;
                p[i + 1] = p[i].lerp(p[i + 1], lambda);
            }
        }

        let reached = p[last].distance(target) <= params.tolerance;
        Ok(IkSolution {
            joints: p,
            iterations,
            reached,
        })
    }
}

/// The minimal rotation aligning direction `from` onto `to` — the swing that turns
/// a rest bone direction into a solved one, so IK positions can drive joint
/// rotations. Deterministic; a zero-length input yields [`Quat::IDENTITY`].
#[must_use]
pub fn align_rotation(from: Vec3, to: Vec3) -> Quat {
    let f = from.normalize_or_zero();
    let t = to.normalize_or_zero();
    if f.length_squared() < EPS || t.length_squared() < EPS {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_arc(f, t)
    }
}
