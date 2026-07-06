//! **Engine render pipeline** — baseline PBR rendering with clustered-forward+,
//! cascaded shadow maps, baked GI integration, and spatial anti-aliasing.
//!
//! # Architecture
//! The render layer is optional and gates behind the `render` feature (native
//! heads only). Headless builds (`--no-default-features`) exclude it entirely.
//! Pure logic (tier selection, material mapping, CSM math, instrumentation) runs
//! headless and testable; device code (GPU passes, shaders) compiles only under
//! the feature.
//!
//! # Headless-first
//! [Tier selection](tier) and [material data mapping](material) are pure,
//! deterministic logic with no GPU/window dependency — an agent reasons about the
//! render tier ladder and validates material data in a headless harness exactly as
//! the rendered client does. Under the `render` feature, [`PbrMaterial`] gains a
//! [`to_standard_material`](PbrMaterial::to_standard_material) conversion and
//! [`EngineRenderPlugin`] adds the device pipeline; `--no-default-features` boots
//! headless with zero GPU deps. Either way the render-config types are registered
//! for reflection so tools and agents can enumerate them.
//! → `docs/specs/game-engine/rendering/README.md`.
//!
//! # What CI verifies vs what it does not
//!
//! **CI verifies (headless, every commit):**
//! - Compile under `--no-default-features` (headless): all pure logic compiles, no
//!   GPU dep leaks through.
//! - Compile under `--features render` (native): the feature gate itself compiles;
//!   no linker error from mis-wired optional deps.
//! - Tier-selection logic: `GpuCapabilities` → `RenderTier` is deterministic and
//!   matches the spec (all paths, boundary cases, precedence order).
//! - CSM split math: cascade near/far bounds are finite, ordered, and match the
//!   Zhang et al. practical-split formula to 0.1 % tolerance.
//! - Frame-budget logic: EMA seeding, smoothing (α=0.1), over-budget tallying, and
//!   determinism (same sequence → same aggregates).
//! - Reflection registration: every render-config type is in the app's
//!   `AppTypeRegistry` so the MCP editor and agents can enumerate them.
//! - `RenderBudget` resource is inserted by `EngineRenderPlugin` in headless mode.
//!
//! **CI does NOT verify (GPU required, client-track / manual):**
//! - Actual rendered frames — pixel correctness, shadow seam blending, AA quality.
//! - Real frame times — the EMA and over-budget tally are exercised with synthetic
//!   samples; wall-clock measurements require a real GPU tick loop.
//! - DLSS, TAA, SMAA output quality — verified against reference images in the
//!   client-track manual pass.
//! - GPU memory / VRAM budget — no GPU allocator in headless.
//! - Shader compilation — `wgpu` shaders are compiled on device; CI only sees the
//!   Rust side.
//!
//! # Honest gaps vs Unreal Engine
//!
//! This crate is honest about what it is: a **principled scaffold** for an
//! open-core MMORPG engine, not a decade of shipping AAA titles.
//!
//! | Capability | This crate | Unreal Engine |
//! |---|---|---|
//! | Tier ladder | 3 tiers: Web/High/Ultra; capability-driven selection | Scalability groups (low → epic) with per-CVars |
//! | GI | Baked irradiance volumes (High/Web); Solari RT (Ultra, NVIDIA-only) | Lumen (HW RT + SW fallback, all vendors) |
//! | Geometry | Discrete LOD + imposters (High/Web); meshlet virtual geometry (Ultra) | Nanite (all DX12 targets) |
//! | Shadows | 4-cascade CSM + contact shadows; VSM tracked upstream | VSM (Virtual Shadow Maps) shipping |
//! | AA/Upscaling | SMAA (Web), TAA (High), DLSS (Ultra, NVIDIA) | TSR (Temporal Super Resolution, cross-vendor) |
//! | Frame budget | Headless instrumentation resource (EMA + over-budget tally) | Unreal Insights (full GPU/CPU profiler, stat commands) |
//! | Shader pipeline | wgpu (Vulkan/Metal/DX12/WebGPU) | Unreal's HLSL pipeline; manual per-vendor shader variants |
//!
//! The tier ladder and pure-logic modules are designed so that adding a new
//! technique (e.g. a VXGI baked-GI replacement) is a contained change in one module
//! with no cross-cutting effect — the architecture is the investment, not the
//! current technique list.

mod budget;
mod error;
mod material;
mod shadows;
mod tier;

#[cfg(feature = "render")]
pub use bevy_pbr;
#[cfg(feature = "render")]
pub use bevy_render;

