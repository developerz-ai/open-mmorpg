use super::*;

/// Every tile costs the same — isolates the budget/eviction policy from cost noise.
fn uniform(bytes: u64) -> impl Fn(TileCoord) -> u64 {
    move |_| bytes
}

fn coord(x: i32, y: i32) -> TileCoord {
    TileCoord { x, y }
}

fn loaded_sorted(grid: &StreamingGrid) -> Vec<TileCoord> {
    grid.loaded_tiles().collect()
}

// ---- construction ----

#[test]
fn rejects_non_positive_config() {
    assert!(matches!(
        StreamingGrid::new(0.0, 10.0, 1024),
        Err(AssetError::InvalidStreamingConfig { field: "tile_size" })
    ));
    assert!(matches!(
        StreamingGrid::new(10.0, -1.0, 1024),
        Err(AssetError::InvalidStreamingConfig {
            field: "view_radius"
        })
    ));
    assert!(matches!(
        StreamingGrid::new(10.0, 10.0, 0),
        Err(AssetError::InvalidStreamingConfig { field: "budget" })
    ));
    assert!(matches!(
        StreamingGrid::new(f32::NAN, 10.0, 1024),
        Err(AssetError::InvalidStreamingConfig { .. })
    ));
}

#[test]
fn tile_at_floors_into_cells() {
    let grid = StreamingGrid::new(10.0, 5.0, 1024).expect("grid");
    assert_eq!(grid.tile_at(Vec2::new(5.0, 5.0)), coord(0, 0));
    assert_eq!(grid.tile_at(Vec2::new(-1.0, -1.0)), coord(-1, -1));
    assert_eq!(grid.tile_at(Vec2::new(23.0, -4.0)), coord(2, -1));
}

// ---- basic load / unload ----

#[test]
fn loads_only_the_containing_tile_within_a_tight_radius() {
    // radius 3 around a tile centre reaches no neighbour (nearest is 5 away).
    let mut grid = StreamingGrid::new(10.0, 3.0, 1_000_000).expect("grid");
    let delta = grid
        .update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("update");
    assert_eq!(loaded_sorted(&grid), vec![coord(0, 0)]);
    assert_eq!(delta.loaded, vec![coord(0, 0)]);
    assert!(delta.unloaded.is_empty() && delta.skipped.is_empty());
    assert_eq!(grid.resident_bytes(), 100);
}

#[test]
fn moving_the_camera_unloads_the_tiles_left_behind() {
    let mut grid = StreamingGrid::new(10.0, 3.0, 1_000_000).expect("grid");
    grid.update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("frame 1");
    assert!(grid.is_loaded(coord(0, 0)));

    // Jump far away — the old tile is out of view and must unload.
    let delta = grid
        .update(Vec2::new(105.0, 5.0), uniform(100))
        .expect("frame 2");
    assert!(!grid.is_loaded(coord(0, 0)));
    assert!(grid.is_loaded(coord(10, 0)));
    assert_eq!(delta.unloaded, vec![coord(0, 0)]);
    assert_eq!(delta.loaded, vec![coord(10, 0)]);
}

// ---- memory budget ----

#[test]
fn budget_caps_residency_and_reports_skipped_tiles() {
    // 5 tiles wanted (the plus-shape), budget for only 4.
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("grid");
    let delta = grid
        .update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("update");

    assert_eq!(grid.resident_bytes(), 400, "must not exceed the budget");
    // The camera's tile plus the three lowest-coord equidistant neighbours win.
    assert_eq!(
        loaded_sorted(&grid),
        vec![coord(-1, 0), coord(0, -1), coord(0, 0), coord(0, 1)]
    );
    // The tied-but-later neighbour (1,0) is wanted yet skipped, and reported.
    assert!(
        delta.skipped.contains(&coord(1, 0)),
        "skipped: {:?}",
        delta.skipped
    );
    assert!(
        grid.is_loaded(coord(0, 0)),
        "the containing tile is always resident"
    );
}

#[test]
fn a_nearer_tile_evicts_a_farther_resident_one_under_budget() {
    // Budget for 4 tiles, wide view so a full neighbourhood is wanted.
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("grid");
    grid.update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("frame 1");
    assert_eq!(
        loaded_sorted(&grid),
        vec![coord(-1, 0), coord(0, -1), coord(0, 0), coord(0, 1)]
    );

    // Move one tile down: (0,-1) now contains the camera. Tiles (-1,0)/(0,1) are
    // still in view but far, so nearer tiles evict them under the same budget.
    let delta = grid
        .update(Vec2::new(5.0, -5.0), uniform(100))
        .expect("frame 2");
    assert_eq!(grid.resident_bytes(), 400);
    assert_eq!(
        loaded_sorted(&grid),
        vec![coord(-1, -1), coord(0, -2), coord(0, -1), coord(0, 0)]
    );
    // The two far tiles were evicted for nearer ones — near-priority under budget.
    assert!(delta.unloaded.contains(&coord(-1, 0)));
    assert!(delta.unloaded.contains(&coord(0, 1)));
    assert!(delta.loaded.contains(&coord(-1, -1)));
    assert!(delta.loaded.contains(&coord(0, -2)));
}

#[test]
fn a_tile_larger_than_the_whole_budget_fails_loud() {
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
fn zero_view_radius_still_streams_the_camera_tile() {
    let mut grid = StreamingGrid::new(10.0, 0.0, 1024).expect("grid");
    grid.update(Vec2::new(5.0, 5.0), uniform(100))
        .expect("update");
    assert_eq!(loaded_sorted(&grid), vec![coord(0, 0)]);
}

// ---- determinism ----

#[test]
fn identical_inputs_produce_identical_residency_and_deltas() {
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
        assert_eq!(da, db, "same inputs must yield the same delta");
    }
    assert_eq!(loaded_sorted(&a), loaded_sorted(&b));
}

#[test]
fn variable_tile_costs_are_respected() {
    // A pricey central tile leaves room for fewer neighbours than a uniform grid.
    let mut grid = StreamingGrid::new(10.0, 25.0, 400).expect("grid");
    let cost = |c: TileCoord| if c == coord(0, 0) { 300 } else { 100 };
    grid.update(Vec2::new(5.0, 5.0), cost).expect("update");
    assert!(grid.is_loaded(coord(0, 0)));
    assert_eq!(grid.resident_bytes(), 400); // 300 + one 100 neighbour
    assert_eq!(grid.loaded_tiles().count(), 2);
}
