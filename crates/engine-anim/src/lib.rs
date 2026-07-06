//! # Engine Animation — blend graph, IK, VAT math + distance-tiered LOD
//!
//! First-party animation layer for the game engine. Four pure, headless pieces
//! plus a feature-gated skinning head:
//! - [`graph`] — a data-authored [`BlendGraph`](graph::BlendGraph): weighted,
//!   additive, and masked pose blending, the three primitives every AAA anim
//!   graph reduces to. Pure math, evaluated identically in CI and on the client.
//! - [`ik`] — in-house inverse kinematics: an exact analytic
//!   [`solve_two_bone`](ik::solve_two_bone) limb solver (pole-controlled) and a
//!   general [`IkChain`](ik::IkChain) FABRIK solver. Pure, deterministic — no
//!   ecosystem IK crate (ADR-0001).
//! - [`vat`] — vertex-animation-texture frame/index math: [`VatClip`](vat::VatClip)
//!   frame sampling and [`VatLayout`](vat::VatLayout) texel/UV addressing for
//!   crowd-scale baked instances. Pure; GPU bake/playback is the client's job.
//! - [`lod`] — distance-tiered [`AnimTier`](lod::AnimTier) selection: near
//!   skeletal → mid reduced → far VAT, with the local player pinned to full
//!   skeletal. Pure, deterministic.
//! - Under the `render` feature, the [first-party
//!   `bevy_animation`](https://docs.rs/bevy_animation) plugin drives
//!   `AnimationPlayer`/`AnimationClip`/`AnimationGraph` playback and the GPU
//!   `SkinnedMesh` component is registered for reflection. We deliberately use
//!   the first-party graph, *not* third-party `bevy_animation_graph` (ADR-0001:
//!   avoid ecosystem crates that may lag Bevy 0.19 and break CI).
//!
//! # Headless-first
//! Blend evaluation and LOD selection have no GPU, window, or asset dependency —
//! an agent reasons about motion blending and the crowd LOD ladder in a headless
//! harness exactly as the client does. The `render` feature only adds the device
//! skinning wiring on top; `--no-default-features` boots with zero GPU deps.
//!
//! # What CI verifies vs what it does not
//!
//! **CI verifies (headless, every commit):**
//! - Weighted/additive/masked blend math: identities, symmetry, endpoints,
//!   and fail-loud joint-count/weight/index validation.
//! - Blend-graph evaluation over the flat node arena, including cyclic/over-deep
//!   and dangling-id rejection.
//! - Two-bone + FABRIK IK: reach, bone-length preservation, pole bend direction,
//!   unreachable straightening, determinism, and degenerate-input guards.
//! - VAT frame sampling (loop/once/ping-pong wrap & clamp) and layout texel/UV/index
//!   math, including out-of-range and non-finite-time handling.
//! - LOD tier selection across the distance ladder, the local-player-never-VAT
//!   rule, NaN handling, and reduced-bone-budget math.
//! - Reflection registration: every authored animation type is in the app's
//!   `AppTypeRegistry` so the MCP editor and agents can enumerate them.
//!
//! **CI does NOT verify (GPU required, client-track / manual):**
//! - GPU linear-blend skinning output — vertex deformation needs a device.
//! - VAT baking / shader playback — the baked-texture path needs a real GPU.
//! - `bevy_animation` clip sampling on device and root-motion authority (server
//!   drives authoritative motion; the client animates *to* it).
//!
//! # Honest gaps vs Unreal
//! Bevy core ships no state machines, blend trees, IK, or root motion. This crate
//! fills the blend primitives + data-authored graph, in-house two-bone/FABRIK IK,
//! and the VAT crowd-instancing math — the pieces the spec named community crates
//! for, implemented ourselves (ADR-0003) to avoid ecosystem lag on Bevy 0.19. FSM
//! transition logic layers on top and is not part of this batch. Motion is cosmetic
//! on the client — authoritative position comes from the server; animation
//! interpolates toward it, never asserts it.
//! → `docs/specs/game-engine/animation/README.md`.

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

pub mod error;
pub mod graph;
pub mod ik;
pub mod lod;
pub mod vat;

pub use error::AnimError;
pub use graph::{BlendGraph, BlendNode, BoneMask, ClipId, JointId, NodeId, Pose};
pub use ik::{align_rotation, solve_two_bone, IkChain, IkParams, IkSolution, TwoBoneSolution};
pub use lod::{AnimTier, LodThresholds};
pub use vat::{PlaybackMode, VatClip, VatLayout, VatSample};

/// Global animation configuration and runtime state.
#[derive(Resource, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Resource)]
pub struct AnimationConfig {
    /// Enable GPU skinning under the `render` feature (native heads only).
    pub skinning_enabled: bool,
    /// Distance thresholds and bone budget driving [`AnimTier`] selection.
    pub lod: LodThresholds,
}

