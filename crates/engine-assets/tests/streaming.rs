//! Integration tests for the camera-position world streaming system — budget
//! eviction, load/unload lifecycles and determinism, via the public API only.
//!
//! These mirror how the client uses the streaming logic: pass a camera position
//! and a per-tile cost function, receive a [`StreamingDelta`] describing what
//! changed, then ask the asset server to act on it.

use bevy_math::Vec2;
use omm_engine_assets::{AssetError, StreamingDelta, StreamingGrid, TileCoord};

// ── helpers ───────────────────────────────────────────────────────────────────

fn coord(x: i32, y: i32) -> TileCoord {
    TileCoord { x, y }
}

fn uniform(bytes: u64) -> impl Fn(TileCoord) -> u64 {
    move |_| bytes
}

fn sorted_loaded(grid: &StreamingGrid) -> Vec<TileCoord> {
    grid.loaded_tiles().collect()
}

// ── construction guards ───────────────────────────────────────────────────────

#[test]
fn rejects_zero_tile_size() {
    assert!(matches!(
        StreamingGrid::new(0.0, 10.0, 1024),
        Err(AssetError::InvalidStreamingConfig { field: "tile_size" })
    ));
}

#[test]
fn rejects_negative_view_radius() {
    assert!(matches!(
        StreamingGrid::new(10.0, -1.0, 1024),
        Err(AssetError::InvalidStreamingConfig {
            field: "view_radius"
        })
    ));
}

#[test]
fn rejects_zero_budget() {
    assert!(matches!(
        StreamingGrid::new(10.0, 10.0, 0),
        Err(AssetError::InvalidStreamingConfig { field: "budget" })
    ));
}

#[test]
fn rejects_nan_tile_size() {
    assert!(matches!(
        StreamingGrid::new(f32::NAN, 10.0, 1024),
        Err(AssetError::InvalidStreamingConfig { .. })
    ));
}

// ── basic load / unload ───────────────────────────────────────────────────────

#[test]
fn streams_single_tile_within_tight_radius() {
    // radius 3 around tile (0,0)'s centre — no neighbour cell is within reach.
    let mut grid = StreamingGrid::new(10.0, 3.0, 1_000_000).expect("grid");
    let delta = grid
        .update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("update");
    assert_eq!(sorted_loaded(&grid), vec![coord(0, 0)]);
    assert_eq!(delta.loaded, vec![coord(0, 0)]);
    assert!(delta.unloaded.is_empty());
    assert!(delta.skipped.is_empty());
    assert_eq!(grid.resident_bytes(), 100);
}

#[test]
fn moving_camera_unloads_distant_tiles() {
    let mut grid = StreamingGrid::new(10.0, 3.0, 1_000_000).expect("grid");
    grid.update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("frame 1");
    assert!(grid.is_loaded(coord(0, 0)));

    let delta = grid
        .update(Vec2::new(105.0, 5.0), uniform(100))
        .expect("frame 2");
    assert!(!grid.is_loaded(coord(0, 0)), "distant tile must unload");
    assert!(grid.is_loaded(coord(10, 0)), "new tile must load");
    assert!(delta.unloaded.contains(&coord(0, 0)));
    assert!(delta.loaded.contains(&coord(10, 0)));
}

#[test]
fn zero_radius_still_loads_camera_tile() {
    let mut grid = StreamingGrid::new(10.0, 0.0, 1024).expect("grid");
    grid.update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("update");
    assert_eq!(sorted_loaded(&grid), vec![coord(0, 0)]);
}

// ── budget eviction ───────────────────────────────────────────────────────────

#[test]
fn budget_caps_residency_and_skips_excess() {
    // 5 tiles wanted; budget for only 4 × 100 = 400 bytes.
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("grid");
    let delta = grid
        .update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("update");

    assert_eq!(grid.resident_bytes(), 400, "never exceed the budget");
    assert_eq!(grid.loaded_tiles().count(), 4);
    assert!(
        !delta.skipped.is_empty(),
        "budget pressure must be reported, not hidden"
    );
    // The camera's tile is always resident.
    assert!(
        grid.is_loaded(grid.tile_at(Vec2::new(5.0, 5.0))),
        "camera tile must be resident"
    );
}

