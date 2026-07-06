//! Blend-graph evaluation tests — weighted / additive / masked pose blending and
//! the flat node arena. Pure math, so these run under both `--no-default-features`
//! (headless) and `--all-features` alike; no GPU, no window.

use bevy_math::{Quat, Vec3};
use bevy_transform::components::Transform;
use omm_engine_anim::graph::{BlendGraph, BlendNode, BoneMask, ClipId, JointId, Pose};

const EPS: f32 = 1e-5;

fn t(x: f32, y: f32, z: f32) -> Transform {
    Transform::from_xyz(x, y, z)
}

fn pose(joints: &[Transform]) -> Pose {
    Pose::new(joints.to_vec())
}

fn assert_translation(actual: &Transform, expected: Vec3) {
    assert!(
        actual.translation.abs_diff_eq(expected, EPS),
        "translation {:?} != {:?}",
        actual.translation,
        expected
    );
}

// ── weighted blend ─────────────────────────────────────────────────────────

#[test]
fn weighted_blend_endpoints_are_the_inputs() {
    let a = pose(&[t(0.0, 0.0, 0.0)]);
    let b = pose(&[t(10.0, 0.0, 0.0)]);

    let at_zero = a.weighted_blend(&b, 0.0).expect("blend");
    let at_one = a.weighted_blend(&b, 1.0).expect("blend");
    assert_translation(&at_zero.joints[0], Vec3::ZERO);
    assert_translation(&at_one.joints[0], Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn weighted_blend_midpoint_is_halfway() {
    let a = pose(&[t(0.0, 0.0, 0.0)]);
    let b = pose(&[t(10.0, -4.0, 2.0)]);
    let mid = a.weighted_blend(&b, 0.5).expect("blend");
    assert_translation(&mid.joints[0], Vec3::new(5.0, -2.0, 1.0));
}

#[test]
fn weighted_blend_clamps_weight_out_of_range() {
    let a = pose(&[t(0.0, 0.0, 0.0)]);
    let b = pose(&[t(10.0, 0.0, 0.0)]);
    // weight > 1 clamps to the `b` end, weight < 0 clamps to the `a` end.
    assert_translation(
        &a.weighted_blend(&b, 5.0).expect("blend").joints[0],
        Vec3::new(10.0, 0.0, 0.0),
    );
    assert_translation(
        &a.weighted_blend(&b, -5.0).expect("blend").joints[0],
        Vec3::ZERO,
    );
}

#[test]
fn weighted_blend_slerps_rotation() {
    let a = pose(&[Transform::from_rotation(Quat::IDENTITY)]);
    let b = pose(&[Transform::from_rotation(Quat::from_rotation_z(
        std::f32::consts::FRAC_PI_2,
    ))]);
    let mid = a.weighted_blend(&b, 0.5).expect("blend");
    let expected = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
    assert!(
        mid.joints[0].rotation.abs_diff_eq(expected, EPS),
        "rotation slerp midpoint wrong"
    );
}

#[test]
fn weighted_blend_rejects_joint_count_mismatch() {
    let a = pose(&[t(0.0, 0.0, 0.0), t(1.0, 0.0, 0.0)]);
    let b = pose(&[t(10.0, 0.0, 0.0)]);
    assert!(
        a.weighted_blend(&b, 0.5).is_err(),
        "mismatched joint counts must fail loud"
    );
}

#[test]
fn weighted_blend_is_deterministic() {
    let a = pose(&[t(1.0, 2.0, 3.0), t(-1.0, 0.0, 4.0)]);
    let b = pose(&[t(4.0, 0.0, 1.0), t(2.0, 2.0, 2.0)]);
    let first = a.weighted_blend(&b, 0.3).expect("blend");
    let second = a.weighted_blend(&b, 0.3).expect("blend");
    assert_eq!(
        first, second,
        "same inputs must produce bit-identical output"
    );
}

// ── additive blend ───────────────────────────────────────────────────────────

#[test]
fn additive_blend_zero_weight_is_the_base() {
    let base = pose(&[t(1.0, 1.0, 1.0)]);
    let additive = pose(&[t(5.0, 5.0, 5.0)]);
    let reference = pose(&[t(0.0, 0.0, 0.0)]);
    let out = base
        .additive_blend(&additive, &reference, 0.0)
        .expect("additive");
    assert_translation(&out.joints[0], Vec3::new(1.0, 1.0, 1.0));
}

#[test]
fn additive_blend_full_weight_adds_the_delta() {
    let base = pose(&[t(1.0, 1.0, 1.0)]);
    let additive = pose(&[t(5.0, 2.0, 0.0)]);
    let reference = pose(&[t(2.0, 0.0, 0.0)]);
    // delta = additive - reference = (3, 2, 0); out = base + delta.
    let out = base
        .additive_blend(&additive, &reference, 1.0)
        .expect("additive");
    assert_translation(&out.joints[0], Vec3::new(4.0, 3.0, 1.0));
}

#[test]
fn additive_blend_zero_delta_leaves_base_unchanged() {
    let base = pose(&[t(3.0, -2.0, 7.0)]);
    // additive == reference ⇒ zero delta ⇒ base is unchanged at any weight.
    let clip = pose(&[t(9.0, 9.0, 9.0)]);
    let out = base.additive_blend(&clip, &clip, 1.0).expect("additive");
    assert_translation(&out.joints[0], Vec3::new(3.0, -2.0, 7.0));
}

#[test]
fn additive_blend_rejects_length_mismatch() {
    let base = pose(&[t(0.0, 0.0, 0.0)]);
    let additive = pose(&[t(1.0, 0.0, 0.0), t(2.0, 0.0, 0.0)]);
    let reference = pose(&[t(0.0, 0.0, 0.0)]);
    assert!(
        base.additive_blend(&additive, &reference, 1.0).is_err(),
        "length mismatch must fail"
    );
}

// ── masked blend ─────────────────────────────────────────────────────────────

#[test]
fn masked_blend_selects_per_joint() {
    let base = pose(&[t(0.0, 0.0, 0.0), t(0.0, 0.0, 0.0)]);
    let overlay = pose(&[t(10.0, 0.0, 0.0), t(20.0, 0.0, 0.0)]);
    // Joint 0 masked in (upper body), joint 1 masked out (lower body).
    let mask = BoneMask::from_joints(2, &[JointId(0)]);
    let out = base.masked_blend(&overlay, &mask).expect("masked");
    assert_translation(&out.joints[0], Vec3::new(10.0, 0.0, 0.0));
    assert_translation(&out.joints[1], Vec3::ZERO);
}

#[test]
fn masked_blend_full_and_none_masks() {
    let base = pose(&[t(0.0, 0.0, 0.0)]);
    let overlay = pose(&[t(10.0, 0.0, 0.0)]);
    assert_translation(
        &base
            .masked_blend(&overlay, &BoneMask::full(1))
            .expect("masked")
            .joints[0],
        Vec3::new(10.0, 0.0, 0.0),
    );
    assert_translation(
        &base
            .masked_blend(&overlay, &BoneMask::none(1))
            .expect("masked")
            .joints[0],
        Vec3::ZERO,
    );
}

#[test]
fn masked_blend_partial_weight_interpolates() {
    let base = pose(&[t(0.0, 0.0, 0.0)]);
    let overlay = pose(&[t(10.0, 0.0, 0.0)]);
    let mask = BoneMask {
        weights: vec![0.25],
    };
    let out = base.masked_blend(&overlay, &mask).expect("masked");
    assert_translation(&out.joints[0], Vec3::new(2.5, 0.0, 0.0));
}

#[test]
fn masked_blend_rejects_length_mismatch() {
    let base = pose(&[t(0.0, 0.0, 0.0), t(0.0, 0.0, 0.0)]);
    let overlay = pose(&[t(10.0, 0.0, 0.0), t(20.0, 0.0, 0.0)]);
    let mask = BoneMask::full(1); // wrong length
    assert!(
        base.masked_blend(&overlay, &mask).is_err(),
        "mask length mismatch must fail"
    );
}

#[test]
fn bone_mask_from_joints_and_weight_lookup() {
    let mask = BoneMask::from_joints(3, &[JointId(0), JointId(2), JointId(9)]);
    assert_eq!(mask.len(), 3);
    assert_eq!(mask.weight(JointId(0)), 1.0);
    assert_eq!(mask.weight(JointId(1)), 0.0);
    assert_eq!(mask.weight(JointId(2)), 1.0);
    // Out-of-range joint id clamps to 0.0 rather than panicking.
    assert_eq!(mask.weight(JointId(99)), 0.0);
}

// ── graph evaluation ─────────────────────────────────────────────────────────

/// Sampler for the fixtures below: clip 0 at origin, clip 1 at x=10, clip 2 at
/// x=2 (an additive reference), anything else unresolved.
fn sampler(id: ClipId) -> Option<Pose> {
    match id.0 {
        0 => Some(pose(&[t(0.0, 0.0, 0.0)])),
        1 => Some(pose(&[t(10.0, 0.0, 0.0)])),
        2 => Some(pose(&[t(2.0, 0.0, 0.0)])),
        _ => None,
    }
}

#[test]
fn graph_single_clip_returns_sampled_pose() {
    let mut g = BlendGraph::new();
    let root = g.add(BlendNode::Clip(ClipId(1)));
    g.set_root(root);
    let out = g.evaluate(&sampler).expect("evaluate");
    assert_translation(&out.joints[0], Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn graph_weighted_blend_node() {
    let mut g = BlendGraph::new();
    let a = g.add(BlendNode::Clip(ClipId(0)));
    let b = g.add(BlendNode::Clip(ClipId(1)));
    let root = g.add(BlendNode::Blend { a, b, weight: 0.5 });
    g.set_root(root);
    let out = g.evaluate(&sampler).expect("evaluate");
    assert_translation(&out.joints[0], Vec3::new(5.0, 0.0, 0.0));
}

#[test]
fn graph_additive_node() {
    let mut g = BlendGraph::new();
    let base = g.add(BlendNode::Clip(ClipId(0))); // origin
    let additive = g.add(BlendNode::Clip(ClipId(1))); // x=10
                                                      // delta = clip1 - clip2 = (8,0,0); out = base + delta.
    let root = g.add(BlendNode::Additive {
        base,
        additive,
        reference: ClipId(2),
        weight: 1.0,
    });
    g.set_root(root);
    let out = g.evaluate(&sampler).expect("evaluate");
    assert_translation(&out.joints[0], Vec3::new(8.0, 0.0, 0.0));
}

#[test]
fn graph_masked_node() {
    let mut g = BlendGraph::new();
    let base = g.add(BlendNode::Clip(ClipId(0))); // origin
    let overlay = g.add(BlendNode::Clip(ClipId(1))); // x=10
    let root = g.add(BlendNode::Masked {
        base,
        overlay,
        mask: BoneMask::full(1),
    });
    g.set_root(root);
    let out = g.evaluate(&sampler).expect("evaluate");
    assert_translation(&out.joints[0], Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn graph_nested_blend_of_blends() {
    let mut g = BlendGraph::new();
    let a = g.add(BlendNode::Clip(ClipId(0))); // 0
    let b = g.add(BlendNode::Clip(ClipId(1))); // 10
    let inner = g.add(BlendNode::Blend { a, b, weight: 1.0 }); // -> 10
    let root = g.add(BlendNode::Blend {
        a,
        b: inner,
        weight: 0.5,
    }); // -> 5
    g.set_root(root);
    let out = g.evaluate(&sampler).expect("evaluate");
    assert_translation(&out.joints[0], Vec3::new(5.0, 0.0, 0.0));
}

#[test]
fn graph_missing_clip_sample_fails_loud() {
    let mut g = BlendGraph::new();
    let root = g.add(BlendNode::Clip(ClipId(42))); // unresolved
    g.set_root(root);
    assert!(
        g.evaluate(&sampler).is_err(),
        "an unresolved clip must fail loud"
    );
}

#[test]
fn graph_dangling_node_id_fails_loud() {
    let mut g = BlendGraph::new();
    let a = g.add(BlendNode::Clip(ClipId(0)));
    let root = g.add(BlendNode::Blend {
        a,
        b: omm_engine_anim::NodeId(99),
        weight: 0.5,
    });
    g.set_root(root);
    assert!(
        g.evaluate(&sampler).is_err(),
        "a dangling node id must fail loud"
    );
}

#[test]
fn graph_cycle_fails_loud() {
    use omm_engine_anim::NodeId;
    let mut g = BlendGraph::new();
    // Node 0 blends with itself → a cycle the depth guard must catch.
    let root = g.add(BlendNode::Blend {
        a: NodeId(0),
        b: NodeId(0),
        weight: 0.5,
    });
    g.set_root(root);
    assert!(
        g.evaluate(&sampler).is_err(),
        "a cyclic graph must fail loud, never loop"
    );
}

#[test]
fn graph_non_finite_weight_fails_loud() {
    let mut g = BlendGraph::new();
    let a = g.add(BlendNode::Clip(ClipId(0)));
    let b = g.add(BlendNode::Clip(ClipId(1)));
    let root = g.add(BlendNode::Blend {
        a,
        b,
        weight: f32::NAN,
    });
    g.set_root(root);
    assert!(g.evaluate(&sampler).is_err(), "a NaN weight must fail loud");
}

#[test]
fn graph_evaluation_is_deterministic() {
    let mut g = BlendGraph::new();
    let a = g.add(BlendNode::Clip(ClipId(0)));
    let b = g.add(BlendNode::Clip(ClipId(1)));
    let root = g.add(BlendNode::Blend { a, b, weight: 0.37 });
    g.set_root(root);
    let first = g.evaluate(&sampler).expect("evaluate");
    let second = g.evaluate(&sampler).expect("evaluate");
    assert_eq!(first, second, "graph evaluation must be deterministic");
}
