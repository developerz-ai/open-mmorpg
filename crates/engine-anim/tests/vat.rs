//! Vertex-animation-texture math tests — frame sampling (loop/once/ping-pong) and
//! texture layout (dimensions, flat index, sample UVs). Pure integer/`f32` math, so
//! these run under both `--no-default-features` (headless) and `--all-features`.

use bevy_math::Vec2;
use omm_engine_anim::vat::{PlaybackMode, VatClip, VatLayout};

const EPS: f32 = 1e-5;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

// ── clip validation ──────────────────────────────────────────────────────────

#[test]
fn clip_new_rejects_degenerate() {
    assert!(
        VatClip::new(0, 30.0, PlaybackMode::Loop).is_err(),
        "zero frames rejected"
    );
    assert!(
        VatClip::new(4, 0.0, PlaybackMode::Loop).is_err(),
        "zero fps rejected"
    );
    assert!(
        VatClip::new(4, -1.0, PlaybackMode::Loop).is_err(),
        "negative fps rejected"
    );
    assert!(
        VatClip::new(4, f32::NAN, PlaybackMode::Loop).is_err(),
        "NaN fps rejected"
    );
    assert!(VatClip::new(4, 30.0, PlaybackMode::Loop).is_ok());
}

#[test]
fn clip_duration_is_frames_over_fps() {
    let clip = VatClip::new(30, 30.0, PlaybackMode::Loop).expect("clip");
    assert!(approx(clip.duration_secs(), 1.0));
    assert_eq!(clip.frame_count(), 30);
    assert_eq!(clip.fps(), 30.0);
    assert_eq!(clip.mode(), PlaybackMode::Loop);
}

// ── loop sampling ────────────────────────────────────────────────────────────

#[allow(clippy::expect_used)] // static-valid test fixture
fn loop_clip() -> VatClip {
    VatClip::new(4, 1.0, PlaybackMode::Loop).expect("clip")
}

#[test]
fn loop_start_samples_frame_zero() {
    let s = loop_clip().sample_at(0.0);
    assert_eq!((s.frame_a, s.frame_b), (0, 1));
    assert!(approx(s.blend, 0.0));
}

#[test]
fn loop_midframe_blends_neighbors() {
    let s = loop_clip().sample_at(0.5);
    assert_eq!((s.frame_a, s.frame_b), (0, 1));
    assert!(approx(s.blend, 0.5));
}

#[test]
fn loop_last_frame_wraps_to_first() {
    // At 3.5s the lower frame is the last (3); the upper wraps back to 0.
    let s = loop_clip().sample_at(3.5);
    assert_eq!((s.frame_a, s.frame_b), (3, 0));
    assert!(approx(s.blend, 0.5));
}

#[test]
fn loop_wraps_past_the_end() {
    // 4.5s ≡ 0.5s into the next loop.
    let s = loop_clip().sample_at(4.5);
    assert_eq!((s.frame_a, s.frame_b), (0, 1));
    assert!(approx(s.blend, 0.5));
}

// ── once (clamp) sampling ────────────────────────────────────────────────────

#[allow(clippy::expect_used)] // static-valid test fixture
fn once_clip() -> VatClip {
    VatClip::new(4, 1.0, PlaybackMode::Once).expect("clip")
}

#[test]
fn once_holds_the_last_frame() {
    // Past the end clamps to the final frame with zero blend.
    let s = once_clip().sample_at(10.0);
    assert_eq!((s.frame_a, s.frame_b), (3, 3));
    assert!(approx(s.blend, 0.0));
}

#[test]
fn once_mid_playback_blends() {
    let s = once_clip().sample_at(2.5);
    assert_eq!((s.frame_a, s.frame_b), (2, 3));
    assert!(approx(s.blend, 0.5));
}

// ── ping-pong sampling ───────────────────────────────────────────────────────

#[allow(clippy::expect_used)] // static-valid test fixture
fn pingpong_clip() -> VatClip {
    VatClip::new(4, 1.0, PlaybackMode::PingPong).expect("clip")
}