#[test]
fn nearer_tile_evicts_farther_resident_under_budget() {
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("grid");
    // Frame 1 at (5,5): fills 4 tiles (camera + 3 equidistant neighbours).
    grid.update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("frame 1");
    let before = sorted_loaded(&grid);
    assert_eq!(before.len(), 4);

    // Frame 2 at (5,-5): camera is now in (0,-1). Tiles nearer to the new camera
    // position must win the budget, evicting some previous residents.
    let delta = grid
        .update(Vec2::new(5.0, -5.0), uniform(100))
        .expect("frame 2");

    assert_eq!(grid.resident_bytes(), 400);
    assert_eq!(grid.loaded_tiles().count(), 4);
    // Something was evicted and something new was loaded.
    assert!(
        !delta.unloaded.is_empty(),
        "distant tiles must be evicted: {:?}",
        delta.unloaded
    );
    assert!(
        !delta.loaded.is_empty(),
        "nearer tiles must be loaded: {:?}",
        delta.loaded
    );
    // Camera's new tile is always resident.
    assert!(grid.is_loaded(coord(0, -1)));
}

#[test]
fn tile_larger_than_budget_fails_loud() {
    let mut grid = StreamingGrid::new(10.0, 1.0, 50).expect("grid");
    match grid.update(Vec2::new(5.0, 5.0), uniform(100)) {
        Err(AssetError::TileExceedsBudget { x, y, cost, budget }) => {
            assert_eq!((x, y), (0, 0));
            assert_eq!((cost, budget), (100, 50));
        }
        other => panic!("expected TileExceedsBudget, got {other:?}"),
    }
}

#[test]
fn variable_tile_costs_respected() {
    // Central tile costs 300; each neighbour costs 100. Budget = 400.
    // Only the central tile + one neighbour fit.
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("grid");
    let cost = |c: TileCoord| {
        if c == coord(0, 0) {
            300
        } else {
            100
        }
    };
    grid.update(Vec2::new(5.0, 5.0), cost).expect("update");
    assert!(grid.is_loaded(coord(0, 0)));
    assert_eq!(grid.resident_bytes(), 400); // 300 + one 100-byte neighbour
    assert_eq!(grid.loaded_tiles().count(), 2);
}

// ── delta structure ───────────────────────────────────────────────────────────

#[test]
fn delta_loaded_and_unloaded_are_sorted() {
    let mut grid = StreamingGrid::new(10.0, 25.0, 1_000_000).expect("grid");
    let d1 = grid
        .update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("frame 1");
    let d2 = grid
        .update(Vec2::new(105.0, 5.0), uniform(100))
        .expect("frame 2");

    // Every list the delta exposes must be in ascending coordinate order.
    let is_sorted = |v: &[TileCoord]| v.windows(2).all(|w| w[0] <= w[1]);
    assert!(is_sorted(&d1.loaded), "loaded not sorted: {:?}", d1.loaded);
    assert!(
        is_sorted(&d2.unloaded),
        "unloaded not sorted: {:?}",
        d2.unloaded
    );
    assert!(is_sorted(&d2.loaded), "loaded not sorted: {:?}", d2.loaded);
}

#[test]
fn delta_default_is_empty() {
    let d = StreamingDelta::default();
    assert!(d.loaded.is_empty());
    assert!(d.unloaded.is_empty());
    assert!(d.skipped.is_empty());
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn identical_inputs_yield_identical_residency_and_deltas() {
    let path = [
        Vec2::new(5.0, 5.0),
        Vec2::new(5.0, -5.0),
        Vec2::new(35.0, 12.0),
    ];
    let mut a = StreamingGrid::new(10.0, 25.0, 400).expect("grid a");
    let mut b = StreamingGrid::new(10.0, 25.0, 400).expect("grid b");
    for camera in path {
        let da = a.update(camera, uniform(100)).expect("a");
        let db = b.update(camera, uniform(100)).expect("b");
        assert_eq!(da, db, "same inputs must produce the same delta");
    }
    assert_eq!(sorted_loaded(&a), sorted_loaded(&b));
}

// ── tile_at ───────────────────────────────────────────────────────────────────

#[test]
fn tile_at_floors_into_integer_cells() {
    let grid = StreamingGrid::new(10.0, 5.0, 1024).expect("grid");
    assert_eq!(grid.tile_at(Vec2::new(5.0, 5.0)), coord(0, 0));
    assert_eq!(grid.tile_at(Vec2::new(-1.0, -1.0)), coord(-1, -1));
    assert_eq!(grid.tile_at(Vec2::new(23.0, -4.0)), coord(2, -1));
}

// ── budget accessor ───────────────────────────────────────────────────────────

#[test]
fn budget_bytes_accessor_matches_construction() {
    let grid = StreamingGrid::new(10.0, 5.0, 8192).expect("grid");
    assert_eq!(grid.budget_bytes(), 8192);
}
