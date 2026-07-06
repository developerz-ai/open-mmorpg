//! Pure, headless **animation blend graph** — weighted / additive / masked pose
//! blending, authored as data and evaluated with no GPU or window.
//!
//! # Data-authored motion
//! Locomotion is a blend space / state machine authored as *data*, not code — a
//! new emote or gait is a [`BlendGraph`] edit, not a recompile. This is the
//! [content-is-data](../../../../CLAUDE.md) rule applied to motion. A graph is a
//! flat arena of [`BlendNode`]s referenced by [`NodeId`] index (no `Box`, no
//! cycles-by-construction, trivially reflectable and serializable), evaluated
//! from a [`BlendGraph::root`].
//!
//! # Three primitives
//! Every AAA anim graph reduces to three pose operations, all pure math here:
//! * **Weighted** — linear-interpolate two poses (`lerp`/`slerp` per joint). The
//!   crossfade between gaits in a blend space.
//! * **Additive** — layer a delta (authored relative to a reference clip) on top
//!   of a base: `out = base ∘ w·(additive − reference)`. A weapon swing over a
//!   walk.
//! * **Masked** — per-joint blend gated by a [`BoneMask`]: upper body plays one
//!   clip, lower body another. Independent body-part control.
//!
//! # Headless & fail-loud
//! Evaluation samples clips through a caller-supplied closure (the render head
//! wires it to Bevy's first-party `AnimationClip` sampling; a headless test
//! supplies fixed poses), so the math is exercised identically in CI and on the
//! client. Every joint-count mismatch, dangling [`NodeId`], missing clip sample,
//! non-finite weight, or cyclic/over-deep graph returns a typed [`AnimError`] —
//! never a panic on the animation tick path.

use bevy_math::Quat;
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;

use crate::error::AnimError;

/// A data-authored reference to an `AnimationClip` (resolved to a sampled
/// [`Pose`] by the caller's sampler at evaluation time). Newtype so clip indices
/// never mix with joint or node indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub struct ClipId(pub u32);

/// A skeleton joint index. Newtype so joint math never mixes with clip or node
/// indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub struct JointId(pub u16);

/// An index into a [`BlendGraph`]'s node arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub struct NodeId(pub u32);

/// A full skeletal pose: one local [`Transform`] per joint, indexed by joint.
///
/// Runtime data (sampled from clips, produced by blends) rather than authored
/// content, so it is deliberately *not* reflected — the authored graph is the
/// data artifact, the pose is its transient evaluation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pose {
    /// Local transform per joint, ordered by [`JointId`].
    pub joints: Vec<Transform>,
}

impl Pose {
    /// Wrap a per-joint transform list as a pose.
    #[must_use]
    pub fn new(joints: Vec<Transform>) -> Self {
        Self { joints }
    }

    /// Number of joints in this pose.
    #[must_use]
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// Weighted blend toward `other` by `weight` (clamped to `[0, 1]`): the
    /// crossfade primitive. Per joint: `lerp` translation/scale, `slerp`
    /// rotation. Fails loud if the poses have different joint counts.
    pub fn weighted_blend(&self, other: &Pose, weight: f32) -> Result<Pose, AnimError> {
        self.require_same_len(other.joint_count(), "weighted blend")?;
        let w = weight.clamp(0.0, 1.0);
        let joints = self
            .joints
            .iter()
            .zip(&other.joints)
            .map(|(a, b)| blend_transform(a, b, w))
            .collect();
        Ok(Pose { joints })
    }

    /// Additive blend: layer the delta of `additive` relative to `reference` on
    /// top of `self`, scaled by `weight` (clamped to `[0, 1]`). The delta
    /// rotation is pre-multiplied onto the base; translation/scale deltas add.
    /// Fails loud if any of the three poses disagree on joint count.
    pub fn additive_blend(
        &self,
        additive: &Pose,
        reference: &Pose,
        weight: f32,
    ) -> Result<Pose, AnimError> {
        self.require_same_len(additive.joint_count(), "additive blend")?;
        self.require_same_len(reference.joint_count(), "additive reference")?;
        let w = weight.clamp(0.0, 1.0);
        let joints = (0..self.joints.len())
            .map(|i| {
                add_transform(
                    &self.joints[i],
                    &additive.joints[i],
                    &reference.joints[i],
                    w,
                )
            })
            .collect();
        Ok(Pose { joints })
    }

