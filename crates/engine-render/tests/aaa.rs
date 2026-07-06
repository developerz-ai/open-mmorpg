//! Integration tests for the **AAA hero-asset path** — exercises the *public API*
//! only; no `crate::` imports.
//!
//! CI verifies: compile + the pure hero-geometry decision (meshlet ⇔ Ultra tier
//! *and* a `meshlet` build) and its reflection registration. Actual meshlet cluster
//! culling / rendered frames need a Vulkan/Metal device and are verified in the
//! client-track / manual pass.
//!
//! # Feature gate
//! These run under `--no-default-features` (headless) and `--all-features` alike.
//! The decision is always headless; `meshlet` only adds the device wiring on top,
//! so `meshlet_compiled()` flips the Ultra path from baseline to meshlet.

use omm_engine_render::{meshlet_compiled, HeroGeometry, RenderTier};

// ── hero-geometry decision ──────────────────────────────────────────────────────

/// The one corner that lights up virtual geometry: Ultra tier on a build that
/// compiled the meshlet backend.
#[test]
fn meshlet_only_on_ultra_with_a_meshlet_build() {
    assert_eq!(
        HeroGeometry::select(RenderTier::Ultra, true),
        HeroGeometry::Meshlet
    );
    assert_eq!(
        HeroGeometry::select(RenderTier::Ultra, false),
        HeroGeometry::DiscreteLod
    );
}

/// Meshlet is Ultra-only — High/Web always degrade to discrete LOD, whatever the
/// build offers.
#[test]
fn non_ultra_tiers_always_degrade_to_discrete_lod() {
    for tier in [RenderTier::High, RenderTier::Web] {
        for available in [true, false] {
            assert_eq!(
                HeroGeometry::select(tier, available),
                HeroGeometry::DiscreteLod,
                "{tier:?} must degrade to discrete LOD (meshlet is Ultra-only)"
            );
        }
    }
}

/// The decision is an AND gate over (Ultra tier, meshlet build) — `is_meshlet`
/// agrees only on that single corner.
#[test]
fn meshlet_requires_both_tier_and_build() {
    assert!(HeroGeometry::select(RenderTier::Ultra, true).is_meshlet());
    assert!(!HeroGeometry::select(RenderTier::Ultra, false).is_meshlet());
    assert!(!HeroGeometry::select(RenderTier::High, true).is_meshlet());
    assert!(!HeroGeometry::select(RenderTier::Web, true).is_meshlet());
}

/// `for_current_build` folds in this build's compiled capability, so the Ultra path
/// resolves to meshlet iff the crate was built with `--features meshlet`.
#[test]
fn for_current_build_tracks_the_compiled_feature() {
    let expected = if meshlet_compiled() {
        HeroGeometry::Meshlet
    } else {
        HeroGeometry::DiscreteLod
    };
    assert_eq!(HeroGeometry::for_current_build(RenderTier::Ultra), expected);
    assert_eq!(
        HeroGeometry::for_current_build(RenderTier::Web),
        HeroGeometry::DiscreteLod
    );
}

// ── reflection registration ────────────────────────────────────────────────────

/// The hero-asset config types must be registered for reflection — an unregistered
/// type is invisible to the MCP editor and agents. Gated to headless builds: adding
/// the plugin under `--features render` would create a GPU device the display-less
/// CI runner has no window for; registration itself is headless behavior.
#[cfg(not(feature = "render"))]
#[test]
fn hero_asset_types_are_reflected() {
    use bevy_app::App;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;
    use omm_engine_render::{EngineRenderPlugin, HeroAsset};

    let mut app = App::new();
    app.add_plugins((EnginePlugins, EngineRenderPlugin));

    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, type_id) in [
        ("HeroAsset", TypeId::of::<HeroAsset>()),
        ("HeroGeometry", TypeId::of::<HeroGeometry>()),
    ] {
        assert!(
            registry.get(type_id).is_some(),
            "{name} must be registered for reflection"
        );
    }
}
