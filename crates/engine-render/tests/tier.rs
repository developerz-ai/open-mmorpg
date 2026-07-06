//! Integration tests for the render tier ladder — exercises the **public API**
//! only; no `crate::` imports.
//!
//! CI verifies: compile + pure-logic tier selection, capability checks, and the
//! feature matrix. Actual rendered frames are verified in the client-track /
//! manual test pass (GPU required).
//!
//! # Feature gate
//! These tests run under `--no-default-features` (headless) and `--all-features`
//! alike. Tier selection is always headless; the `render` feature only adds the
//! device pipeline on top.

use omm_engine_render::{AntiAliasing, GlobalIllumination, GpuCapabilities, RenderTier};

// ── default-capability floor ──────────────────────────────────────────────────

/// An un-probed device must never over-claim. The safe floor is always `Web`;
/// a client that fails to probe capabilities falls back gracefully.
#[test]
fn default_capabilities_resolve_to_web() {
    let tier = RenderTier::select(&GpuCapabilities::default());
    assert_eq!(tier, RenderTier::Web);
}

/// `GpuCapabilities::default()` and `web_baseline()` are the same state — there
/// is no invisible difference between "not probed" and "explicitly WebGPU."
#[test]
fn default_caps_equal_web_baseline() {
    assert_eq!(GpuCapabilities::default(), GpuCapabilities::web_baseline());
}

// ── tier selection ─────────────────────────────────────────────────────────────

#[test]
fn ultra_caps_select_ultra() {
    assert_eq!(
        RenderTier::select(&GpuCapabilities::native_ultra()),
        RenderTier::Ultra
    );
}

#[test]
fn high_caps_select_high() {
    assert_eq!(
        RenderTier::select(&GpuCapabilities::native_high()),
        RenderTier::High
    );
}

#[test]
fn web_caps_select_web() {
    assert_eq!(
        RenderTier::select(&GpuCapabilities::web_baseline()),
        RenderTier::Web
    );
}

/// Each Ultra prerequisite is load-bearing: drop any one and the tier falls to High.
#[test]
fn missing_one_ultra_prerequisite_falls_to_high() {
    let tweaks: [fn(&mut GpuCapabilities); 4] = [
        |c| c.compute = false,
        |c| c.bindless = false,
        |c| c.multi_draw_indirect = false,
        |c| c.ray_tracing = false,
    ];
    for tweak in tweaks {
        let mut caps = GpuCapabilities::native_ultra();
        tweak(&mut caps);
        assert_eq!(
            RenderTier::select(&caps),
            RenderTier::High,
            "missing prerequisite must fall to High, not Ultra"
        );
    }
}

/// Advanced flags without a native backend must not promote the tier — a browser
/// that reports vendor extensions is still a `Web` device.
#[test]
fn vendor_flags_without_native_backend_stay_web() {
    let caps = GpuCapabilities {
        native_backend: false,
        compute: true,
        bindless: true,
        multi_draw_indirect: true,
        ray_tracing: true,
    };
    assert_eq!(RenderTier::select(&caps), RenderTier::Web);
}

// ── capability validation ──────────────────────────────────────────────────────

#[test]
fn web_tier_is_universally_supported() {
    for caps in [
        GpuCapabilities::web_baseline(),
        GpuCapabilities::native_high(),
        GpuCapabilities::native_ultra(),
    ] {
        RenderTier::Web
            .ensure_supported_by(&caps)
            .expect("Web tier must be supported by any device");
    }
}

#[test]
fn high_tier_requires_native_backend() {
    let err = RenderTier::High
        .ensure_supported_by(&GpuCapabilities::web_baseline())
        .expect_err("web baseline cannot run High");
    assert!(
        err.to_string().contains("native_backend"),
        "error must name the missing capability, got: {err}"
    );
}

#[test]
fn ultra_requires_ray_tracing_on_otherwise_capable_device() {
    let err = RenderTier::Ultra
        .ensure_supported_by(&GpuCapabilities::native_high())
        .expect_err("native-high device cannot run Ultra");
    assert!(
        err.to_string().contains("ray_tracing"),
        "error must name the missing capability, got: {err}"
    );
}

#[test]
fn ultra_reports_first_missing_capability_in_priority_order() {
    // Priority: native_backend > compute > bindless > multi_draw_indirect > ray_tracing.
    let err = RenderTier::Ultra
        .ensure_supported_by(&GpuCapabilities::web_baseline())
        .expect_err("web baseline cannot run Ultra");
    assert!(
        err.to_string().contains("native_backend"),
        "first gap must be native_backend, got: {err}"
    );
}

