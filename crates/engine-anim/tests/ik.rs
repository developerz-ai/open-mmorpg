//! Inverse-kinematics tests — analytic two-bone limb solving and general FABRIK
//! chain solving. Pure math, so these run under both `--no-default-features`
//! (headless) and `--all-features` alike; no GPU, no window.

use bevy_math::{Quat, Vec3};
use omm_engine_anim::ik::{align_rotation, solve_two_bone, IkChain, IkParams};

const EPS: f32 = 1e-4;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn close(a: Vec3, b: Vec3) -> bool {
    a.abs_diff_eq(b, EPS)
}

// ── two-bone IK ──────────────────────────────────────────────────────────────

/// A straight-down arm: root at origin, mid at -1 y, end at -2 y, both bones
/// length 1. Pole in +x so the elbow bends toward +x.
fn straight_arm() -> (Vec3, Vec3, Vec3) {
    (v(0.0, 0.0, 0.0), v(0.0, -1.0, 0.0), v(0.0, -2.0, 0.0))
}

#[test]
fn two_bone_reaches_a_target_in_range() {
    let (root, mid, end) = straight_arm();
    // Target within reach (dist 1.5 < 2.0): end must land exactly on it.
    let target = v(1.5, 0.0, 0.0);
    let pole = v(1.0, 0.0, 0.0);
    let sol = solve_two_bone(root, mid, end, target, pole).expect("solve");
    assert!(sol.reached, "target inside reach must report reached");
    assert!(
        close(sol.end, target),
        "end {:?} != target {:?}",
        sol.end,
        target
    );
}

#[test]
fn two_bone_preserves_bone_lengths() {
    let (root, mid, end) = straight_arm();
    let target = v(1.2, -0.4, 0.3);
    let pole = v(1.0, 0.0, 0.0);
    let sol = solve_two_bone(root, mid, end, target, pole).expect("solve");
    // Upper and lower bones stay length 1 — IK never stretches a bone.
    assert!(
        (sol.mid.distance(root) - 1.0).abs() < EPS,
        "upper bone stretched"
    );
    assert!(
        (sol.end.distance(sol.mid) - 1.0).abs() < EPS,
        "lower bone stretched"
    );
}

#[test]
fn two_bone_bends_toward_the_pole() {
    let (root, mid, end) = straight_arm();
    // Target along +x; the elbow bends perpendicular to the aim line, toward the
    // pole. Pole +y → elbow to +y; pole -y → elbow to -y. Same end effector.
    let target = v(1.5, 0.0, 0.0);
    let up = solve_two_bone(root, mid, end, target, v(0.0, 1.0, 0.0)).expect("solve");
    let down = solve_two_bone(root, mid, end, target, v(0.0, -1.0, 0.0)).expect("solve");
    assert!(up.mid.y > root.y, "pole +y should bend the elbow toward +y");
    assert!(
        down.mid.y < root.y,
        "pole -y should bend the elbow toward -y"
    );
    assert!(close(up.end, down.end));
}

#[test]
fn two_bone_unreachable_target_straightens() {
    let (root, mid, end) = straight_arm();
    // Target at dist 5 > reach 2: arm fully extends toward it, end short of target.
    let target = v(5.0, 0.0, 0.0);
    let pole = v(1.0, 0.0, 0.0);
    let sol = solve_two_bone(root, mid, end, target, pole).expect("solve");
    assert!(!sol.reached, "out-of-reach target must report not reached");
    // Fully extended: end is 2.0 (l1+l2) along the target direction from root.
    assert!(
        close(sol.end, v(2.0, 0.0, 0.0)),
        "end {:?} not fully extended",
        sol.end
    );
    // Straight: root→mid and mid→end colinear (mid on the root→end line).
    assert!(
        close(sol.mid, v(1.0, 0.0, 0.0)),
        "mid {:?} not straight",
        sol.mid
    );
}

