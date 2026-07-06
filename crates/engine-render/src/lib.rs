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
