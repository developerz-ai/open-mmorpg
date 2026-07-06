//! Integration tests for the LOD system — tier selection by screen size, imposter
//! fallback, cull threshold, and octahedral atlas cell mapping.
//!
//! Exercises the public [`LodChain`] / [`ImposterAtlas`] / [`projected_screen_size`]
//! surface only — no `crate::` imports, exactly as a render client would use them.

use core::num::NonZeroU32;

use bevy_math::Vec3;
use omm_engine_assets::{
    lod::projected_screen_size, AssetError, ImposterAtlas, ImposterCell, LodChain, LodSelection,
};

// ── helpers ───────────────────────────────────────────────────────────────────

/// 3-tier chain: finest ≥ 0.5, mid ≥ 0.2, coarse ≥ 0.05; imposter [0.01, 0.05); cull 0.01.
#[allow(clippy::expect_used)]
fn chain() -> LodChain {
    LodChain::new(vec![0.5, 0.2, 0.05], true, 0.01).expect("valid chain")
}

#[allow(clippy::expect_used)]
fn atlas(grid: u32) -> ImposterAtlas {
    ImposterAtlas::new(NonZeroU32::new(grid).expect("non-zero"))
}

// ── tier selection by screen size ─────────────────────────────────────────────

#[test]
fn large_near_object_gets_finest_tier() {
    let lod = chain();
    // Screen size well above the finest threshold (0.5) → tier 0.
    assert_eq!(lod.select(0.9), LodSelection::Mesh(0));
    assert_eq!(lod.select(0.5), LodSelection::Mesh(0)); // exactly at threshold
}

#[test]
fn medium_distance_lands_in_mid_tier() {
    let lod = chain();
    assert_eq!(lod.select(0.3), LodSelection::Mesh(1));
    assert_eq!(lod.select(0.2), LodSelection::Mesh(1)); // exactly at threshold
}

#[test]
fn far_object_lands_in_coarse_tier() {
    let lod = chain();
    assert_eq!(lod.select(0.1), LodSelection::Mesh(2));
    assert_eq!(lod.select(0.05), LodSelection::Mesh(2)); // exactly at threshold
}

#[test]
fn very_far_object_becomes_imposter() {
    let lod = chain();
    // Below coarsest threshold (0.05), above cull (0.01) → imposter.
    assert_eq!(lod.select(0.04), LodSelection::Imposter);
    assert_eq!(lod.select(0.02), LodSelection::Imposter);
    assert_eq!(lod.select(0.01), LodSelection::Imposter); // cull is exclusive lower bound
}

#[test]
fn tiny_object_is_culled() {
    let lod = chain();
    assert_eq!(lod.select(0.009_999), LodSelection::Culled);
    assert_eq!(lod.select(0.0), LodSelection::Culled);
}

#[test]
fn non_finite_input_culls() {
    let lod = chain();
    assert_eq!(lod.select(f32::NAN), LodSelection::Culled);
    assert_eq!(lod.select(f32::NEG_INFINITY), LodSelection::Culled);
    // INFINITY means "at/behind camera" — always the finest tier.
    assert_eq!(lod.select(f32::INFINITY), LodSelection::Mesh(0));
}

// ── without imposter ─────────────────────────────────────────────────────────

#[test]
fn without_imposter_coarsest_tier_holds_to_cull() {
    let lod = LodChain::new(vec![0.5, 0.05], false, 0.01).expect("valid");
    assert!(!lod.has_imposter());
    // Below coarsest (0.05) but above cull (0.01) → coarsest mesh, not imposter.
    assert_eq!(lod.select(0.03), LodSelection::Mesh(1));
    assert_eq!(lod.select(0.01), LodSelection::Mesh(1));
    assert_eq!(lod.select(0.009), LodSelection::Culled);
}

// ── construction invariants ───────────────────────────────────────────────────

#[test]
fn empty_thresholds_rejected() {
    assert_eq!(
        LodChain::new(vec![], true, 0.0),
        Err(AssetError::EmptyLodChain)
    );
}

