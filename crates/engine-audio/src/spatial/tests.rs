//! Unit tests for pure spatial-audio math.

use bevy_math::Vec3;

use super::*;

/// Absolute tolerance for gain/pan comparisons.
const TOL: f32 = 1e-4;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < TOL
}

fn atten(rolloff: Rolloff) -> Attenuation {
    Attenuation::new(2.0, 10.0, rolloff).expect("valid attenuation")
}

// Standard bevy basis: at the origin, facing -Z, up +Y. Right is then +X.
fn listener() -> Listener {
    Listener::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y).expect("valid listener")
}

#[test]
fn attenuation_new_rejects_bad_ranges() {
    assert!(Attenuation::new(f32::NAN, 10.0, Rolloff::Linear).is_err());
    assert!(Attenuation::new(2.0, f32::INFINITY, Rolloff::Linear).is_err());
    assert!(Attenuation::new(-1.0, 10.0, Rolloff::Linear).is_err());
    assert!(Attenuation::new(10.0, 10.0, Rolloff::Linear).is_err());
    assert!(Attenuation::new(11.0, 10.0, Rolloff::Linear).is_err());
    assert!(Attenuation::new(0.0, 10.0, Rolloff::Linear).is_ok());
}

#[test]
fn gain_is_full_within_min_and_silent_beyond_max() {
    for rolloff in [Rolloff::Linear, Rolloff::Inverse, Rolloff::InverseSquare] {
        let a = atten(rolloff);
        assert!(approx(a.gain(0.0), 1.0));
        assert!(approx(a.gain(2.0), 1.0)); // exactly at min
        assert!(approx(a.gain(10.0), 0.0)); // exactly at max
        assert!(approx(a.gain(100.0), 0.0));
        assert!(approx(a.gain(f32::NAN), 0.0));
    }
}

#[test]
fn gain_decreases_monotonically_between_edges() {
    for rolloff in [Rolloff::Linear, Rolloff::Inverse, Rolloff::InverseSquare] {
        let a = atten(rolloff);
        let mut prev = a.gain(2.0);
        for step in 1u16..=80 {
            let d = 2.0 + f32::from(step) * 0.1;
            let g = a.gain(d);
            assert!(g <= prev + TOL, "gain rose at d={d} ({g} > {prev})");
            assert!((0.0..=1.0).contains(&g), "gain out of range at d={d}: {g}");
            prev = g;
        }
    }
}

#[test]
fn inverse_square_is_quieter_than_inverse_than_linear_at_midrange() {
    let d = 5.0;
    let lin = atten(Rolloff::Linear).gain(d);
    let inv = atten(Rolloff::Inverse).gain(d);
    let sq = atten(Rolloff::InverseSquare).gain(d);
    assert!(sq < inv, "inverse-square {sq} should be < inverse {inv}");
    assert!(inv < lin, "inverse {inv} should be < linear {lin}");
}

#[test]
fn listener_new_rejects_degenerate_basis() {
    assert!(Listener::new(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::NEG_Z, Vec3::Y).is_err());
    assert!(Listener::new(Vec3::ZERO, Vec3::ZERO, Vec3::Y).is_err());
    assert!(Listener::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::ZERO).is_err());
    // forward parallel to up → no right axis.
    assert!(Listener::new(Vec3::ZERO, Vec3::Y, Vec3::Y).is_err());
}

#[test]
fn listener_right_is_plus_x_for_standard_basis() {
    let l = listener();
    assert!((l.right() - Vec3::X).length() < TOL);
}

#[test]
fn pan_places_emitter_left_and_right() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    // Emitter to the listener's right (+X) → pan > 0.
    let right = spatialize(&l, Vec3::new(3.0, 0.0, 0.0), &a);
    assert!(right.pan > 0.9, "expected hard right, got {}", right.pan);
    // Emitter to the left (-X) → pan < 0.
    let left = spatialize(&l, Vec3::new(-3.0, 0.0, 0.0), &a);
    assert!(left.pan < -0.9, "expected hard left, got {}", left.pan);
    // Straight ahead (-Z) → centred.
    let ahead = spatialize(&l, Vec3::new(0.0, 0.0, -3.0), &a);
    assert!(approx(ahead.pan, 0.0), "expected centre, got {}", ahead.pan);
}

#[test]
fn pan_is_bounded() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    for pos in [
        Vec3::new(100.0, 3.0, 2.0),
        Vec3::new(-100.0, -5.0, 4.0),
        Vec3::new(1.0, 50.0, -1.0),
    ] {
        let m = spatialize(&l, pos, &a);
        assert!((-1.0..=1.0).contains(&m.pan), "pan out of range: {}", m.pan);
    }
}

#[test]
fn colocated_emitter_is_centred_and_full_gain() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    let m = spatialize(&l, Vec3::ZERO, &a);
    assert!(approx(m.pan, 0.0));
    assert!(approx(m.gain, 1.0));
    assert!(approx(m.distance, 0.0));
}

#[test]
fn non_finite_emitter_is_silent() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    let m = spatialize(&l, Vec3::new(f32::NAN, 0.0, 0.0), &a);
    assert!(approx(m.gain, 0.0));
    assert!(approx(m.pan, 0.0));
    assert!(!m.distance.is_finite());
}

#[test]
fn distance_matches_euclidean() {
    let l = listener();
    let a = atten(Rolloff::Linear);
    let m = spatialize(&l, Vec3::new(3.0, 4.0, 0.0), &a);
    assert!(approx(m.distance, 5.0));
}