#[test]
fn pingpong_peaks_then_reverses() {
    let clip = pingpong_clip();
    // Forward to the peak at t=3 (frame 3).
    let peak = clip.sample_at(3.0);
    assert_eq!(peak.frame_a, 3);
    // Just past the peak it walks back down: t=4.5 → frame 1.5 on the way back.
    let back = clip.sample_at(4.5);
    assert_eq!((back.frame_a, back.frame_b), (1, 2));
    assert!(approx(back.blend, 0.5));
}

#[test]
fn pingpong_completes_a_full_cycle() {
    // Period = 2·(4−1) = 6s; at 6s we are back at the start.
    let s = pingpong_clip().sample_at(6.0);
    assert_eq!((s.frame_a, s.frame_b), (0, 1));
    assert!(approx(s.blend, 0.0));
}

// ── edge cases ───────────────────────────────────────────────────────────────

#[test]
fn single_frame_clip_always_samples_frame_zero() {
    let clip = VatClip::new(1, 30.0, PlaybackMode::Loop).expect("clip");
    for &t in &[0.0, 1.0, 100.0] {
        let s = clip.sample_at(t);
        assert_eq!((s.frame_a, s.frame_b), (0, 0));
        assert!(approx(s.blend, 0.0));
    }
}

#[test]
fn non_finite_or_negative_time_clamps_to_start() {
    let clip = loop_clip();
    for &t in &[-5.0, f32::NAN, f32::NEG_INFINITY] {
        let s = clip.sample_at(t);
        assert_eq!((s.frame_a, s.frame_b), (0, 1));
        assert!(approx(s.blend, 0.0), "garbage time must clamp to frame 0");
    }
}

#[test]
fn sampling_is_deterministic() {
    let clip = loop_clip();
    assert_eq!(clip.sample_at(1.234), clip.sample_at(1.234));
}

// ── layout ───────────────────────────────────────────────────────────────────

#[test]
fn layout_new_rejects_zero_dimensions() {
    assert!(VatLayout::new(0, 8).is_err(), "zero vertices rejected");
    assert!(VatLayout::new(100, 0).is_err(), "zero frames rejected");
    assert!(VatLayout::new(100, 8).is_ok());
}

#[test]
fn layout_dimensions_and_counts() {
    let layout = VatLayout::new(100, 8).expect("layout");
    assert_eq!(layout.width(), 100);
    assert_eq!(layout.height(), 8);
    assert_eq!(layout.vertex_count(), 100);
    assert_eq!(layout.frame_count(), 8);
    assert_eq!(layout.texel_count(), 800);
}

#[test]
fn layout_fits_within_max_texture_size() {
    let layout = VatLayout::new(4096, 256).expect("layout");
    assert!(layout.fits(4096), "exactly at the limit fits");
    assert!(!layout.fits(2048), "over the limit does not fit");
}

#[test]
fn layout_texel_index_is_row_major() {
    let layout = VatLayout::new(100, 8).expect("layout");
    // index = frame·width + vertex.
    assert_eq!(layout.texel_index(5, 3).expect("index"), 305);
    assert_eq!(layout.texel_index(0, 0).expect("index"), 0);
    assert_eq!(layout.texel_index(99, 7).expect("index"), 799);
}

#[test]
fn layout_texel_index_bounds_are_fail_loud() {
    let layout = VatLayout::new(100, 8).expect("layout");
    assert!(layout.texel_index(100, 0).is_err(), "vertex out of range");
    assert!(layout.texel_index(0, 8).is_err(), "frame out of range");
}

#[test]
fn layout_texel_uv_is_the_texel_center() {
    let layout = VatLayout::new(4, 2).expect("layout");
    // (v+0.5)/w, (f+0.5)/h.
    let uv = layout.texel_uv(0, 0).expect("uv");
    assert!(
        uv.abs_diff_eq(Vec2::new(0.125, 0.25), EPS),
        "uv {uv:?} not centered"
    );
    let uv = layout.texel_uv(3, 1).expect("uv");
    assert!(
        uv.abs_diff_eq(Vec2::new(0.875, 0.75), EPS),
        "uv {uv:?} not centered"
    );
}

#[test]
fn layout_frame_v_is_the_row_center() {
    let layout = VatLayout::new(4, 2).expect("layout");
    assert!(approx(layout.frame_v(0).expect("v"), 0.25));
    assert!(approx(layout.frame_v(1).expect("v"), 0.75));
    assert!(layout.frame_v(2).is_err(), "frame out of range fails loud");
}