#[test]
fn non_descending_thresholds_rejected() {
    assert_eq!(
        LodChain::new(vec![0.3, 0.5], true, 0.0), // ascending = wrong
        Err(AssetError::LodNotDescending { index: 0 })
    );
    assert_eq!(
        LodChain::new(vec![0.5, 0.5], true, 0.0), // equal = wrong
        Err(AssetError::LodNotDescending { index: 0 })
    );
}

#[test]
fn cull_at_or_above_coarsest_threshold_rejected() {
    assert!(matches!(
        LodChain::new(vec![0.5, 0.1], true, 0.1),
        Err(AssetError::LodCull { .. })
    ));
    assert!(matches!(
        LodChain::new(vec![0.5, 0.1], true, -0.01),
        Err(AssetError::LodCull { .. })
    ));
}

#[test]
fn single_tier_chain_is_valid() {
    let lod = LodChain::new(vec![0.1], false, 0.0).expect("single tier");
    assert_eq!(lod.tiers(), 1);
    assert_eq!(lod.select(0.5), LodSelection::Mesh(0));
}

// ── projected screen size ─────────────────────────────────────────────────────

#[test]
fn nearer_object_projects_larger() {
    let near = projected_screen_size(1.0, 5.0, 0.5);
    let far = projected_screen_size(1.0, 50.0, 0.5);
    assert!(near > far, "near {near} vs far {far}");
}

#[test]
fn object_at_or_behind_camera_saturates_to_infinity() {
    assert_eq!(projected_screen_size(1.0, 0.0, 1.0), f32::INFINITY);
    assert_eq!(projected_screen_size(1.0, -1.0, 1.0), f32::INFINITY);
    assert_eq!(projected_screen_size(1.0, 5.0, 0.0), f32::INFINITY);
}

#[test]
fn screen_size_formula_matches_expected_value() {
    // radius=1, distance=2, tan_half_fov=1 → 1/(2*1) = 0.5
    let size = projected_screen_size(1.0, 2.0, 1.0);
    assert!((size - 0.5).abs() < 1e-6, "got {size}");
}

#[test]
fn screen_size_feeds_lod_chain_correctly() {
    let lod = chain();
    // Near: radius 1, distance 2, tan_half_fov 0.5 → size = 1/(2*0.5) = 1.0 → Mesh(0)
    let near = projected_screen_size(1.0, 2.0, 0.5);
    assert_eq!(lod.select(near), LodSelection::Mesh(0));
    // Far: radius 1, distance 60, tan_half_fov 0.5 → ≈0.033 → Imposter
    let far = projected_screen_size(1.0, 60.0, 0.5);
    assert_eq!(lod.select(far), LodSelection::Imposter);
}

// ── octahedral imposter atlas ─────────────────────────────────────────────────

#[test]
fn atlas_grid_and_cell_count() {
    let a = atlas(8);
    assert_eq!(a.grid(), 8);
    assert_eq!(a.cell_count(), 64);
}

#[test]
fn front_and_back_views_land_in_different_cells() {
    let a = atlas(8);
    let front = a.cell_for_view(Vec3::Z);
    let back = a.cell_for_view(Vec3::NEG_Z);
    assert_ne!(front, back, "+Z and -Z must map to distinct atlas cells");
}

#[test]
fn zero_direction_maps_to_centre_cell() {
    let a = atlas(8);
    assert_eq!(a.cell_for_view(Vec3::ZERO), ImposterCell { x: 4, y: 4 });
}

#[test]
fn cell_selection_is_deterministic_and_in_range() {
    let a = atlas(16);
    for dir in [
        Vec3::X,
        Vec3::Y,
        Vec3::Z,
        Vec3::NEG_X,
        Vec3::NEG_Y,
        Vec3::NEG_Z,
        Vec3::new(0.6, 0.2, 0.7),
        Vec3::new(-0.3, 0.8, -0.4),
    ] {
        let cell = a.cell_for_view(dir);
        assert!(cell.x < 16 && cell.y < 16, "cell out of range for {dir:?}");
        assert_eq!(cell, a.cell_for_view(dir), "non-deterministic for {dir:?}");
    }
}

#[test]
fn opposite_directions_select_different_cells() {
    let a = atlas(16);
    assert_ne!(
        a.cell_for_view(Vec3::new(0.6, 0.2, 0.7)),
        a.cell_for_view(Vec3::new(-0.6, -0.2, -0.7))
    );
}
