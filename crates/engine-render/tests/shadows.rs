//! Integration tests for CSM (Cascaded Shadow Map) split math — exercises the
//! **public API** only; no `crate::` imports.
//!
//! CI verifies: compile + pure-logic cascade-split computation, boundary
//! validation, and tier presets. Actual shadow rendering is verified in the
//! client-track / manual test pass (GPU required).
//!
//! # Feature gate
//! CSM split math is always headless — no `render` feature required. These tests
//! run under `--no-default-features` and `--all-features` alike.

use omm_engine_render::{CascadeInterval, CsmConfig, RenderTier};

// Only used in the headless reflection test.
#[cfg(feature = "render")]
use omm_engine_render::MAX_CASCADES;
#[cfg(not(feature = "render"))]
use omm_engine_render::{CascadeSplits, MAX_CASCADES};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Approximate equality — tolerance scales with magnitude.
fn approx(got: f32, want: f32) {
    let tol = 1e-3 * want.abs().max(1.0);
    assert!((got - want).abs() <= tol, "expected {want}, got {got}");
}

// ── default config ─────────────────────────────────────────────────────────────

#[test]
fn default_config_is_valid_and_spans_the_full_range() {
    let config = CsmConfig::default();
    config.validate().expect("default config must be valid");

    let splits = config
        .compute_splits()
        .expect("default config must compute");
    let intervals = splits.intervals();

    assert_eq!(intervals.len(), config.cascade_count as usize);
    approx(intervals[0].near, config.near_distance);
    assert_eq!(
        intervals.last().unwrap().far,
        config.far_distance,
        "last cascade far must equal the configured far plane"
    );
}

#[test]
fn default_far_bounds_match_the_spec_values() {
    // near=0.1, far=1000, lambda=0.5: practical-split far bounds are deterministic.
    let splits = CsmConfig::default().compute_splits().expect("valid");
    let far: Vec<f32> = splits.intervals().iter().map(|c| c.far).collect();
    approx(far[0], 125.5375);
    approx(far[1], 255.025);
    approx(far[2], 425.0125);
    approx(far[3], 1000.0);
}

// ── structural invariants ──────────────────────────────────────────────────────

#[test]
fn all_intervals_are_finite_and_non_degenerate() {
    let splits = CsmConfig::default().compute_splits().expect("valid");
    for interval in splits.intervals() {
        assert!(
            interval.near.is_finite() && interval.far.is_finite(),
            "interval bounds must be finite"
        );
        assert!(
            interval.near < interval.far,
            "each cascade must be non-degenerate (near < far)"
        );
    }
}

#[test]
fn far_bounds_are_strictly_increasing() {
    let splits = CsmConfig::default().compute_splits().expect("valid");
    let intervals = splits.intervals();
    for pair in intervals.windows(2) {
        assert!(
            pair[0].far < pair[1].far,
            "far bounds must strictly increase across cascades"
        );
    }
}

#[test]
fn near_bounds_are_strictly_increasing() {
    let splits = CsmConfig::default().compute_splits().expect("valid");
    let intervals = splits.intervals();
    for pair in intervals.windows(2) {
        assert!(
            pair[0].near < pair[1].near,
            "near bounds must strictly increase across cascades"
        );
    }
}

#[test]
fn last_cascade_far_is_exactly_the_far_plane() {
    // No rounding drift: the final cascade is pinned to the exact far distance.
    let config = CsmConfig::default();
    let splits = config.compute_splits().expect("valid");
    let last = splits.intervals().last().expect("at least one cascade");
    assert_eq!(
        last.far, config.far_distance,
        "last cascade far must equal far_distance exactly"
    );
}

// ── overlap ────────────────────────────────────────────────────────────────────

#[test]
fn overlap_pulls_each_near_into_the_previous_cascade() {
    let config = CsmConfig::default(); // 20% overlap
    let splits = config.compute_splits().expect("valid");
    let intervals = splits.intervals();
    for i in 1..intervals.len() {
        assert!(
            intervals[i].near < intervals[i - 1].far,
            "cascades must overlap for blend bands"
        );
        approx(
            intervals[i].near,
            intervals[i - 1].far * (1.0 - config.overlap_proportion),
        );
    }
}

