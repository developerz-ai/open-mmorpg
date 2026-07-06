//! Integration tests for the **tier fallback ladder** — Ultra → High → Web.
//!
//! This file has two jobs:
//!
//! 1. **Fallback correctness** — exhaustively verify every step of the
//!    `Ultra → High → Web` degrade chain for all capability combinations.  The
//!    companion `tier.rs` covers the underlying `select` / `ensure_supported_by`
//!    API; this file covers the *end-to-end degrade path* that a hero asset or
//!    shader variant takes at runtime.
//!
//! 2. **`--all-features` headless compile assertion** — prove that compiling with
//!    `meshlet` (which is the only feature included in `--all-features` beyond
//!    `render`) does not pull GPU code into headless-safe paths. The test itself
//!    runs fine on a display-less CI runner; the assertion is the **compile step**.
//!
//! # ADR reference
//! See `docs/architecture/decisions/0002-aaa-render-tiers.md`:
//! - DLSS / Solari are *documented downstream overlays* — zero SDK code here, so
//!   `--all-features` is always clean.
//! - `meshlet` is MIT-safe and included in `--all-features`; its decision logic
//!   (`meshlet_compiled()`, `HeroGeometry::select`) is headless.
//!
//! # Feature gate note
//! All tests run under **both** `--no-default-features` (headless) and
//! `--all-features` (meshlet on). When the meshlet feature is on, the only
//! behavioural difference is that `meshlet_compiled()` returns `true` and
//! `HeroGeometry::for_current_build(Ultra)` returns `Meshlet`; everything else
//! is identical.

use omm_engine_render::{meshlet_compiled, GpuCapabilities, HeroGeometry, RenderTier};

// ── Ultra → High → Web: step-by-step degrade ─────────────────────────────────

/// Ultra capability set → Ultra.  No degrade occurs on the richest device.
#[test]
fn ultra_device_serves_ultra() {
    assert_eq!(
        RenderTier::Ultra.degrade_to_supported(&GpuCapabilities::native_ultra()),
        RenderTier::Ultra,
    );
}

/// High capability set → High when desire is Ultra.
#[test]
fn ultra_desire_on_high_device_degrades_to_high() {
    assert_eq!(
        RenderTier::Ultra.degrade_to_supported(&GpuCapabilities::native_high()),
        RenderTier::High,
    );
}

/// Web baseline → Web when desire is Ultra.  Two steps down in one jump.
#[test]
fn ultra_desire_on_web_device_degrades_to_web() {
    assert_eq!(
        RenderTier::Ultra.degrade_to_supported(&GpuCapabilities::web_baseline()),
        RenderTier::Web,
    );
}

/// High desire on a High device → High.
#[test]
fn high_desire_on_high_device_serves_high() {
    assert_eq!(
        RenderTier::High.degrade_to_supported(&GpuCapabilities::native_high()),
        RenderTier::High,
    );
}

/// High desire on a Web device → Web (one step down).
#[test]
fn high_desire_on_web_device_degrades_to_web() {
    assert_eq!(
        RenderTier::High.degrade_to_supported(&GpuCapabilities::web_baseline()),
        RenderTier::Web,
    );
}

/// Web desire is always served — no device is too lean for Web.
#[test]
fn web_desire_is_always_served() {
    for caps in [
        GpuCapabilities::web_baseline(),
        GpuCapabilities::native_high(),
        GpuCapabilities::native_ultra(),
    ] {
        assert_eq!(
            RenderTier::Web.degrade_to_supported(&caps),
            RenderTier::Web,
            "Web desire must always resolve to Web regardless of device"
        );
    }
}

// ── degrade is monotonically non-increasing ───────────────────────────────────

/// For any fixed capability set, `degrade_to_supported` never returns a richer
/// tier than the desire.  No device promotes a forced-Web config to Ultra.
#[test]
fn degrade_never_upgrades_across_all_cap_sets() {
    let cap_sets = [
        GpuCapabilities::web_baseline(),
        GpuCapabilities::native_high(),
        GpuCapabilities::native_ultra(),
    ];
    let desires = [RenderTier::Web, RenderTier::High, RenderTier::Ultra];

    for caps in &cap_sets {
        for &desire in &desires {
            let result = desire.degrade_to_supported(caps);
            // "richest first" ordering: Ultra < High < Web (as PartialOrd).
            // result must be ≥ desire in the same ordering, i.e. never richer.
            assert!(
                result >= desire,
                "degrade_to_supported returned a richer tier than desired: \
                 desire={desire:?} result={result:?} caps={caps:?}"
            );
        }
    }
}

/// `degrade_to_supported(Ultra, caps)` must equal `RenderTier::select(caps)` for
/// every capability set — "give me the best you can run" is the definition of
/// `select`.
#[test]
fn ultra_degrade_equals_select_for_every_cap_set() {
    for caps in [
        GpuCapabilities::web_baseline(),
        GpuCapabilities::native_high(),
        GpuCapabilities::native_ultra(),
    ] {
        assert_eq!(
            RenderTier::Ultra.degrade_to_supported(&caps),
            RenderTier::select(&caps),
            "Ultra.degrade_to_supported must equal RenderTier::select for caps={caps:?}"
        );
    }
}

