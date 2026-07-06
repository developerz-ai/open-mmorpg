//! Round-trip + determinism + fail-loud integration tests for the scene/prefab
//! pipeline.
//!
//! Proves (end-to-end, full plugin stack):
//! 1. The agent-authored fixture loads and spawns with the expected component
//!    values — the data-authoring contract.
//! 2. Loading the same RON twice produces identical component values in declared
//!    order (determinism guarantee required for replay / anti-cheat).
//! 3. `from_ron → to_ron → from_ron` round-trip preserves component values.
//! 4. An unregistered type in prefab data fails loudly (`SceneError::Parse`)
//!    before the world is touched.
//!
//! Types below are referenced by the fixture with their integration-test type
//! paths (`scene_roundtrip::Hp`, `scene_roundtrip::Tier`).  The test binary's
//! file-stem is the crate-root module name for integration-test binaries.

use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use omm_engine_core::EnginePlugins;
use omm_engine_scene::{EngineScenePlugin, Prefab, SceneError};

// ---------------------------------------------------------------------------
// Component types referenced from the fixture
// ---------------------------------------------------------------------------

/// Hitpoints component — fixture uses `scene_roundtrip::Hp`.
#[derive(Component, Reflect, Default, PartialEq, Debug, Clone)]
#[reflect(Component)]
struct Hp {
    current: u32,
    max: u32,
}

/// Tier component — fixture uses `scene_roundtrip::Tier`.
#[derive(Component, Reflect, Default, PartialEq, Debug, Clone)]
#[reflect(Component)]
struct Tier {
    value: u32,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn registered_app() -> App {
    let mut app = App::new();
    app.add_plugins(EnginePlugins);
    app.add_plugins(EngineScenePlugin);
    app.register_type::<Hp>();
    app.register_type::<Tier>();
    app
}

// The fixture is embedded at compile time so tests are self-contained.
const FIXTURE: &str = include_str!("fixtures/prefab.scn.ron");

// ---------------------------------------------------------------------------
// Test 1 — fixture loads and expected component values are present
// ---------------------------------------------------------------------------

#[test]
fn fixture_loads_and_validates() {
    let mut app = registered_app();

    let prefab = {
        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        Prefab::from_ron(FIXTURE, &registry).expect("fixture must be valid RON")
    };

    assert_eq!(prefab.len(), 2, "fixture must define two entities");

    let spawned = prefab.spawn(app.world_mut()).expect("fixture must spawn cleanly");
    assert_eq!(spawned.len(), 2);

    let world = app.world();
    assert_eq!(world.get::<Hp>(spawned[0]), Some(&Hp { current: 50, max: 100 }));
    assert_eq!(world.get::<Tier>(spawned[0]), Some(&Tier { value: 3 }));
    assert_eq!(world.get::<Hp>(spawned[1]), Some(&Hp { current: 10, max: 10 }));
    // Entity 1 has no Tier — absence is part of the contract.
    assert_eq!(world.get::<Tier>(spawned[1]), None);
}

// ---------------------------------------------------------------------------
// Test 2 — two loads produce identical component values (determinism)
// ---------------------------------------------------------------------------

/// All component values extracted from a spawn, keyed by entity index.
fn extract(world: &World, entities: &[Entity]) -> Vec<(Option<Hp>, Option<Tier>)> {
    entities
        .iter()
        .map(|&e| (world.get::<Hp>(e).cloned(), world.get::<Tier>(e).cloned()))
        .collect()
}

#[test]
fn two_loads_produce_identical_component_values() {
    let load = || {
        let mut app = registered_app();
        let prefab = {
            let registry = app.world().resource::<AppTypeRegistry>().clone();
            let registry = registry.read();
            Prefab::from_ron(FIXTURE, &registry).expect("fixture must parse")
        };
        let spawned = prefab.spawn(app.world_mut()).expect("spawn must succeed");
        let values = extract(app.world(), &spawned);
        values
    };

    assert_eq!(
        load(),
        load(),
        "same RON must yield identical component values across two independent loads"
    );
}

// ---------------------------------------------------------------------------
// Test 3 — from_ron → to_ron → from_ron round-trip preserves component values
// ---------------------------------------------------------------------------

#[test]
fn serialize_deserialize_roundtrip() {
    let app1 = registered_app();
    let mut app2 = registered_app();

    // First parse + serialize back to RON.
    let ron_out = {
        let registry = app1.world().resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let prefab = Prefab::from_ron(FIXTURE, &registry).expect("first parse");
        prefab.to_ron(&registry).expect("to_ron must succeed")
    };

    // Second parse from the re-serialized form.
    let prefab2 = {
        let registry = app2.world().resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        Prefab::from_ron(&ron_out, &registry).expect("second parse of re-serialized RON")
    };

    assert_eq!(
        prefab2.len(),
        2,
        "re-serialized prefab must still have two entities"
    );

    // Spawn into app2 and verify the component values match the fixture's
    // expected values — same data must survive the round-trip.
    let spawned2 = prefab2.spawn(app2.world_mut()).expect("spawn must succeed");
    let world = app2.world();

    assert_eq!(world.get::<Hp>(spawned2[0]), Some(&Hp { current: 50, max: 100 }));
    assert_eq!(world.get::<Tier>(spawned2[0]), Some(&Tier { value: 3 }));
    assert_eq!(world.get::<Hp>(spawned2[1]), Some(&Hp { current: 10, max: 10 }));
    assert_eq!(world.get::<Tier>(spawned2[1]), None);
}

// ---------------------------------------------------------------------------
// Test 4 — unregistered type errors loud; world is untouched
// ---------------------------------------------------------------------------

#[test]
fn unregistered_type_in_ron_fails_loud() {
    // `registered_app` registers Hp + Tier but not Ghost.
    let app = registered_app();
    let registry = app.world().resource::<AppTypeRegistry>().clone();
    let registry = registry.read();

    let ron = r#"[[{"scene_roundtrip::Ghost": (scary: true)}]]"#;
    match Prefab::from_ron(ron, &registry) {
        Err(SceneError::Parse(_)) => {} // correct — load refused before touching the world
        other => panic!("expected SceneError::Parse for an unregistered type, got {other:?}"),
    }
}
