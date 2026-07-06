//! Integration tests for spatial-audio math: **attenuation curves** and **stereo
//! pan from azimuth angle**.
//!
//! These tests exercise only the public crate API (`omm_engine_audio::*`). No
//! audio device is opened — safe under CI `--all-features`.

use bevy_math::Vec3;
use omm_engine_audio::{spatialize, Attenuation, Listener, Rolloff, SpatialMix};

const TOL: f32 = 1e-4;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < TOL
}

// Standard Bevy basis: origin, facing −Z, up +Y.  Right axis → +X.
fn listener() -> Listener {
    Listener::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y).expect("valid listener")
}

fn atten(rolloff: Rolloff) -> Attenuation {
    Attenuation::new(2.0, 10.0, rolloff).expect("valid attenuation")
}

// ── Attenuation curve ────────────────────────────────────────────────────────

/// All three rolloff curves must be exactly 1.0 at `min_distance` and exactly
/// 0.0 at (and beyond) `max_distance`.
#[test]
fn attenuation_boundaries_for_all_rolloffs() {
    for rolloff in [Rolloff::Linear, Rolloff::Inverse, Rolloff::InverseSquare] {
        let a = atten(rolloff);
        // Within min_distance → full gain.
        assert!(
            approx(a.gain(0.0), 1.0),
            "{rolloff:?}: gain at 0 should be 1.0"
        );
        assert!(
            approx(a.gain(a.min_distance), 1.0),
            "{rolloff:?}: gain at min_distance should be 1.0"
        );
        // At and beyond max_distance → silence.
        assert!(
            approx(a.gain(a.max_distance), 0.0),
            "{rolloff:?}: gain at max_distance should be 0.0"
        );
        assert!(
            approx(a.gain(a.max_distance + 100.0), 0.0),
            "{rolloff:?}: gain past max_distance should be 0.0"
        );
        // Non-finite distance → silence.
        assert!(
            approx(a.gain(f32::NAN), 0.0),
            "{rolloff:?}: gain at NaN should be 0.0"
        );
        assert!(
            approx(a.gain(f32::INFINITY), 0.0),
            "{rolloff:?}: gain at ∞ should be 0.0"
        );
    }
}

/// Every rolloff must decrease monotonically from `min_distance` to
/// `max_distance` and stay in `0.0..=1.0`.
#[test]
fn attenuation_curves_are_monotonically_decreasing() {
    let steps = 100u16;
    for rolloff in [Rolloff::Linear, Rolloff::Inverse, Rolloff::InverseSquare] {
        let a = atten(rolloff);
        let span = a.max_distance - a.min_distance;
        let mut prev = a.gain(a.min_distance);
        for i in 1..=steps {
            let d = a.min_distance + f32::from(i) / f32::from(steps) * span;
            let g = a.gain(d);
            assert!(
                g <= prev + TOL,
                "{rolloff:?}: gain rose at d={d:.3} ({g:.5} > {prev:.5})"
            );
            assert!((0.0..=1.0).contains(&g), "{rolloff:?}: gain {g} out of 0..1");
            prev = g;
        }
    }
}

/// InverseSquare must be the steepest, Linear the shallowest, at mid-range.
#[test]
fn rolloff_steepness_order_at_midrange() {
    let d = 5.0; // midpoint of [2..10]
    let lin = atten(Rolloff::Linear).gain(d);
    let inv = atten(Rolloff::Inverse).gain(d);
    let sq = atten(Rolloff::InverseSquare).gain(d);
    assert!(sq < inv, "InverseSquare {sq:.4} should be < Inverse {inv:.4}");
    assert!(inv < lin, "Inverse {inv:.4} should be < Linear {lin:.4}");
}

/// `Attenuation::new` validates its inputs.
#[test]
fn attenuation_new_rejects_invalid_inputs() {
    // Non-finite
    assert!(Attenuation::new(f32::NAN, 10.0, Rolloff::Linear).is_err());
    assert!(Attenuation::new(2.0, f32::INFINITY, Rolloff::Linear).is_err());
    // Negative min
    assert!(Attenuation::new(-1.0, 10.0, Rolloff::Linear).is_err());
    // min >= max
    assert!(Attenuation::new(10.0, 10.0, Rolloff::Linear).is_err());
    assert!(Attenuation::new(11.0, 10.0, Rolloff::Linear).is_err());
    // Valid zero-min
    assert!(Attenuation::new(0.0, 10.0, Rolloff::Linear).is_ok());
}