// ── meshlet: decision logic is always headless-safe ───────────────────────────
//
// The tests below compile and run under *both* `--no-default-features` and
// `--all-features`.  Under `--all-features`, `meshlet_compiled()` returns `true`
// and the meshlet hero-geometry path is active; the test assertions branch on
// `meshlet_compiled()` so they are correct in either build.
//
// The compile step itself is the key assertion for the `--all-features` headless
// guarantee: if any meshlet path accidentally pulled GPU code into a headless
// symbol the test binary would fail to link on a display-less runner.

/// `meshlet_compiled()` reflects the build flag deterministically.
///
/// - Under `--no-default-features`: `false`.
/// - Under `--features meshlet` or `--all-features`: `true`.
///
/// If this test panics the build-flag plumbing is broken.
#[test]
fn meshlet_compiled_matches_feature_flag() {
    // cfg(feature = "meshlet") is compiled in from the same feature gate —
    // these two must always agree.
    let expected = cfg!(feature = "meshlet");
    assert_eq!(
        meshlet_compiled(),
        expected,
        "meshlet_compiled() must agree with cfg!(feature = \"meshlet\")"
    );
}

/// Ultra tier with a meshlet build → `Meshlet`; without → `DiscreteLod`.
/// Non-Ultra tiers always return `DiscreteLod`.
///
/// This runs headlessly even when `--features meshlet` is active: the *decision*
/// is a pure function; the GPU cluster-culling code is compiled but not called.
#[test]
fn hero_geometry_decision_is_headless_under_all_features() {
    let ultra = RenderTier::Ultra;
    let compiled = meshlet_compiled();

    let expected_ultra = if compiled {
        HeroGeometry::Meshlet
    } else {
        HeroGeometry::DiscreteLod
    };
    assert_eq!(
        HeroGeometry::for_current_build(ultra),
        expected_ultra,
        "HeroGeometry::for_current_build(Ultra) must agree with meshlet_compiled()"
    );

    // Non-Ultra is always DiscreteLod whatever the build.
    for tier in [RenderTier::High, RenderTier::Web] {
        assert_eq!(
            HeroGeometry::for_current_build(tier),
            HeroGeometry::DiscreteLod,
            "{tier:?} must always degrade to DiscreteLod"
        );
    }
}

/// Full ladder: Ultra desire on Ultra/High/Web devices, with the current build's
/// hero-geometry decision, returns the right (tier, geometry) pair.
///
/// This is the end-to-end scenario a renderer calls once per hero asset load:
/// pick the best tier, then pick the geometry mode for that tier on this build.
#[test]
fn full_fallback_chain_with_hero_geometry() {
    let compiled = meshlet_compiled();

    // (caps, expected_tier, expected_geometry)
    let cases = [
        (
            GpuCapabilities::native_ultra(),
            RenderTier::Ultra,
            if compiled {
                HeroGeometry::Meshlet
            } else {
                HeroGeometry::DiscreteLod
            },
        ),
        (
            GpuCapabilities::native_high(),
            RenderTier::High,
            HeroGeometry::DiscreteLod,
        ),
        (
            GpuCapabilities::web_baseline(),
            RenderTier::Web,
            HeroGeometry::DiscreteLod,
        ),
    ];

    for (caps, expected_tier, expected_geo) in cases {
        let tier = RenderTier::Ultra.degrade_to_supported(&caps);
        let geo = HeroGeometry::for_current_build(tier);
        assert_eq!(tier, expected_tier, "wrong tier for caps={caps:?}");
        assert_eq!(geo, expected_geo, "wrong geometry for tier={tier:?}");
    }
}

// ── DLSS / Solari: documented slots, no SDK symbols ───────────────────────────
//
// ADR-0002 §1: DLSS and Solari are "documented downstream overlays" — the
// `AntiAliasing::Dlss` and `GlobalIllumination::Realtime` variants exist as named
// slots but carry zero proprietary SDK code. The tests below confirm the slot
// semantics compile and evaluate correctly under `--all-features`.

use omm_engine_render::{AntiAliasing, GlobalIllumination};

/// Ultra tier exposes the DLSS slot for anti-aliasing. This is a *named slot*
/// only — the SDK wire is a downstream operator overlay (see ADR-0002 §1).
/// The variant exists, compiles, and round-trips through the enum.
#[test]
fn ultra_anti_aliasing_slot_is_dlss() {
    assert_eq!(RenderTier::Ultra.anti_aliasing(), AntiAliasing::Dlss);
    // The slot value is stable and printable (Debug derive — agents enumerate it).
    assert_eq!(format!("{:?}", AntiAliasing::Dlss), "Dlss");
}

/// Ultra tier exposes the Solari (real-time GI) slot. Named slot, no NVIDIA SDK.
#[test]
fn ultra_global_illumination_slot_is_realtime() {
    assert_eq!(
        RenderTier::Ultra.global_illumination(),
        GlobalIllumination::Realtime
    );
    assert_eq!(format!("{:?}", GlobalIllumination::Realtime), "Realtime");
}

/// High and Web fall back to Baked GI — the Solari slot is Ultra-only.
#[test]
fn non_ultra_gi_is_baked() {
    assert_eq!(
        RenderTier::High.global_illumination(),
        GlobalIllumination::Baked
    );
    assert_eq!(
        RenderTier::Web.global_illumination(),
        GlobalIllumination::Baked
    );
}

/// High falls back to TAA (vendor-agnostic temporal AA) — not DLSS.
#[test]
fn high_aa_is_taa_not_dlss() {
    assert_eq!(RenderTier::High.anti_aliasing(), AntiAliasing::Taa);
}
