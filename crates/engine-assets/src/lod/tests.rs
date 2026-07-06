use super::*;

fn chain() -> LodChain {
    // Finest tier ≥ 0.5 of the half-height, mid ≥ 0.2, coarsest ≥ 0.05; imposter
    // fills [0.01, 0.05); below 0.01 cull.
    LodChain::new(vec![0.5, 0.2, 0.05], true, 0.01).expect("valid chain")
}

#[test]
fn picks_finer_tiers_as_the_object_grows() {
    let lod = chain();
    assert_eq!(lod.select(0.8), LodSelection::Mesh(0)); // above finest threshold
    assert_eq!(lod.select(0.5), LodSelection::Mesh(0)); // exactly finest threshold
    assert_eq!(lod.select(0.3), LodSelection::Mesh(1)); // between finest and mid
    assert_eq!(lod.select(0.1), LodSelection::Mesh(2)); // between mid and coarsest
    assert_eq!(lod.select(0.03), LodSelection::Imposter); // below coarsest, above cull
    assert_eq!(lod.select(0.005), LodSelection::Culled); // below cull
}

#[test]
fn imposter_boundaries_are_half_open() {
    let lod = chain();
    // At the coarsest threshold it is still a mesh; just below, the imposter.
    assert_eq!(lod.select(0.05), LodSelection::Mesh(2));
    assert_eq!(lod.select(0.0499), LodSelection::Imposter);
    // At exactly the cull size it culls (cull is the inclusive lower bound of Culled).
    assert_eq!(lod.select(0.01), LodSelection::Imposter);
    assert_eq!(lod.select(0.009_999), LodSelection::Culled);
}

#[test]
fn without_an_imposter_the_coarsest_tier_holds_to_cull() {
    let lod = LodChain::new(vec![0.5, 0.05], false, 0.01).expect("valid");
    assert!(!lod.has_imposter());
    assert_eq!(lod.select(0.02), LodSelection::Mesh(1)); // no imposter → hold coarsest
    assert_eq!(lod.select(0.005), LodSelection::Culled);
}

#[test]
fn a_single_tier_chain_is_valid() {
    let lod = LodChain::new(vec![0.1], false, 0.0).expect("single tier");
    assert_eq!(lod.tiers(), 1);
    assert_eq!(lod.select(0.5), LodSelection::Mesh(0));
    assert_eq!(lod.select(0.05), LodSelection::Mesh(0)); // holds to cull (0.0)
}

#[test]
fn non_finite_screen_size_culls() {
    let lod = chain();
    assert_eq!(lod.select(f32::NAN), LodSelection::Culled);
    assert_eq!(lod.select(f32::NEG_INFINITY), LodSelection::Culled);
    // A huge/near object resolves to the finest tier.
    assert_eq!(lod.select(f32::INFINITY), LodSelection::Mesh(0));
}

#[test]
fn empty_thresholds_are_rejected() {
    assert_eq!(
        LodChain::new(vec![], true, 0.0),
        Err(AssetError::EmptyLodChain)
    );
}

#[test]
fn non_descending_thresholds_are_rejected() {
    // Equal is not strictly descending.
    assert_eq!(
        LodChain::new(vec![0.5, 0.5], true, 0.0),
        Err(AssetError::LodNotDescending { index: 0 })
    );
    // Ascending.
    assert_eq!(
        LodChain::new(vec![0.5, 0.2, 0.3], true, 0.0),
        Err(AssetError::LodNotDescending { index: 1 })
    );
    // Non-finite value.
    assert!(matches!(
        LodChain::new(vec![f32::NAN], true, 0.0),
        Err(AssetError::LodNotDescending { .. })
    ));
}

#[test]
fn cull_out_of_range_is_rejected() {
    // cull ≥ coarsest threshold.
    assert!(matches!(
        LodChain::new(vec![0.5, 0.1], true, 0.1),
        Err(AssetError::LodCull { .. })
    ));
    // Negative cull.
    assert!(matches!(
        LodChain::new(vec![0.5], true, -0.1),
        Err(AssetError::LodCull { .. })
    ));
}

#[test]
fn projected_screen_size_shrinks_with_distance() {
    let near = projected_screen_size(1.0, 2.0, 1.0);
    let far = projected_screen_size(1.0, 20.0, 1.0);
    assert!(
        near > far,
        "nearer object must project larger: {near} vs {far}"
    );
    // radius / (distance * tan_half_fov) = 1 / (2 * 1) = 0.5
    assert!((near - 0.5).abs() < 1e-6, "got {near}");
}

#[test]
fn projected_screen_size_saturates_at_or_behind_camera() {
    assert_eq!(projected_screen_size(1.0, 0.0, 1.0), f32::INFINITY);
    assert_eq!(projected_screen_size(1.0, -5.0, 1.0), f32::INFINITY);
    assert_eq!(projected_screen_size(1.0, 5.0, 0.0), f32::INFINITY);
}

#[test]
fn projected_size_feeds_the_chain_for_a_camera_distance() {
    // A radius-1 object seen with tan_half_fov 0.5: near → finest, far → imposter.
    let lod = chain();
    let near = projected_screen_size(1.0, 2.0, 0.5); // 1.0 → Mesh(0)
    let far = projected_screen_size(1.0, 60.0, 0.5); // ~0.033 → Imposter
    assert_eq!(lod.select(near), LodSelection::Mesh(0));
    assert_eq!(lod.select(far), LodSelection::Imposter);
}

// ---- octahedral imposter atlas ----

fn atlas(grid: u32) -> ImposterAtlas {
    ImposterAtlas::new(NonZeroU32::new(grid).expect("nonzero grid"))
}

#[test]
fn atlas_reports_grid_and_cell_count() {
    let atlas = atlas(8);
    assert_eq!(atlas.grid(), 8);
    assert_eq!(atlas.cell_count(), 64);
}

#[test]
fn front_and_back_views_map_to_opposite_hemispheres() {
    let atlas = atlas(8);
    // +Z (looking at the front) lands at the octahedron centre.
    let front = atlas.cell_for_view(Vec3::Z);
    assert_eq!(front, ImposterCell { x: 4, y: 4 });
    // -Z (behind) folds to a corner — distinct from the front cell.
    let back = atlas.cell_for_view(Vec3::NEG_Z);
    assert_ne!(back, front);
}

#[test]
fn a_zero_direction_maps_to_the_centre_cell() {
    let atlas = atlas(8);
    assert_eq!(atlas.cell_for_view(Vec3::ZERO), ImposterCell { x: 4, y: 4 });
}

#[test]
fn cell_selection_is_deterministic_and_in_range() {
    let atlas = atlas(16);
    let dirs = [
        Vec3::X,
        Vec3::Y,
        Vec3::Z,
        Vec3::NEG_X,
        Vec3::NEG_Y,
        Vec3::NEG_Z,
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-0.3, 0.7, -0.5),
    ];
    for dir in dirs {
        let cell = atlas.cell_for_view(dir);
        assert!(
            cell.x < 16 && cell.y < 16,
            "cell out of range for {dir:?}: {cell:?}"
        );
        // Same direction → same cell, every time.
        assert_eq!(cell, atlas.cell_for_view(dir));
    }
}

#[test]
fn opposite_directions_select_different_cells() {
    let atlas = atlas(16);
    assert_ne!(
        atlas.cell_for_view(Vec3::new(0.6, 0.2, 0.7)),
        atlas.cell_for_view(Vec3::new(-0.6, -0.2, -0.7))
    );
}