pub use budget::{FrameBudget, FrameSample, RenderBudget};
pub use error::RenderError;
pub use material::{GltfMetallicRoughness, MaterialAlphaMode, PbrMaterial};
pub use shadows::{CascadeInterval, CascadeSplits, CsmConfig, MAX_CASCADES};
pub use tier::{AntiAliasing, GlobalIllumination, GpuCapabilities, RenderTier};

use bevy_app::{App, Plugin};

/// The engine's render pipeline plugin: clustered-forward+ PBR, CSM, baked GI,
/// spatial AA, and tier-driven optimization.
///
/// Headless-safe when the `render` feature is disabled. When enabled (native
/// heads), adds the full render pipeline: bevy render/PBR/core_pipeline plugins.
/// Either way it registers the render-config types (tiers, capabilities, material
/// data) for reflection — an unregistered type is invisible to agents, which is a
/// bug.
///
/// Composition: add on top of `omm_engine_core::EnginePlugins` after
/// `EngineAssetsPlugin`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EngineRenderPlugin;

impl Plugin for EngineRenderPlugin {
    fn build(&self, app: &mut App) {
        // Headless-safe, always: the render-config data types are reflected so the
        // inspector / MCP editor / agents can enumerate them in any build, and the
        // frame-budget instrumentation resource exists to be read (pure data, no GPU).
        register_render_types(app);
        app.init_resource::<RenderBudget>();

        #[cfg(feature = "render")]
        {
            use bevy_pbr::PbrPlugin;
            use bevy_render::RenderPlugin;

            // Render substrate: GPU dispatch, material system, core 3D pass.
            if !app.is_plugin_added::<RenderPlugin>() {
                app.add_plugins(RenderPlugin::default());
            }
            if !app.is_plugin_added::<PbrPlugin>() {
                app.add_plugins(PbrPlugin::default());
            }
        }
    }
}

/// Register the render-config types (tiers, capabilities, material data) with the
/// app's reflection registry. Headless-safe — pure reflection, no GPU — so it runs
/// in every build and is unit-testable without instantiating a device.
fn register_render_types(app: &mut App) {
    app.register_type::<RenderTier>()
        .register_type::<AntiAliasing>()
        .register_type::<GlobalIllumination>()
        .register_type::<GpuCapabilities>()
        .register_type::<MaterialAlphaMode>()
        .register_type::<GltfMetallicRoughness>()
        .register_type::<PbrMaterial>()
        .register_type::<CsmConfig>()
        .register_type::<CascadeInterval>()
        .register_type::<CascadeSplits>()
        .register_type::<FrameBudget>()
        .register_type::<FrameSample>()
        .register_type::<RenderBudget>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;

    #[test]
    fn plugin_constructs() {
        let _plugin = EngineRenderPlugin;
    }

    #[test]
    fn render_config_types_are_registered() {
        // Exercise the registration path directly rather than adding the full
        // plugin: under `--all-features` the plugin's `render` branch would create
        // a GPU device, which the headless CI runner has no display for. The
        // reflection registration is the headless behavior we assert here.
        let mut app = App::new();
        app.add_plugins(EnginePlugins);
        register_render_types(&mut app);

        let registry = app.world().resource::<AppTypeRegistry>().read();
        for type_id in [
            TypeId::of::<RenderTier>(),
            TypeId::of::<AntiAliasing>(),
            TypeId::of::<GlobalIllumination>(),
            TypeId::of::<GpuCapabilities>(),
            TypeId::of::<MaterialAlphaMode>(),
            TypeId::of::<GltfMetallicRoughness>(),
            TypeId::of::<PbrMaterial>(),
            TypeId::of::<CsmConfig>(),
            TypeId::of::<CascadeInterval>(),
            TypeId::of::<CascadeSplits>(),
            TypeId::of::<FrameBudget>(),
            TypeId::of::<FrameSample>(),
            TypeId::of::<RenderBudget>(),
        ] {
            assert!(
                registry.get(type_id).is_some(),
                "a render-config type is not registered"
            );
        }
    }

    /// The headless build's plugin wiring: adding [`EngineRenderPlugin`] with the
    /// `render` feature off must register the config types and insert the pure
    /// frame-budget resource — no GPU, no window. Gated off under `render` so
    /// `--all-features` never tries to create a device on the display-less runner.
    #[cfg(not(feature = "render"))]
    #[test]
    fn headless_plugin_inserts_budget_resource() {
        let mut app = App::new();
        app.add_plugins(EnginePlugins)
            .add_plugins(EngineRenderPlugin);
        assert!(
            app.world().get_resource::<RenderBudget>().is_some(),
            "EngineRenderPlugin must insert the RenderBudget instrumentation resource"
        );
    }
}