#[test]
fn fully_capable_device_satisfies_every_tier() {
    let ultra = GpuCapabilities::native_ultra();
    for tier in [RenderTier::Web, RenderTier::High, RenderTier::Ultra] {
        tier.ensure_supported_by(&ultra)
            .expect("ultra caps must satisfy any tier");
    }
}

// ── feature matrix ────────────────────────────────────────────────────────────

#[test]
fn anti_aliasing_matches_tier_spec() {
    assert_eq!(RenderTier::Ultra.anti_aliasing(), AntiAliasing::Dlss);
    assert_eq!(RenderTier::High.anti_aliasing(), AntiAliasing::Taa);
    assert_eq!(RenderTier::Web.anti_aliasing(), AntiAliasing::Smaa);
}

#[test]
fn global_illumination_matches_tier_spec() {
    assert_eq!(
        RenderTier::Ultra.global_illumination(),
        GlobalIllumination::Realtime,
    );
    assert_eq!(
        RenderTier::High.global_illumination(),
        GlobalIllumination::Baked,
    );
    assert_eq!(
        RenderTier::Web.global_illumination(),
        GlobalIllumination::Baked,
    );
}

#[test]
fn virtual_geometry_is_ultra_only() {
    assert!(
        RenderTier::Ultra.virtual_geometry(),
        "Ultra must enable virtual geometry"
    );
    assert!(
        !RenderTier::High.virtual_geometry(),
        "High must not enable virtual geometry"
    );
    assert!(
        !RenderTier::Web.virtual_geometry(),
        "Web must not enable virtual geometry"
    );
}

#[test]
fn tier_order_is_richest_to_leanest() {
    assert!(
        RenderTier::Ultra < RenderTier::High,
        "Ultra should sort before High"
    );
    assert!(
        RenderTier::High < RenderTier::Web,
        "High should sort before Web"
    );
}

// ── tier degrade ladder ─────────────────────────────────────────────────────────

/// A hero asset that *wants* Ultra steps down to the richest tier the device runs,
/// walking Ultra → High → Web — the graceful counterpart to `ensure_supported_by`.
#[test]
fn ultra_desire_degrades_to_the_supported_tier() {
    assert_eq!(
        RenderTier::Ultra.degrade_to_supported(&GpuCapabilities::native_ultra()),
        RenderTier::Ultra
    );
    assert_eq!(
        RenderTier::Ultra.degrade_to_supported(&GpuCapabilities::native_high()),
        RenderTier::High
    );
    assert_eq!(
        RenderTier::Ultra.degrade_to_supported(&GpuCapabilities::web_baseline()),
        RenderTier::Web
    );
}

/// Degrade only steps *down*: a leaner desire is honored even on the richest GPU,
/// so a forced-Web config never gets silently upgraded to Ultra.
#[test]
fn degrade_never_upgrades_a_leaner_desire() {
    let ultra = GpuCapabilities::native_ultra();
    assert_eq!(
        RenderTier::Web.degrade_to_supported(&ultra),
        RenderTier::Web
    );
    assert_eq!(
        RenderTier::High.degrade_to_supported(&ultra),
        RenderTier::High
    );
}

/// Asking for the top tier is exactly "give me the best this device runs" — the
/// degrade of Ultra must equal `select` for every capability set.
#[test]
fn degrade_of_ultra_equals_select() {
    for caps in [
        GpuCapabilities::native_ultra(),
        GpuCapabilities::native_high(),
        GpuCapabilities::web_baseline(),
    ] {
        assert_eq!(
            RenderTier::Ultra.degrade_to_supported(&caps),
            RenderTier::select(&caps)
        );
    }
}

// ── reflection registration ────────────────────────────────────────────────────

/// All tier types must be registered in the app's reflection registry.
/// An unregistered type is invisible to the MCP editor and agents — that is a bug.
///
/// Gated to headless builds: when `--features render` is on, adding the plugin
/// creates a GPU device the display-less CI runner does not have. The reflection
/// registrations are headless behavior — the `render` feature only adds the device
/// pipeline on top.
#[cfg(not(feature = "render"))]
#[test]
fn tier_types_are_reflected() {
    use bevy_app::App;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;
    use omm_engine_render::EngineRenderPlugin;

    let mut app = App::new();
    app.add_plugins((EnginePlugins, EngineRenderPlugin));

    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, type_id) in [
        ("RenderTier", TypeId::of::<RenderTier>()),
        ("AntiAliasing", TypeId::of::<AntiAliasing>()),
        ("GlobalIllumination", TypeId::of::<GlobalIllumination>()),
        ("GpuCapabilities", TypeId::of::<GpuCapabilities>()),
    ] {
        assert!(
            registry.get(type_id).is_some(),
            "{name} must be registered for reflection"
        );
    }
}
