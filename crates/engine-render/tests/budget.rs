//! Integration tests for frame-budget instrumentation — exercises the **public
//! API** only; no `crate::` imports.
//!
//! CI verifies: compile + pure-logic budget checks, EMA accuracy, over-budget
//! tallying, and reflection registration. Actual frame times are verified in the
//! client-track / manual test pass (GPU required).
//!
//! # Feature gate
//! The frame-budget resource is always headless — no `render` feature required.
//! These tests run under `--no-default-features` and `--all-features` alike.

use omm_engine_render::{FrameBudget, FrameSample, RenderBudget};

// ── helpers ───────────────────────────────────────────────────────────────────

fn approx(got: f32, want: f32) {
    let tol = 1e-3 * want.abs().max(1.0);
    assert!((got - want).abs() <= tol, "expected {want}, got {got}");
}

// ── FrameBudget ────────────────────────────────────────────────────────────────

#[test]
fn default_budget_targets_sixty_fps() {
    let budget = FrameBudget::default();
    approx(budget.target_frame_time_ms, 1000.0 / 60.0);
    budget.validate().expect("default budget must be valid");
}

#[test]
fn for_fps_converts_frame_rate_to_target_time() {
    approx(
        FrameBudget::for_fps(30.0).target_frame_time_ms,
        1000.0 / 30.0,
    );
    approx(
        FrameBudget::for_fps(120.0).target_frame_time_ms,
        1000.0 / 120.0,
    );
    approx(
        FrameBudget::for_fps(144.0).target_frame_time_ms,
        1000.0 / 144.0,
    );
}

#[test]
fn non_positive_fps_yields_target_zero_which_validate_rejects() {
    let budget = FrameBudget::for_fps(0.0);
    assert_eq!(budget.target_frame_time_ms, 0.0);
    assert!(budget.validate().is_err());
}

#[test]
fn validate_rejects_non_finite_frame_time() {
    let nan = FrameBudget {
        target_frame_time_ms: f32::NAN,
        ..FrameBudget::default()
    };
    assert!(nan.validate().is_err(), "NaN frame time must be rejected");
    let inf = FrameBudget {
        target_frame_time_ms: f32::INFINITY,
        ..FrameBudget::default()
    };
    assert!(
        inf.validate().is_err(),
        "infinite frame time must be rejected"
    );
}

#[test]
fn validate_rejects_zero_draw_and_triangle_ceilings() {
    let no_draws = FrameBudget {
        max_draw_calls: 0,
        ..FrameBudget::default()
    };
    assert!(no_draws.validate().is_err());

    let no_tris = FrameBudget {
        max_triangles: 0,
        ..FrameBudget::default()
    };
    assert!(no_tris.validate().is_err());
}

// ── is_within / check ─────────────────────────────────────────────────────────

#[test]
fn sample_within_every_ceiling_passes() {
    let budget = FrameBudget::default();
    let good = FrameSample {
        frame_time_ms: 10.0,
        draw_calls: 100,
        triangles: 1_000_000,
    };
    assert!(budget.is_within(&good));
    budget.check(&good).expect("sample is within budget");
}

#[test]
fn frame_time_breach_is_reported_first() {
    let budget = FrameBudget::default();
    let slow = FrameSample {
        frame_time_ms: 20.0,
        ..FrameSample::default()
    };
    assert!(!budget.is_within(&slow));
    let err = budget.check(&slow).expect_err("over frame-time budget");
    assert!(
        err.to_string().contains("frame_time"),
        "error must name frame_time, got: {err}"
    );
}

#[test]
fn draw_call_breach_is_reported_when_frame_time_is_fine() {
    let budget = FrameBudget::default();
    let draw_heavy = FrameSample {
        frame_time_ms: 10.0,
        draw_calls: budget.max_draw_calls + 1,
        triangles: 0,
    };
    let err = budget
        .check(&draw_heavy)
        .expect_err("over draw-call budget");
    assert!(
        err.to_string().contains("draw_calls"),
        "error must name draw_calls, got: {err}"
    );
}