#[test]
fn zero_overlap_produces_hard_seams() {
    let config = CsmConfig {
        overlap_proportion: 0.0,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    let intervals = splits.intervals();
    for i in 1..intervals.len() {
        assert_eq!(
            intervals[i].near,
            intervals[i - 1].far,
            "zero overlap → cascades must meet at a seam"
        );
    }
}

// ── split distributions ────────────────────────────────────────────────────────

/// Fully logarithmic (lambda=1) → the far bounds form a geometric sequence.
#[test]
fn pure_logarithmic_split_is_geometric() {
    let config = CsmConfig {
        split_lambda: 1.0,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    let far: Vec<f32> = splits.intervals().iter().map(|c| c.far).collect();
    // 0.1 × (1000/0.1)^(i/4): 1, 10, 100, 1000.
    approx(far[0], 1.0);
    approx(far[1], 10.0);
    approx(far[2], 100.0);
    approx(far[3], 1000.0);
}

/// Fully uniform (lambda=0) → the far bounds are evenly spaced.
#[test]
fn pure_uniform_split_is_arithmetic() {
    let config = CsmConfig {
        split_lambda: 0.0,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    let far: Vec<f32> = splits.intervals().iter().map(|c| c.far).collect();
    // 0.1 + (1000 - 0.1) × (i/4): 250.075, 500.05, 750.025, 1000.
    approx(far[0], 250.075);
    approx(far[1], 500.05);
    approx(far[2], 750.025);
    approx(far[3], 1000.0);
}

// ── edge-case cascade counts ───────────────────────────────────────────────────

#[test]
fn single_cascade_spans_the_whole_range() {
    let config = CsmConfig {
        cascade_count: 1,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    let intervals = splits.intervals();
    assert_eq!(intervals.len(), 1);
    approx(intervals[0].near, config.near_distance);
    assert_eq!(intervals[0].far, config.far_distance);
}

#[test]
fn max_cascades_produces_correct_interval_count() {
    let config = CsmConfig {
        cascade_count: MAX_CASCADES as u8,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    assert_eq!(splits.intervals().len(), MAX_CASCADES);
}

// ── tier presets ───────────────────────────────────────────────────────────────

#[test]
fn tier_presets_produce_valid_splits() {
    for tier in [RenderTier::Ultra, RenderTier::High, RenderTier::Web] {
        let config = CsmConfig::for_tier(tier);
        config
            .validate()
            .unwrap_or_else(|e| panic!("{tier:?} preset must be valid: {e}"));
        config
            .compute_splits()
            .unwrap_or_else(|e| panic!("{tier:?} preset must compute splits: {e}"));
    }
}

#[test]
fn native_tiers_use_the_full_cascade_ladder() {
    for tier in [RenderTier::Ultra, RenderTier::High] {
        let config = CsmConfig::for_tier(tier);
        assert_eq!(
            config,
            CsmConfig::default(),
            "{tier:?} must use the full default cascade preset"
        );
    }
}

#[test]
fn web_tier_uses_fewer_shorter_cascades() {
    let web = CsmConfig::for_tier(RenderTier::Web);
    assert_eq!(web.cascade_count, 2, "Web uses 2 cascades");
    assert_eq!(web.far_distance, 300.0, "Web shadow reach is 300 m");
}

// ── validation ─────────────────────────────────────────────────────────────────

#[test]
fn invalid_configs_all_fail_loud() {
    type Tweak = fn(&mut CsmConfig);
    let base = CsmConfig::default();
    let cases: &[(&str, Tweak)] = &[
        ("zero cascades", |c| c.cascade_count = 0),
        ("cascade count above MAX", |c| {
            c.cascade_count = (MAX_CASCADES + 1) as u8
        }),
        ("zero near", |c| c.near_distance = 0.0),
        ("negative near", |c| c.near_distance = -1.0),
        ("NaN near", |c| c.near_distance = f32::NAN),
        ("far equals near", |c| c.far_distance = c.near_distance),
        ("far below near", |c| c.far_distance = c.near_distance - 1.0),
        ("infinite far", |c| c.far_distance = f32::INFINITY),
        ("lambda above 1", |c| c.split_lambda = 1.1),
        ("lambda below 0", |c| c.split_lambda = -0.1),
        ("NaN lambda", |c| c.split_lambda = f32::NAN),
        ("overlap at 1.0", |c| c.overlap_proportion = 1.0),
        ("negative overlap", |c| c.overlap_proportion = -0.1),
    ];
    for (label, tweak) in cases {
        let mut config = base;
        tweak(&mut config);
        let result = config.compute_splits();
        assert!(result.is_err(), "{label}: expected an error but got Ok");
    }
}

#[test]
fn valid_boundary_values_are_accepted() {
    type Tweak = fn(&mut CsmConfig);
    let cases: &[(&str, Tweak)] = &[
        ("cascade_count = 1", |c| c.cascade_count = 1),
        ("cascade_count = MAX_CASCADES", |c| {
            c.cascade_count = MAX_CASCADES as u8
        }),
        ("split_lambda = 0.0", |c| c.split_lambda = 0.0),
        ("split_lambda = 1.0", |c| c.split_lambda = 1.0),
        ("overlap_proportion = 0.0", |c| c.overlap_proportion = 0.0),
    ];
    for (label, tweak) in cases {
        let mut config = CsmConfig::default();
        tweak(&mut config);
        config
            .validate()
            .unwrap_or_else(|e| panic!("{label} should be valid: {e}"));
    }
}

// ── CascadeInterval helpers ────────────────────────────────────────────────────

#[test]
fn cascade_interval_depth_range_is_far_minus_near() {
    let interval = CascadeInterval {
        near: 10.0,
        far: 42.0,
    };
    approx(interval.depth_range(), 32.0);
}

// ── reflection registration ────────────────────────────────────────────────────

/// CSM types must be registered; an unregistered type is invisible to tools.
///
/// Gated to headless builds: when `--features render` is on, adding the plugin
/// creates a GPU device the display-less CI runner does not have. The reflection
/// registrations are headless behavior — the `render` feature only adds the device
/// pipeline on top.
#[cfg(not(feature = "render"))]
#[test]
fn csm_types_are_reflected() {
    use bevy_app::App;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;
    use omm_engine_render::EngineRenderPlugin;

    let mut app = App::new();
    app.add_plugins((EnginePlugins, EngineRenderPlugin));

    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, type_id) in [
        ("CsmConfig", TypeId::of::<CsmConfig>()),
        ("CascadeInterval", TypeId::of::<CascadeInterval>()),
        ("CascadeSplits", TypeId::of::<CascadeSplits>()),
    ] {
        assert!(
            registry.get(type_id).is_some(),
            "{name} must be registered for reflection"
        );
    }
}