#[test]
fn two_bone_pole_on_aim_line_does_not_nan() {
    let (root, mid, end) = straight_arm();
    let target = v(1.5, 0.0, 0.0);
    // Pole colinear with the aim direction (both +x) — bend axis is degenerate.
    let sol = solve_two_bone(root, mid, end, target, v(3.0, 0.0, 0.0)).expect("solve");
    assert!(
        sol.mid.is_finite() && sol.end.is_finite(),
        "degenerate pole produced NaN"
    );
    assert!(
        (sol.mid.distance(root) - 1.0).abs() < EPS,
        "bone length lost on fallback"
    );
}

#[test]
fn two_bone_is_deterministic() {
    let (root, mid, end) = straight_arm();
    let target = v(0.9, -0.8, 0.5);
    let pole = v(1.0, 0.2, 0.0);
    let a = solve_two_bone(root, mid, end, target, pole).expect("solve");
    let b = solve_two_bone(root, mid, end, target, pole).expect("solve");
    assert_eq!(a, b, "same inputs must produce bit-identical output");
}

#[test]
fn two_bone_rejects_zero_length_bone() {
    // mid == root ⇒ zero upper bone.
    let r = solve_two_bone(
        v(0.0, 0.0, 0.0),
        v(0.0, 0.0, 0.0),
        v(0.0, -1.0, 0.0),
        v(1.0, 0.0, 0.0),
        v(1.0, 0.0, 0.0),
    );
    assert!(r.is_err(), "a zero-length bone must fail loud");
}

#[test]
fn two_bone_rejects_non_finite_input() {
    let (root, mid, end) = straight_arm();
    let r = solve_two_bone(root, mid, end, v(f32::NAN, 0.0, 0.0), v(1.0, 0.0, 0.0));
    assert!(r.is_err(), "a NaN target must fail loud");
}

// ── FABRIK chain IK ──────────────────────────────────────────────────────────

fn chain_positions() -> Vec<Vec3> {
    // A 3-bone chain along +x, each bone length 1 (reach 3).
    vec![
        v(0.0, 0.0, 0.0),
        v(1.0, 0.0, 0.0),
        v(2.0, 0.0, 0.0),
        v(3.0, 0.0, 0.0),
    ]
}

#[test]
fn fabrik_chain_derives_lengths_and_reach() {
    let chain = IkChain::from_positions(&chain_positions()).expect("chain");
    assert_eq!(chain.segment_count(), 3);
    assert_eq!(chain.joint_count(), 4);
    assert!((chain.reach() - 3.0).abs() < EPS);
}

#[test]
fn fabrik_reaches_a_target_in_range() {
    let joints = chain_positions();
    let chain = IkChain::from_positions(&joints).expect("chain");
    let target = v(1.0, 1.5, 0.0);
    let sol = chain
        .solve(&joints, target, &IkParams::default())
        .expect("solve");
    assert!(sol.reached, "in-range target must converge");
    let end = *sol.joints.last().expect("end joint");
    assert!(
        end.distance(target) <= IkParams::default().tolerance,
        "end {end:?} off target"
    );
}

#[test]
fn fabrik_keeps_the_base_pinned() {
    let joints = chain_positions();
    let chain = IkChain::from_positions(&joints).expect("chain");
    let sol = chain
        .solve(&joints, v(0.5, 1.0, 0.5), &IkParams::default())
        .expect("solve");
    assert!(
        close(sol.joints[0], joints[0]),
        "FABRIK must not move the root"
    );
}

#[test]
fn fabrik_preserves_segment_lengths() {
    let joints = chain_positions();
    let chain = IkChain::from_positions(&joints).expect("chain");
    let sol = chain
        .solve(&joints, v(1.2, 1.3, -0.6), &IkParams::default())
        .expect("solve");
    for i in 0..sol.joints.len() - 1 {
        let len = sol.joints[i + 1].distance(sol.joints[i]);
        assert!(
            (len - 1.0).abs() < 1e-3,
            "segment {i} length drifted to {len}"
        );
    }
}