// `skinning_enabled` defaults to whether the `render` feature is on, so this is
// not derivable — under `--no-default-features` the cfg folds to `false` (which
// *looks* derivable to clippy), but under `render` it is `true`.
#[allow(clippy::derivable_impls)]
impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            skinning_enabled: cfg!(feature = "render"),
            lod: LodThresholds::default(),
        }
    }
}

/// Engine animation plugin: data-authored blend graph, distance-tiered LOD, and
/// (under `render`) first-party `bevy_animation` playback + GPU skinning.
///
/// Headless-safe when the `render` feature is disabled — it only registers the
/// authored animation types for reflection and inserts [`AnimationConfig`]. The
/// pure blend/LOD logic is available in every build. Compose on top of
/// `omm_engine_core::EnginePlugins`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // Headless-safe, always: the authored animation types are reflected so the
        // inspector / MCP editor / agents can enumerate them in any build, and the
        // config resource exists to be read (pure data, no GPU).
        register_anim_types(app);
        app.init_resource::<AnimationConfig>();

        #[cfg(feature = "render")]
        {
            // First-party playback: AnimationPlayer/AnimationClip/AnimationGraph
            // systems + assets. Requires bevy_asset's AssetPlugin upstream (the
            // rendered client adds it); never instantiated in headless CI, which
            // runs the `render` feature for compile coverage only.
            if !app.is_plugin_added::<bevy_animation::AnimationPlugin>() {
                app.add_plugins(bevy_animation::AnimationPlugin);
            }
            // GPU linear-blend skinning component — reflected so tools can author it.
            app.register_type::<bevy_render::mesh::skinning::SkinnedMesh>();
        }
    }
}

/// Register the authored animation types with the app's reflection registry.
/// Headless-safe — pure reflection, no GPU — so it runs in every build and is
/// unit-testable without a device. An unregistered type is invisible to agents,
/// which is a bug.
fn register_anim_types(app: &mut App) {
    app.register_type::<AnimationConfig>()
        .register_type::<AnimTier>()
        .register_type::<LodThresholds>()
        .register_type::<ClipId>()
        .register_type::<JointId>()
        .register_type::<NodeId>()
        .register_type::<BoneMask>()
        .register_type::<BlendNode>()
        .register_type::<BlendGraph>()
        .register_type::<IkParams>()
        .register_type::<IkChain>()
        .register_type::<PlaybackMode>()
        .register_type::<VatClip>()
        .register_type::<VatLayout>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;

    #[test]
    fn plugin_constructs() {
        let _plugin = AnimationPlugin;
    }

    #[test]
    fn anim_types_are_registered() {
        // Exercise the registration path directly rather than adding the full
        // plugin: under `--all-features` the plugin's `render` branch adds the
        // first-party bevy_animation plugin, which needs an asset backend the
        // display-less CI runner does not wire up. Reflection registration is the
        // headless behavior we assert here.
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        register_anim_types(&mut app);

        let registry = app.world().resource::<AppTypeRegistry>().read();
        for type_id in [
            TypeId::of::<AnimationConfig>(),
            TypeId::of::<AnimTier>(),
            TypeId::of::<LodThresholds>(),
            TypeId::of::<ClipId>(),
            TypeId::of::<JointId>(),
            TypeId::of::<NodeId>(),
            TypeId::of::<BoneMask>(),
            TypeId::of::<BlendNode>(),
            TypeId::of::<BlendGraph>(),
            TypeId::of::<IkParams>(),
            TypeId::of::<IkChain>(),
            TypeId::of::<PlaybackMode>(),
            TypeId::of::<VatClip>(),
            TypeId::of::<VatLayout>(),
        ] {
            assert!(
                registry.get(type_id).is_some(),
                "an animation type is not registered"
            );
        }
    }

    #[test]
    fn anim_config_headless_defaults() {
        let config = AnimationConfig::default();
        assert_eq!(config.lod.mid_distance, 30.0);
        assert_eq!(config.lod.far_distance, 100.0);

        // Headless (no render feature): skinning_enabled == false.
        #[cfg(not(feature = "render"))]
        assert!(!config.skinning_enabled);
    }

    /// The headless build's plugin wiring: adding [`AnimationPlugin`] with the
    /// `render` feature off registers the types and inserts [`AnimationConfig`] —
    /// no GPU, no window. Gated off under `render` so `--all-features` never tries
    /// to add the device/asset-backed animation plugin on the display-less runner.
    #[cfg(not(feature = "render"))]
    #[test]
    fn headless_plugin_inserts_config() {
        let mut app = App::new();
        app.add_plugins(EnginePlugins).add_plugins(AnimationPlugin);
        assert!(
            app.world().get_resource::<AnimationConfig>().is_some(),
            "AnimationPlugin must insert the AnimationConfig resource"
        );
    }
}