#[test]
fn triangle_breach_is_reported_when_frame_and_draws_are_fine() {
    let budget = FrameBudget::default();
    let tri_heavy = FrameSample {
        frame_time_ms: 10.0,
        draw_calls: 0,
        triangles: budget.max_triangles + 1,
    };
    let err = budget.check(&tri_heavy).expect_err("over triangle budget");
    assert!(
        err.to_string().contains("triangles"),
        "error must name triangles, got: {err}"
    );
}

#[test]
fn sample_exactly_at_ceiling_is_within_budget() {
    let budget = FrameBudget::default();
    let at_limit = FrameSample {
        frame_time_ms: budget.target_frame_time_ms,
        draw_calls: budget.max_draw_calls,
        triangles: budget.max_triangles,
    };
    assert!(
        budget.is_within(&at_limit),
        "sample exactly at ceiling must be within budget"
    );
}

// ── FrameSample helpers ───────────────────────────────────────────────────────

#[test]
fn frame_sample_fps_is_inverse_of_frame_time() {
    let sample = FrameSample {
        frame_time_ms: 20.0,
        ..FrameSample::default()
    };
    approx(sample.fps(), 50.0);
}

#[test]
fn frame_sample_zero_time_yields_zero_fps() {
    assert_eq!(FrameSample::default().fps(), 0.0);
}

// ── RenderBudget ──────────────────────────────────────────────────────────────

#[test]
fn empty_history_is_within_budget() {
    let rb = RenderBudget::default();
    assert!(rb.last_within_budget());
    assert_eq!(rb.over_budget_ratio(), 0.0);
    assert_eq!(rb.average_fps(), 0.0);
    assert_eq!(rb.frames_recorded, 0);
}

#[test]
fn first_sample_seeds_the_moving_average_directly() {
    let mut rb = RenderBudget::default();
    rb.record(FrameSample {
        frame_time_ms: 10.0,
        ..FrameSample::default()
    });
    // No lag-up from 0: the EMA is seeded with the first sample.
    approx(rb.average_frame_time_ms, 10.0);
    assert_eq!(rb.frames_recorded, 1);
}

#[test]
fn ema_smooths_subsequent_samples_with_ten_percent_alpha() {
    let mut rb = RenderBudget::default();
    rb.record(FrameSample {
        frame_time_ms: 10.0,
        ..FrameSample::default()
    });
    rb.record(FrameSample {
        frame_time_ms: 20.0,
        ..FrameSample::default()
    });
    // EMA(α=0.1): 0.1×20 + 0.9×10 = 11.
    approx(rb.average_frame_time_ms, 11.0);
    approx(rb.average_fps(), 1000.0 / 11.0);
    assert_eq!(rb.frames_recorded, 2);
}

#[test]
fn over_budget_frames_are_tallied_correctly() {
    let mut rb = RenderBudget::default();
    // One good frame (10 ms < 16.67 ms), one over budget (40 ms).
    rb.record(FrameSample {
        frame_time_ms: 10.0,
        ..FrameSample::default()
    });
    rb.record(FrameSample {
        frame_time_ms: 40.0,
        ..FrameSample::default()
    });
    assert_eq!(rb.over_budget_frames, 1);
    approx(rb.over_budget_ratio(), 0.5);
    assert!(!rb.last_within_budget());
}

#[test]
fn all_good_frames_have_zero_over_budget_ratio() {
    let mut rb = RenderBudget::default();
    for _ in 0..5 {
        rb.record(FrameSample {
            frame_time_ms: 5.0,
            draw_calls: 10,
            triangles: 100,
        });
    }
    assert_eq!(rb.over_budget_frames, 0);
    assert_eq!(rb.over_budget_ratio(), 0.0);
    assert!(rb.last_within_budget());
}

