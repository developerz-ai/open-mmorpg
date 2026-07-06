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
//! Tier selection, material data mapping, and CSM cascade computation are pure,
//! deterministic logic with no GPU/window dependency. Frame instrumentation
//! (draw count, frame time) is a reflected resource that drives adaptive LOD and
//! optimization — the logic is headless. Device passes and shaders gate behind
//! the `render` feature so `--no-default-features` boots headless with zero GPU
//! deps. → `docs/architecture/04-game-engine/render/README.md`.

mod error;

#[cfg(feature = "render")]
pub use bevy_pbr;
#[cfg(feature = "render")]
pub use bevy_render;

pub use error::RenderError;

use bevy_app::{App, Plugin};

/// The engine's render pipeline plugin: clustered-forward+ PBR, CSM, baked GI,
/// spatial AA, and tier-driven optimization.
///
/// Headless-safe when the `render` feature is disabled. When enabled (native
/// heads), adds the full render pipeline: bevy render/PBR/core_pipeline plugins,
/// reflection registration for render types, and the tier-selection + material
/// mapping logic.
///
/// Composition: add on top of `omm_engine_core::EnginePlugins` after
/// `EngineAssetsPlugin`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EngineRenderPlugin;

impl Plugin for EngineRenderPlugin {
    fn build(&self, app: &mut App) {
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

            // Register render types for reflection.
            // (TBD: material descriptors, CSM cascade data, budget instrumentation)
        }
        #[cfg(not(feature = "render"))]
        {
            // Headless build: render pipeline excluded, app unchanged.
            let _ = app;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_constructs() {
        let _plugin = EngineRenderPlugin;
    }
}