    /// Masked blend: per joint, blend from `self` toward `overlay` by the mask's
    /// per-joint weight — upper/lower-body independence. Fails loud if the
    /// overlay or mask length disagrees with this pose.
    pub fn masked_blend(&self, overlay: &Pose, mask: &BoneMask) -> Result<Pose, AnimError> {
        self.require_same_len(overlay.joint_count(), "masked overlay")?;
        self.require_same_len(mask.len(), "masked mask")?;
        let joints = (0..self.joints.len())
            .map(|i| blend_transform(&self.joints[i], &overlay.joints[i], mask.weight_at(i)))
            .collect();
        Ok(Pose { joints })
    }

    fn require_same_len(&self, other: usize, op: &str) -> Result<(), AnimError> {
        if self.joints.len() == other {
            Ok(())
        } else {
            Err(AnimError::new(format!(
                "{op}: joint count mismatch ({} vs {other})",
                self.joints.len()
            )))
        }
    }
}

/// Per-joint blend weights in `[0, 1]` selecting how much an overlay affects each
/// joint — the upper/lower-body split. Length equals the skeleton's joint count.
#[derive(Debug, Clone, PartialEq, Default, Reflect)]
pub struct BoneMask {
    /// One blend weight per joint, ordered by [`JointId`].
    pub weights: Vec<f32>,
}

impl BoneMask {
    /// A mask that fully applies the overlay to every joint (all weights `1.0`).
    #[must_use]
    pub fn full(joint_count: usize) -> Self {
        Self {
            weights: vec![1.0; joint_count],
        }
    }

    /// A mask that applies to no joint (all weights `0.0`).
    #[must_use]
    pub fn none(joint_count: usize) -> Self {
        Self {
            weights: vec![0.0; joint_count],
        }
    }

    /// A binary mask: the listed joints get weight `1.0`, all others `0.0`.
    /// Joint ids outside `joint_count` are ignored (the mask is sized to the
    /// skeleton, not to the request).
    #[must_use]
    pub fn from_joints(joint_count: usize, joints: &[JointId]) -> Self {
        let mut weights = vec![0.0; joint_count];
        for joint in joints {
            if let Some(w) = weights.get_mut(joint.0 as usize) {
                *w = 1.0;
            }
        }
        Self { weights }
    }

    /// Number of joints this mask covers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// Whether the mask covers no joints.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    /// Blend weight for `joint`, clamped to `[0, 1]`; `0.0` if out of range.
    #[must_use]
    pub fn weight(&self, joint: JointId) -> f32 {
        self.weight_at(joint.0 as usize)
    }

    fn weight_at(&self, index: usize) -> f32 {
        self.weights
            .get(index)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0)
    }
}

/// One node of a [`BlendGraph`]. Children are referenced by [`NodeId`] index into
/// the graph's arena, so the type is flat, reflectable, and free of `Box`.
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum BlendNode {
    /// Leaf: the sampled pose of a clip, resolved by the caller's sampler.
    Clip(ClipId),
    /// Weighted crossfade between two child nodes.
    Blend {
        /// Node evaluated as the `weight = 0` end.
        a: NodeId,
        /// Node evaluated as the `weight = 1` end.
        b: NodeId,
        /// Interpolation factor, clamped to `[0, 1]`.
        weight: f32,
    },
    /// Layer an additive delta (authored relative to `reference`) onto `base`.
    Additive {
        /// The base pose the delta is applied on top of.
        base: NodeId,
        /// The clip whose delta from `reference` is layered in.
        additive: NodeId,
        /// The reference the additive clip's delta is measured against.
        reference: ClipId,
        /// Layer intensity, clamped to `[0, 1]`.
        weight: f32,
    },
    /// Per-joint blend of `overlay` onto `base`, gated by a [`BoneMask`].
    Masked {
        /// The base pose (used where the mask weight is `0`).
        base: NodeId,
        /// The overlay pose (used where the mask weight is `1`).
        overlay: NodeId,
        /// Per-joint blend weights.
        mask: BoneMask,
    },
}

/// A data-authored blend graph: a flat arena of [`BlendNode`]s evaluated from
/// [`root`](BlendGraph::root) down to clip leaves.
#[derive(Debug, Clone, PartialEq, Default, Reflect)]
pub struct BlendGraph {
    nodes: Vec<BlendNode>,
    root: NodeId,
}