#[test]
fn reset_clears_history_and_keeps_the_budget() {
    let budget = FrameBudget::for_fps(30.0);
    let mut rb = RenderBudget::new(budget);
    rb.record(FrameSample {
        frame_time_ms: 50.0,
        ..FrameSample::default()
    });
    rb.reset();

    assert_eq!(rb.frames_recorded, 0);
    assert_eq!(rb.over_budget_frames, 0);
    assert_eq!(rb.average_frame_time_ms, 0.0);
    assert_eq!(rb.budget, budget, "configured budget must survive reset");
}

#[test]
fn new_carries_only_the_budget_with_empty_history() {
    let budget = FrameBudget::for_fps(144.0);
    let rb = RenderBudget::new(budget);
    assert_eq!(rb.budget, budget);
    assert_eq!(rb.frames_recorded, 0);
    assert_eq!(rb.over_budget_frames, 0);
}

// ── determinism: same sequence → same aggregates ──────────────────────────────

/// The EMA is deterministic: replaying the same sample sequence twice on two
/// independent resources yields identical aggregates. This is the property that
/// lets a headless agent reason about frame cost identically to the live client.
#[test]
fn identical_sample_sequence_yields_identical_aggregates() {
    let samples = [
        FrameSample {
            frame_time_ms: 8.0,
            draw_calls: 500,
            triangles: 5_000_000,
        },
        FrameSample {
            frame_time_ms: 18.0,
            draw_calls: 9_000,
            triangles: 25_000_000,
        },
        FrameSample {
            frame_time_ms: 12.0,
            draw_calls: 2_000,
            triangles: 10_000_000,
        },
    ];

    let mut a = RenderBudget::default();
    let mut b = RenderBudget::default();
    for &s in &samples {
        a.record(s);
        b.record(s);
    }

    assert_eq!(a.average_frame_time_ms, b.average_frame_time_ms);
    assert_eq!(a.frames_recorded, b.frames_recorded);
    assert_eq!(a.over_budget_frames, b.over_budget_frames);
}

// ── reflection registration ────────────────────────────────────────────────────

/// Budget types must be registered; an unregistered resource is invisible to tools
/// and agents.
///
/// Gated to headless builds: when `--features render` is on, adding the plugin
/// creates a GPU device the display-less CI runner does not have. The headless path
/// is what we assert here — the `render` feature only wires the device pipeline on
/// top, not the reflection registrations.
#[cfg(not(feature = "render"))]
#[test]
fn budget_types_are_reflected() {
    use bevy_app::App;
    use bevy_ecs::prelude::AppTypeRegistry;
    use core::any::TypeId;
    use omm_engine_core::EnginePlugins;
    use omm_engine_render::EngineRenderPlugin;

    let mut app = App::new();
    app.add_plugins((EnginePlugins, EngineRenderPlugin));

    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, type_id) in [
        ("FrameBudget", TypeId::of::<FrameBudget>()),
        ("FrameSample", TypeId::of::<FrameSample>()),
        ("RenderBudget", TypeId::of::<RenderBudget>()),
    ] {
        assert!(
            registry.get(type_id).is_some(),
            "{name} must be registered for reflection"
        );
    }
}

/// The `RenderBudget` resource must be inserted by [`EngineRenderPlugin`], so tools
/// can read live frame cost from an ECS query even before the first rendered frame.
///
/// Gated to headless builds for the same reason as `budget_types_are_reflected`.
#[cfg(not(feature = "render"))]
#[test]
fn render_budget_resource_is_inserted_by_plugin() {
    use bevy_app::App;
    use omm_engine_core::EnginePlugins;
    use omm_engine_render::EngineRenderPlugin;

    let mut app = App::new();
    app.add_plugins((EnginePlugins, EngineRenderPlugin));

    assert!(
        app.world().get_resource::<RenderBudget>().is_some(),
        "EngineRenderPlugin must insert the RenderBudget resource"
    );
}
