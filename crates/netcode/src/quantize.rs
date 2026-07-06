//! Quantization: shrink positions and angles before send. Fiedler reports ~98%
//! state reduction from quantize + delta + bit-pack; here we do the numeric part
//! (mm-quantized positions, angle packed to `i16`), leaving actual bit-packing to
//! the transport codec. Dequantize is lossy but bounded to sub-perceptible error.

use omm_protocol::Vec3;

/// Millimetres per world unit — positions round to the nearest millimetre.
const MM_PER_UNIT: f32 = 1000.0;

/// Quantize a position to integer millimetres (`i32` per axis).
#[must_use]
pub fn quantize_pos(v: Vec3) -> [i32; 3] {
    [
        (v.x * MM_PER_UNIT).round() as i32,
        (v.y * MM_PER_UNIT).round() as i32,
        (v.z * MM_PER_UNIT).round() as i32,
    ]
}

/// Reconstruct a position from integer millimetres. Error ≤ 0.5 mm per axis.
#[must_use]
pub fn dequantize_pos(q: [i32; 3]) -> Vec3 {
    Vec3 {
        x: q[0] as f32 / MM_PER_UNIT,
        y: q[1] as f32 / MM_PER_UNIT,
        z: q[2] as f32 / MM_PER_UNIT,
    }
}

/// Pack an angle in radians into an `i16` covering a full turn. Wraps naturally.
#[must_use]
pub fn quantize_angle(radians: f32) -> i16 {
    let turns = radians / core::f32::consts::TAU;
    let frac = turns - turns.floor(); // 0..1
    (frac * 65536.0).round() as i32 as u16 as i16
}

/// Reconstruct an angle in radians (range `0..TAU`) from its packed form.
#[must_use]
pub fn dequantize_angle(packed: i16) -> f32 {
    let unit = f32::from(packed as u16) / 65536.0;
    unit * core::f32::consts::TAU
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn position_roundtrips_within_a_millimetre() {
        let v = Vec3 {
            x: 12.3456,
            y: -7.891,
            z: 0.0005,
        };
        let back = dequantize_pos(quantize_pos(v));
        assert!((back.x - v.x).abs() <= 0.0005);
        assert!((back.y - v.y).abs() <= 0.0005);
        assert!((back.z - v.z).abs() <= 0.0005);
    }

    proptest! {
        #[test]
        fn position_error_bounded(x in -1000.0f32..1000.0, z in -1000.0f32..1000.0) {
            let v = Vec3 { x, y: 0.0, z };
            let back = dequantize_pos(quantize_pos(v));
            prop_assert!((back.x - x).abs() <= 0.0006);
            prop_assert!((back.z - z).abs() <= 0.0006);
        }

        #[test]
        fn angle_error_bounded(a in 0.0f32..core::f32::consts::TAU) {
            let back = dequantize_angle(quantize_angle(a));
            // The codec wraps at a full turn, so error must be measured on the
            // circle, not the line: an `a` just below TAU quantizes to 0 (≡ TAU),
            // a tiny angular error but a ~TAU linear one. Fold the difference into
            // `[0, TAU)` and take the shorter arc. One i16 step is TAU/65536 rad;
            // allow one step of slack.
            let raw = (back - a).rem_euclid(core::f32::consts::TAU);
            let circular = raw.min(core::f32::consts::TAU - raw);
            prop_assert!(circular <= core::f32::consts::TAU / 32768.0);
        }
    }
}
