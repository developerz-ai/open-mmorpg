use super::*;

/// A mutation that makes one [`CsmConfig`] field illegal, for table-driven tests.
type ConfigTweak = fn(&mut CsmConfig);

/// Assert `got ≈ want` with a tolerance that scales with magnitude.
fn approx(got: f32, want: f32) {
    let tol = 1e-3 * want.abs().max(1.0);
    assert!((got - want).abs() <= tol, "expected {want}, got {got}");
}

#[test]
fn default_config_is_valid_and_covers_full_range() {
    let config = CsmConfig::default();
    let splits = config.compute_splits().expect("default config is valid");
    let intervals = splits.intervals();

    assert_eq!(intervals.len(), config.cascade_count as usize);
    // The cascade set spans the whole configured view depth range.
    approx(intervals[0].near, config.near_distance);
    approx(intervals[intervals.len() - 1].far, config.far_distance);
}

#[test]
fn default_produces_expected_practical_splits() {
    // near=0.1, far=1000 → far/near = 1e4, so the log terms land on clean powers of
    // ten; lambda=0.5 averages them with the uniform distribution.
    let splits = CsmConfig::default().compute_splits().expect("valid");
    let far: Vec<f32> = splits.intervals().iter().map(|c| c.far).collect();
    approx(far[0], 125.5375);
    approx(far[1], 255.025);
    approx(far[2], 425.0125);
    approx(far[3], 1000.0);
}

#[test]
fn cascades_are_ordered_and_finite() {
    let splits = CsmConfig::default().compute_splits().expect("valid");
    let intervals = splits.intervals();
    for cascade in intervals {
        assert!(cascade.near.is_finite() && cascade.far.is_finite());
        assert!(cascade.near < cascade.far, "cascade must be non-degenerate");
    }
    // Far bounds strictly increase; near bounds too.
    for pair in intervals.windows(2) {
        assert!(pair[0].far < pair[1].far, "far bounds must increase");
        assert!(pair[0].near < pair[1].near, "near bounds must increase");
    }
}

#[test]
fn last_cascade_far_is_pinned_to_far_distance() {
    // Not just approximately — the last far bound is the exact far plane, no drift.
    let config = CsmConfig::default();
    let splits = config.compute_splits().expect("valid");
    let last = splits.intervals().last().expect("at least one cascade");
    assert_eq!(last.far, config.far_distance);
}

#[test]
fn overlap_pulls_near_edges_back_into_previous_cascade() {
    let config = CsmConfig::default();
    let splits = config.compute_splits().expect("valid");
    let intervals = splits.intervals();
    for i in 1..intervals.len() {
        // With 20% overlap, cascade i starts inside cascade i-1's range.
        assert!(
            intervals[i].near < intervals[i - 1].far,
            "cascades must overlap for blending"
        );
        approx(
            intervals[i].near,
            intervals[i - 1].far * (1.0 - config.overlap_proportion),
        );
    }
}

#[test]
fn zero_overlap_makes_cascades_meet_exactly() {
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
            "no overlap → cascades meet at a seam"
        );
    }
}

#[test]
fn pure_logarithmic_split_is_geometric() {
    let config = CsmConfig {
        split_lambda: 1.0,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    let far: Vec<f32> = splits.intervals().iter().map(|c| c.far).collect();
    // 0.1 · 10000^(i/4) → 1, 10, 100, 1000.
    approx(far[0], 1.0);
    approx(far[1], 10.0);
    approx(far[2], 100.0);
    approx(far[3], 1000.0);
}

#[test]
fn pure_uniform_split_is_arithmetic() {
    let config = CsmConfig {
        split_lambda: 0.0,
        ..CsmConfig::default()
    };
    let splits = config.compute_splits().expect("valid");
    let far: Vec<f32> = splits.intervals().iter().map(|c| c.far).collect();
    // near + (far-near)·(i/4).
    approx(far[0], 250.075);
    approx(far[1], 500.05);
    approx(far[2], 750.025);
    approx(far[3], 1000.0);
}

#[test]
fn single_cascade_covers_the_whole_range() {
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
fn for_tier_matches_the_tier_ladder() {
    // Native tiers get the full cascade ladder; the web baseline gets fewer, shorter.
    let ultra = CsmConfig::for_tier(RenderTier::Ultra);
    let high = CsmConfig::for_tier(RenderTier::High);
    let web = CsmConfig::for_tier(RenderTier::Web);

    assert_eq!(ultra, CsmConfig::default());
    assert_eq!(high, CsmConfig::default());
    assert_eq!(web.cascade_count, 2);
    assert_eq!(web.far_distance, 300.0);

    // Every tier's preset must produce valid splits.
    for tier in [RenderTier::Ultra, RenderTier::High, RenderTier::Web] {
        CsmConfig::for_tier(tier)
            .compute_splits()
            .expect("tier preset must be valid");
    }
}

#[test]
fn cascade_interval_reports_its_depth_range() {
    let interval = CascadeInterval {
        near: 10.0,
        far: 42.0,
    };
    approx(interval.depth_range(), 32.0);
}

#[test]
fn invalid_configs_fail_loud() {
    // Each tweak makes exactly one parameter illegal; all must be rejected with an
    // InvalidCsmConfig rather than silently emitting NaN or a degenerate cascade.
    let base = CsmConfig::default();
    let cases: [(&str, ConfigTweak); 13] = [
        ("zero cascades", |c| c.cascade_count = 0),
        ("too many cascades", |c| {
            c.cascade_count = (MAX_CASCADES + 1) as u8
        }),
        ("zero near", |c| c.near_distance = 0.0),
        ("negative near", |c| c.near_distance = -1.0),
        ("nan near", |c| c.near_distance = f32::NAN),
        ("far equals near", |c| c.far_distance = c.near_distance),
        ("far below near", |c| c.far_distance = c.near_distance - 1.0),
        ("infinite far", |c| c.far_distance = f32::INFINITY),
        ("lambda above one", |c| c.split_lambda = 1.1),
        ("lambda below zero", |c| c.split_lambda = -0.1),
        ("nan lambda", |c| c.split_lambda = f32::NAN),
        ("overlap at one", |c| c.overlap_proportion = 1.0),
        ("negative overlap", |c| c.overlap_proportion = -0.1),
    ];
    for (name, tweak) in cases {
        let mut config = base;
        tweak(&mut config);
        let err = config
            .compute_splits()
            .expect_err(&format!("{name} must be rejected"));
        assert!(
            matches!(err, RenderError::InvalidCsmConfig { .. }),
            "{name}: expected InvalidCsmConfig, got {err:?}"
        );
    }
}

#[test]
fn valid_boundary_values_are_accepted() {
    // The inclusive/exclusive edges of each range are legal where the spec says so.
    let cases: [ConfigTweak; 5] = [
        |c| c.cascade_count = MAX_CASCADES as u8, // upper cascade bound
        |c| c.cascade_count = 1,                  // lower cascade bound
        |c| c.split_lambda = 0.0,                 // lambda floor
        |c| c.split_lambda = 1.0,                 // lambda ceiling (inclusive)
        |c| c.overlap_proportion = 0.0,           // overlap floor
    ];
    for tweak in cases {
        let mut config = CsmConfig::default();
        tweak(&mut config);
        config.validate().expect("boundary value must be valid");
    }
}
