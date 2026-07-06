//! Distance-tiered animation LOD tests — the near/mid/far ladder, the
//! local-player-never-VAT rule, threshold validation, and bone-budget math.
//! Pure logic, so these run under both `--no-default-features` (headless) and
//! `--all-features` alike.

use omm_engine_anim::{AnimTier, LodThresholds};

// Default matches `new(30.0, 100.0, 24)`; used by the non-`new` cases so the
// helper stays outside a `#[test]` fn without tripping the expect-in-tests lint.
fn thresholds() -> LodThresholds {
    LodThresholds::default()
}

#[test]
fn local_player_is_always_skeletal_regardless_of_distance() {
    let lod = thresholds();
    // Even absurdly far away, the entity the user controls keeps full fidelity.
    assert_eq!(lod.select(0.0, true), AnimTier::Skeletal);
    assert_eq!(lod.select(50.0, true), AnimTier::Skeletal);
    assert_eq!(lod.select(10_000.0, true), AnimTier::Skeletal);
    assert_eq!(lod.select(f32::INFINITY, true), AnimTier::Skeletal);
}

#[test]
fn near_entity_is_skeletal() {
    let lod = thresholds();
    assert_eq!(lod.select(0.0, false), AnimTier::Skeletal);
    assert_eq!(lod.select(29.999, false), AnimTier::Skeletal);
}

#[test]
fn mid_entity_is_reduced() {
    let lod = thresholds();
    // Boundary: distance == mid_distance is no longer "near".
    assert_eq!(lod.select(30.0, false), AnimTier::Reduced);
    assert_eq!(lod.select(99.999, false), AnimTier::Reduced);
}

#[test]
fn far_entity_is_vat() {
    let lod = thresholds();
    // Boundary: distance == far_distance drops to VAT.
    assert_eq!(lod.select(100.0, false), AnimTier::Vat);
    assert_eq!(lod.select(500.0, false), AnimTier::Vat);
}

#[test]
fn negative_distance_is_treated_as_near() {
    let lod = thresholds();
    assert_eq!(lod.select(-5.0, false), AnimTier::Skeletal);
}

#[test]
fn nan_distance_falls_to_cheapest_tier_for_non_local() {
    let lod = thresholds();
    // NaN comparisons are false, so a garbage distance lands on VAT (cheapest) —
    // never silently promotes a random NPC to full skeletal.
    assert_eq!(lod.select(f32::NAN, false), AnimTier::Vat);
    // ...but a NaN distance still can't demote the local player.
    assert_eq!(lod.select(f32::NAN, true), AnimTier::Skeletal);
}

#[test]
fn new_rejects_invalid_ladders() {
    // mid >= far is a mis-ordered ladder.
    assert!(LodThresholds::new(100.0, 30.0, 24).is_err());
    assert!(LodThresholds::new(50.0, 50.0, 24).is_err());
    // negative near threshold.
    assert!(LodThresholds::new(-1.0, 100.0, 24).is_err());
    // non-finite thresholds.
    assert!(LodThresholds::new(f32::NAN, 100.0, 24).is_err());
    assert!(LodThresholds::new(30.0, f32::INFINITY, 24).is_err());
    // a valid ladder.
    assert!(LodThresholds::new(30.0, 100.0, 24).is_ok());
}

#[test]
fn bone_budget_per_tier() {
    let lod = thresholds(); // reduced_bone_budget = 24
    assert_eq!(
        lod.bone_budget(AnimTier::Skeletal, 90),
        90,
        "skeletal keeps every bone"
    );
    assert_eq!(
        lod.bone_budget(AnimTier::Reduced, 90),
        24,
        "reduced caps at the budget"
    );
    assert_eq!(
        lod.bone_budget(AnimTier::Reduced, 10),
        10,
        "reduced never invents bones"
    );
    assert_eq!(
        lod.bone_budget(AnimTier::Vat, 90),
        0,
        "vat has no live skeleton"
    );
}

#[test]
fn tier_capability_flags() {
    assert!(AnimTier::Skeletal.runs_graph());
    assert!(AnimTier::Skeletal.runs_ik());
    assert!(AnimTier::Skeletal.needs_skinning());
    assert!(!AnimTier::Skeletal.is_vat());

    assert!(AnimTier::Reduced.runs_graph());
    assert!(
        !AnimTier::Reduced.runs_ik(),
        "reduced tier drops the IK pass"
    );
    assert!(AnimTier::Reduced.needs_skinning());
    assert!(!AnimTier::Reduced.is_vat());

    assert!(
        !AnimTier::Vat.runs_graph(),
        "vat plays baked clips, not the graph"
    );
    assert!(!AnimTier::Vat.runs_ik());
    assert!(
        !AnimTier::Vat.needs_skinning(),
        "vat animates in the shader"
    );
    assert!(AnimTier::Vat.is_vat());
}

#[test]
fn default_thresholds_are_sane() {
    let lod = LodThresholds::default();
    assert_eq!(lod.mid_distance, 30.0);
    assert_eq!(lod.far_distance, 100.0);
    assert_eq!(lod.reduced_bone_budget, 24);
    assert!(lod.mid_distance < lod.far_distance);
}