impl BlendGraph {
    /// An empty graph (root defaults to node `0`; set it with [`set_root`]).
    ///
    /// [`set_root`]: BlendGraph::set_root
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a node, returning its [`NodeId`].
    pub fn add(&mut self, node: BlendNode) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }

    /// Set the node evaluation starts from.
    pub fn set_root(&mut self, root: NodeId) {
        self.root = root;
    }

    /// The root node id.
    #[must_use]
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Number of nodes in the arena.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the arena has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Evaluate the graph to a final [`Pose`]. `sample` resolves a [`ClipId`] to
    /// its sampled pose (headless test: a fixed map; render head: Bevy clip
    /// sampling). Fails loud on dangling node ids, missing clip samples,
    /// non-finite weights, joint-count mismatches, or a cyclic/over-deep graph.
    pub fn evaluate<F>(&self, sample: &F) -> Result<Pose, AnimError>
    where
        F: Fn(ClipId) -> Option<Pose>,
    {
        // A simple evaluation path visits at most one distinct node per level, so
        // `nodes.len()` levels bound any acyclic graph; exceeding it means a cycle.
        self.eval(self.root, sample, self.nodes.len())
    }

    fn eval<F>(&self, id: NodeId, sample: &F, depth: usize) -> Result<Pose, AnimError>
    where
        F: Fn(ClipId) -> Option<Pose>,
    {
        let depth = depth
            .checked_sub(1)
            .ok_or_else(|| AnimError::new("blend graph is cyclic or exceeds node depth"))?;
        let node = self
            .nodes
            .get(id.0 as usize)
            .ok_or_else(|| AnimError::new(format!("blend node index {} out of range", id.0)))?;
        match node {
            BlendNode::Clip(clip) => sample(*clip)
                .ok_or_else(|| AnimError::new(format!("no sampled pose for clip {}", clip.0))),
            BlendNode::Blend { a, b, weight } => {
                check_weight(*weight)?;
                let pa = self.eval(*a, sample, depth)?;
                let pb = self.eval(*b, sample, depth)?;
                pa.weighted_blend(&pb, *weight)
            }
            BlendNode::Additive {
                base,
                additive,
                reference,
                weight,
            } => {
                check_weight(*weight)?;
                let pbase = self.eval(*base, sample, depth)?;
                let padd = self.eval(*additive, sample, depth)?;
                let pref = sample(*reference).ok_or_else(|| {
                    AnimError::new(format!("no reference pose for clip {}", reference.0))
                })?;
                pbase.additive_blend(&padd, &pref, *weight)
            }
            BlendNode::Masked {
                base,
                overlay,
                mask,
            } => {
                let pbase = self.eval(*base, sample, depth)?;
                let pov = self.eval(*overlay, sample, depth)?;
                pbase.masked_blend(&pov, mask)
            }
        }
    }
}

/// Reject non-finite authored weights before they poison a blend.
fn check_weight(weight: f32) -> Result<(), AnimError> {
    if weight.is_finite() {
        Ok(())
    } else {
        Err(AnimError::new("blend weight is not finite"))
    }
}

/// Weighted transform interpolation (`t` clamped to `[0, 1]`): `lerp`
/// translation/scale, `slerp` rotation.
fn blend_transform(a: &Transform, b: &Transform, t: f32) -> Transform {
    let t = t.clamp(0.0, 1.0);
    Transform {
        translation: a.translation.lerp(b.translation, t),
        rotation: a.rotation.slerp(b.rotation, t),
        scale: a.scale.lerp(b.scale, t),
    }
}

/// Layer the delta of `additive` relative to `reference` onto `base` by `w`
/// (clamped): translation/scale deltas add, the rotation delta is pre-multiplied.
fn add_transform(
    base: &Transform,
    additive: &Transform,
    reference: &Transform,
    w: f32,
) -> Transform {
    let w = w.clamp(0.0, 1.0);
    let delta_rot = additive.rotation * reference.rotation.inverse();
    Transform {
        translation: base.translation + (additive.translation - reference.translation) * w,
        rotation: (Quat::IDENTITY.slerp(delta_rot, w) * base.rotation).normalize(),
        scale: base.scale + (additive.scale - reference.scale) * w,
    }
}