#[test]
fn fabrik_unreachable_target_straightens_toward_it() {
    let joints = chain_positions();
    let chain = IkChain::from_positions(&joints).expect("chain");
    // Target at dist 10 along +y, reach 3: chain lays straight up the +y axis.
    let target = v(0.0, 10.0, 0.0);
    let sol = chain
        .solve(&joints, target, &IkParams::default())
        .expect("solve");
    assert!(!sol.reached, "out-of-reach target must report not reached");
    assert!(
        close(*sol.joints.last().expect("end"), v(0.0, 3.0, 0.0)),
        "not fully extended"
    );
    // Every joint lands on the +y axis (colinear straighten).
    for j in &sol.joints {
        assert!(
            j.x.abs() < EPS && j.z.abs() < EPS,
            "joint {j:?} off the aim axis"
        );
    }
}

#[test]
fn fabrik_is_deterministic() {
    let joints = chain_positions();
    let chain = IkChain::from_positions(&joints).expect("chain");
    let target = v(0.7, 1.1, 0.9);
    let a = chain
        .solve(&joints, target, &IkParams::default())
        .expect("solve");
    let b = chain
        .solve(&joints, target, &IkParams::default())
        .expect("solve");
    assert_eq!(a, b, "same inputs must produce bit-identical output");
}

#[test]
fn fabrik_rejects_joint_count_mismatch() {
    let chain = IkChain::from_positions(&chain_positions()).expect("chain");
    // Only 2 joints for a 3-bone chain.
    let wrong = vec![v(0.0, 0.0, 0.0), v(1.0, 0.0, 0.0)];
    assert!(chain
        .solve(&wrong, v(1.0, 1.0, 0.0), &IkParams::default())
        .is_err());
}

#[test]
fn fabrik_rejects_non_finite_input() {
    let joints = chain_positions();
    let chain = IkChain::from_positions(&joints).expect("chain");
    assert!(chain
        .solve(&joints, v(f32::INFINITY, 0.0, 0.0), &IkParams::default())
        .is_err());
}

#[test]
fn ik_chain_rejects_degenerate_construction() {
    assert!(
        IkChain::from_positions(&[v(0.0, 0.0, 0.0)]).is_err(),
        "needs >= 2 joints"
    );
    assert!(IkChain::from_lengths(vec![]).is_err(), "needs >= 1 bone");
    assert!(
        IkChain::from_lengths(vec![1.0, 0.0]).is_err(),
        "zero-length bone rejected"
    );
    assert!(
        IkChain::from_lengths(vec![1.0, f32::NAN]).is_err(),
        "non-finite length rejected"
    );
    assert!(IkChain::from_lengths(vec![1.0, 2.0]).is_ok());
}

#[test]
fn ik_params_validation() {
    assert!(IkParams::new(0, 1e-3).is_err(), "zero iterations rejected");
    assert!(
        IkParams::new(5, 0.0).is_err(),
        "non-positive tolerance rejected"
    );
    assert!(
        IkParams::new(5, f32::NAN).is_err(),
        "non-finite tolerance rejected"
    );
    assert!(IkParams::new(5, 1e-3).is_ok());
}

// ── rotation helper ──────────────────────────────────────────────────────────

#[test]
fn align_rotation_rotates_from_onto_to() {
    let q = align_rotation(Vec3::X, Vec3::Y);
    assert!(
        (q * Vec3::X).abs_diff_eq(Vec3::Y, EPS),
        "X should rotate onto Y"
    );
}

#[test]
fn align_rotation_zero_vector_is_identity() {
    assert_eq!(align_rotation(Vec3::ZERO, Vec3::Y), Quat::IDENTITY);
    assert_eq!(align_rotation(Vec3::X, Vec3::ZERO), Quat::IDENTITY);
}