// ── Pan from azimuth angle ───────────────────────────────────────────────────

/// Listener faces −Z, right is +X.  An emitter directly to the right (+X)
/// should pan to +1; directly left (−X) to −1; straight ahead to 0.
#[test]
fn pan_at_cardinal_azimuths() {
    let l = listener();
    let a = atten(Rolloff::Linear);

    let right_mix = spatialize(&l, Vec3::new(5.0, 0.0, 0.0), &a);
    assert!(
        right_mix.pan > 0.99,
        "pure right should be near +1.0, got {}",
        right_mix.pan
    );

    let left_mix = spatialize(&l, Vec3::new(-5.0, 0.0, 0.0), &a);
    assert!(
        left_mix.pan < -0.99,
        "pure left should be near −1.0, got {}",
        left_mix.pan
    );

    let ahead_mix = spatialize(&l, Vec3::new(0.0, 0.0, -5.0), &a);
    assert!(
        approx(ahead_mix.pan, 0.0),
        "straight ahead should be centred, got {}",
        ahead_mix.pan
    );

    let behind_mix = spatialize(&l, Vec3::new(0.0, 0.0, 5.0), &a);
    assert!(
        approx(behind_mix.pan, 0.0),
        "straight behind should be centred, got {}",
        behind_mix.pan
    );
}

/// Pan at +45° (northeast in XZ) must be positive and less than hard-right.
#[test]
fn pan_at_diagonal_azimuth_is_intermediate() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    // 45° right of ahead, same Y — the projection onto right (+X) is sin(45°)≈0.707.
    let diag = spatialize(&l, Vec3::new(5.0, 0.0, -5.0), &a);
    let expected = std::f32::consts::FRAC_1_SQRT_2;
    assert!(
        (diag.pan - expected).abs() < TOL,
        "45° pan: expected ≈{expected:.4}, got {}",
        diag.pan
    );
}

/// Pan must always stay within [−1.0, 1.0] regardless of position.
#[test]
fn pan_is_always_bounded() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    for pos in [
        Vec3::new(1000.0, 500.0, 200.0),
        Vec3::new(-1000.0, -200.0, 100.0),
        Vec3::new(0.001, 500.0, -0.001),
    ] {
        let SpatialMix { pan, .. } = spatialize(&l, pos, &a);
        assert!(
            (-1.0..=1.0).contains(&pan),
            "pan out of bounds at {pos}: {pan}"
        );
    }
}

/// A co-located emitter (exactly at listener) gets centred pan and full gain.
#[test]
fn colocated_emitter_is_centred_full_gain() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    let SpatialMix { gain, pan, distance } = spatialize(&l, Vec3::ZERO, &a);
    assert!(approx(pan, 0.0), "co-located pan should be 0: {pan}");
    assert!(approx(gain, 1.0), "co-located gain should be 1: {gain}");
    assert!(approx(distance, 0.0), "co-located distance should be 0: {distance}");
}

/// A non-finite emitter position must produce gain=0 and centred pan.
#[test]
fn nonfinite_emitter_is_silent_and_centred() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    for bad in [
        Vec3::new(f32::NAN, 0.0, 0.0),
        Vec3::new(0.0, f32::INFINITY, 0.0),
        Vec3::new(0.0, 0.0, f32::NEG_INFINITY),
    ] {
        let SpatialMix { gain, pan, .. } = spatialize(&l, bad, &a);
        assert!(approx(gain, 0.0), "NaN/∞ emitter gain should be 0: {gain}");
        assert!(approx(pan, 0.0), "NaN/∞ emitter pan should be 0: {pan}");
    }
}

/// `Listener::new` rejects degenerate bases.
#[test]
fn listener_rejects_degenerate_basis() {
    // NaN position
    assert!(Listener::new(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::NEG_Z, Vec3::Y).is_err());
    // Zero forward
    assert!(Listener::new(Vec3::ZERO, Vec3::ZERO, Vec3::Y).is_err());
    // Zero up
    assert!(Listener::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::ZERO).is_err());
    // Forward parallel to up → no right axis
    assert!(Listener::new(Vec3::ZERO, Vec3::Y, Vec3::Y).is_err());
}

/// Euclidean distance from the listener is reported correctly.
#[test]
fn spatialize_distance_is_euclidean() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    let SpatialMix { distance, .. } = spatialize(&l, Vec3::new(3.0, 4.0, 0.0), &a);
    assert!(approx(distance, 5.0), "3-4-5 triangle, got {distance}");
}
