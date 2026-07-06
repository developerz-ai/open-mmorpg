//! End-to-end: an agent-authored prefab loads and spawns on the headless engine.
//!
//! This is the E2 "spawn-from-data" definition of done exercised through the real
//! plugin stack — `EnginePlugins` (headless core) + `EngineScenePlugin` — with no
//! GPU or window, proving content authored as text becomes live entities and that
//! invalid data fails loud without mutating the world.

use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use omm_engine_core::EnginePlugins;
use omm_engine_scene::{EngineScenePlugin, EntityBlueprint, Prefab, SceneError};

#[derive(Component, Reflect, Default, PartialEq, Debug)]
#[reflect(Component)]
struct Enemy {
    hp: u32,
    level: u32,
}

fn engine_app() -> App {
    let mut app = App::new();
    app.add_plugins(EnginePlugins);
    app.add_plugins(EngineScenePlugin);
    app.register_type::<Enemy>();
    app
}

#[test]
fn authored_prefab_loads_and_spawns_on_headless_engine() {
    let mut app = engine_app();
    let ron = r#"[
        [ {"prefab_spawn::Enemy": (hp: 42, level: 3)} ],
        [ {"prefab_spawn::Enemy": (hp: 7, level: 1)} ],
    ]"#;

    let prefab = {
        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        match Prefab::from_ron(ron, &registry) {
            Ok(prefab) => prefab,
            Err(err) => panic!("authoring a valid prefab failed: {err}"),
        }
    };
    assert_eq!(prefab.len(), 2);

    let spawned = match prefab.spawn(app.world_mut()) {
        Ok(entities) => entities,
        Err(err) => panic!("spawn failed: {err}"),
    };
    assert_eq!(spawned.len(), 2);

    let Some(&first) = spawned.first() else {
        panic!("expected two entities")
    };
    assert_eq!(
        app.world().get::<Enemy>(first),
        Some(&Enemy { hp: 42, level: 3 })
    );
    let Some(&second) = spawned.get(1) else {
        panic!("expected a second entity")
    };
    assert_eq!(
        app.world().get::<Enemy>(second),
        Some(&Enemy { hp: 7, level: 1 })
    );
}

#[test]
fn unregistered_component_fails_loud_and_spawns_nothing() {
    #[derive(Component, Reflect, Default)]
    #[reflect(Component)]
    struct Ghost {
        spooky: bool,
    }

    let mut app = engine_app(); // registers `Enemy`, but never `Ghost`
    let prefab = Prefab::new().with_entity(EntityBlueprint::new().with(Ghost { spooky: true }));

    match prefab.spawn(app.world_mut()) {
        Err(SceneError::UnregisteredType { type_path }) => {
            assert!(type_path.contains("Ghost"), "got {type_path}");
        }
        other => panic!("expected UnregisteredType, got {other:?}"),
    }

    let mut query = app.world_mut().query::<&Enemy>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "invalid prefab must spawn nothing"
    );
}
