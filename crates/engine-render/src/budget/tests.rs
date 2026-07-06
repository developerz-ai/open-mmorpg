use super::*;

fn approx(got: f32, want: f32) {
    let tol = 1e-3 * want.abs().max(1.0);
    assert!((got - want).abs() <= tol, "expected {want}, got {got}");
}

#[test]
fn default_budget_targets_sixty_fps() {
    let budget = FrameBudget::default();
    approx(budget.target_frame_time_ms, 1000.0 / 60.0);
    budget.validate().expect("the default budget is achievable");
}

#[test]
fn for_fps_sets_the_frame_time_target() {
    approx(
        FrameBudget::for_fps(30.0).target_frame_time_ms,
        1000.0 / 30.0,
    );
    approx(
        FrameBudget::for_fps(120.0).target_frame_time_ms,
        1000.0 / 120.0,
    );
    // A non-positive rate collapses to a target validate() then rejects.
    assert_eq!(FrameBudget::for_fps(0.0).target_frame_time_ms, 0.0);
}

#[test]
fn unusable_budgets_fail_loud() {
    let cases: [(&str, FrameBudget); 4] = [
        ("zero fps target", FrameBudget::for_fps(0.0)),
        (
            "nan target",
            FrameBudget {
                target_frame_time_ms: f32::NAN,
                ..FrameBudget::default()
            },
        ),
        (
            "zero draw ceiling",
            FrameBudget {
                max_draw_calls: 0,
                ..FrameBudget::default()
            },
        ),
        (
            "zero triangle ceiling",
            FrameBudget {
                max_triangles: 0,
                ..FrameBudget::default()
            },
        ),
    ];
    for (name, budget) in cases {
        let err = budget.validate().expect_err(&format!("{name} must fail"));
        assert!(
            matches!(err, RenderError::InstrumentationAnomaly { .. }),
            "{name}: expected InstrumentationAnomaly, got {err:?}"
        );
    }
}

#[test]
fn within_budget_sample_passes_every_ceiling() {
    let budget = FrameBudget::default();
    let sample = FrameSample {
        frame_time_ms: 10.0,
        draw_calls: 100,
        triangles: 1_000,
    };
    assert!(budget.is_within(&sample));
    budget.check(&sample).expect("sample is within budget");
}

#[test]
fn check_names_the_first_ceiling_breached() {
    let budget = FrameBudget::default();

    let slow = FrameSample {
        frame_time_ms: 20.0,
        ..FrameSample::default()
    };
    assert!(!budget.is_within(&slow));
    let err = budget.check(&slow).expect_err("frame time over budget");
    assert!(err.to_string().contains("frame_time"), "got: {err}");

    let draw_heavy = FrameSample {
        frame_time_ms: 10.0,
        draw_calls: budget.max_draw_calls + 1,
        triangles: 0,
    };
    let err = budget
        .check(&draw_heavy)
        .expect_err("draw calls over budget");
    assert!(err.to_string().contains("draw_calls"), "got: {err}");

    let tri_heavy = FrameSample {
        frame_time_ms: 10.0,
        draw_calls: 0,
        triangles: budget.max_triangles + 1,
    };
    let err = budget.check(&tri_heavy).expect_err("triangles over budget");
    assert!(err.to_string().contains("triangles"), "got: {err}");
}

#[test]
fn frame_sample_reports_instantaneous_fps() {
    approx(
        FrameSample {
            frame_time_ms: 20.0,
            ..FrameSample::default()
        }
        .fps(),
        50.0,
    );
    // A zero frame time yields 0 fps rather than dividing by zero.
    assert_eq!(FrameSample::default().fps(), 0.0);
}

#[test]
fn first_sample_seeds_the_moving_average() {
    let mut resource = RenderBudget::default();
    resource.record(FrameSample {
        frame_time_ms: 10.0,
        ..FrameSample::default()
    });
    // No lag-up from zero: the average equals the first sample exactly.
    approx(resource.average_frame_time_ms, 10.0);
    assert_eq!(resource.frames_recorded, 1);
}

#[test]
fn moving_average_smooths_subsequent_samples() {
    let mut resource = RenderBudget::default();
    resource.record(FrameSample {
        frame_time_ms: 10.0,
        ..FrameSample::default()
    });
    resource.record(FrameSample {
        frame_time_ms: 20.0,
        ..FrameSample::default()
    });
    // EMA with alpha 0.1: 0.1·20 + 0.9·10 = 11.
    approx(resource.average_frame_time_ms, 11.0);
    approx(resource.average_fps(), 1000.0 / 11.0);
    assert_eq!(resource.frames_recorded, 2);
}

#[test]
fn over_budget_frames_are_tallied() {
    let mut resource = RenderBudget::default();
    // One good frame, one that misses the 60 FPS deadline.
    resource.record(FrameSample {
        frame_time_ms: 10.0,
        ..FrameSample::default()
    });
    resource.record(FrameSample {
        frame_time_ms: 40.0,
        ..FrameSample::default()
    });
    assert_eq!(resource.over_budget_frames, 1);
    approx(resource.over_budget_ratio(), 0.5);
    assert!(!resource.last_within_budget(), "last frame was over budget");
}

#[test]
fn empty_history_is_within_budget() {
    let resource = RenderBudget::default();
    assert!(resource.last_within_budget());
    assert_eq!(resource.over_budget_ratio(), 0.0);
    assert_eq!(resource.average_fps(), 0.0);
}

#[test]
fn reset_clears_history_but_keeps_the_budget() {
    let budget = FrameBudget::for_fps(30.0);
    let mut resource = RenderBudget::new(budget);
    resource.record(FrameSample {
        frame_time_ms: 50.0,
        ..FrameSample::default()
    });
    resource.reset();

    assert_eq!(resource.frames_recorded, 0);
    assert_eq!(resource.over_budget_frames, 0);
    assert_eq!(resource.average_frame_time_ms, 0.0);
    // The configured ceiling survives the reset.
    assert_eq!(resource.budget, budget);
}

#[test]
fn new_carries_the_budget_with_empty_history() {
    let budget = FrameBudget::for_fps(144.0);
    let resource = RenderBudget::new(budget);
    assert_eq!(resource.budget, budget);
    assert_eq!(resource.frames_recorded, 0);
}
