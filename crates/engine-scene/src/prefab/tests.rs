use super::{EntityBlueprint, Prefab};
use crate::error::SceneError;
use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Reflect, Default, PartialEq, Debug, Clone)]
#[reflect(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component, Reflect, Default, PartialEq, Debug, Clone)]
#[reflect(Component)]
struct TeamId(u32);

/// Registered for reflection but deliberately *not* a `Component` (no
/// `#[reflect(Component)]`) — the trigger for a rolled-back partial spawn.
#[derive(Reflect, Default)]
struct DecorativeData {
    note: u32,
}

fn app_with_components() -> App {
    let mut app = App::new();
    app.register_type::<Health>();
    app.register_type::<TeamId>();
    app
}

fn count<C: Component>(world: &mut World) -> usize {
    let mut query = world.query::<&C>();
    query.iter(world).count()
}

#[test]
fn builder_tracks_sizes() {
    let blueprint = EntityBlueprint::new()
        .with(TeamId(1))
        .with(Health::default());
    assert_eq!(blueprint.len(), 2);
    assert!(!blueprint.is_empty());
    assert!(EntityBlueprint::new().is_empty());

    let prefab = Prefab::new().with_entity(blueprint);
    assert_eq!(prefab.len(), 1);
    assert!(!prefab.is_empty());
    assert_eq!(prefab.entities().len(), 1);
    assert!(Prefab::new().is_empty());
}

#[test]
fn spawns_components_in_declared_order_deterministically() {
    let build = || {
        let mut app = app_with_components();
        let prefab = Prefab::new()
            .with_entity(
                EntityBlueprint::new()
                    .with(Health {
                        current: 30.0,
                        max: 100.0,
                    })
                    .with(TeamId(7)),
            )
            .with_entity(EntityBlueprint::new().with_boxed(Box::new(TeamId(2))));
        let spawned = match prefab.spawn(app.world_mut()) {
            Ok(entities) => entities,
            Err(err) => panic!("spawn failed: {err}"),
        };
        (app, spawned)
    };

    let (app_a, a) = build();
    let (_app_b, b) = build();
    assert_eq!(a.len(), 2);
    assert_eq!(
        a, b,
        "same prefab + same world state must yield identical entity ids"
    );

    let world = app_a.world();
    let Some(&first) = a.first() else {
        panic!("missing entity 0")
    };
    assert_eq!(
        world.get::<Health>(first),
        Some(&Health {
            current: 30.0,
            max: 100.0
        })
    );
    assert_eq!(world.get::<TeamId>(first), Some(&TeamId(7)));

    let Some(&second) = a.get(1) else {
        panic!("missing entity 1")
    };
    assert_eq!(world.get::<TeamId>(second), Some(&TeamId(2)));
    assert_eq!(world.get::<Health>(second), None);
}

#[test]
fn unregistered_component_is_rejected_before_spawning() {
    // Note: `Health` is never registered here.
    let mut app = App::new();
    let prefab = Prefab::new().with_entity(EntityBlueprint::new().with(Health {
        current: 1.0,
        max: 1.0,
    }));

    match prefab.spawn(app.world_mut()) {
        Err(SceneError::UnregisteredType { type_path }) => {
            assert!(type_path.contains("Health"), "got {type_path}");
        }
        other => panic!("expected UnregisteredType, got {other:?}"),
    }
    // Atomic: nothing was spawned.
    assert_eq!(count::<Health>(app.world_mut()), 0);
}

#[test]
fn registered_non_component_rolls_back_partial_spawn() {
    let mut app = app_with_components();
    app.register_type::<DecorativeData>();

    let prefab = Prefab::new()
        // Entity 0 commits cleanly...
        .with_entity(EntityBlueprint::new().with(Health {
            current: 5.0,
            max: 5.0,
        }))
        // ...entity 1 references a non-Component, aborting the spawn.
        .with_entity(EntityBlueprint::new().with(DecorativeData { note: 1 }));

    match prefab.spawn(app.world_mut()) {
        Err(SceneError::PartialSpawn { spawned, type_path }) => {
            assert_eq!(spawned, 1, "one entity was committed before the abort");
            assert!(type_path.contains("DecorativeData"), "got {type_path}");
        }
        other => panic!("expected PartialSpawn, got {other:?}"),
    }
    // Rolled back: the already-committed entity 0 was despawned too.
    assert_eq!(count::<Health>(app.world_mut()), 0);
}

#[test]
fn empty_prefab_spawns_no_entities() {
    let mut app = app_with_components();
    match Prefab::new().spawn(app.world_mut()) {
        Ok(spawned) => assert!(spawned.is_empty()),
        Err(err) => panic!("empty prefab should spawn cleanly: {err}"),
    }
}
