//! Integration tests: drive the whole physics pipeline in a headless engine app.
//!
//! Spawns static colliders + a character, ticks the fixed simulation, and
//! asserts the observable outcomes the spec names — "the controller moves" and
//! collision resolves — through `PhysicsPlugin` end to end (broadphase sync →
//! move-and-slide), not just the pure solver.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy_app::App;
use bevy_ecs::entity::Entity;
use bevy_math::Vec3;
use bevy_transform::components::Transform;
use omm_engine_core::headless_app;
use omm_engine_physics::{Aabb3d, CharacterController, Collider, MoveIntent, PhysicsPlugin};

/// A wide floor slab whose top face is at `y = 0`.
fn floor() -> Collider {
    Collider::Aabb(Aabb3d::new(
        Vec3::new(-50.0, -1.0, -50.0),
        Vec3::new(50.0, 0.0, 50.0),
    ))
}

/// Boot a headless engine app with physics installed.
fn physics_app() -> App {
    let mut app = headless_app();
    app.add_plugins(PhysicsPlugin);
    app
}

/// Spawn a character at `feet` with an optional horizontal move intent; return
/// its entity id.
fn spawn_character(app: &mut App, feet: Vec3, intent: Option<Vec3>) -> Entity {
    let mut e = app.world_mut().spawn((
        CharacterController::default(),
        Transform::from_translation(feet),
    ));
    if let Some(v) = intent {
        e.insert(MoveIntent::new(v));
    }
    e.id()
}

fn feet(app: &App, e: Entity) -> Vec3 {
    app.world()
        .entity(e)
        .get::<Transform>()
        .expect("transform")
        .translation
}

fn controller(app: &App, e: Entity) -> CharacterController {
    *app.world()
        .entity(e)
        .get::<CharacterController>()
        .expect("controller")
}

fn run(app: &mut App, ticks: u32) {
    for _ in 0..ticks {
        app.update();
    }
}

#[test]
fn character_falls_under_gravity_and_lands_on_the_floor() {
    let mut app = physics_app();
    app.world_mut().spawn((floor(), Transform::default()));
    let ch = spawn_character(&mut app, Vec3::new(0.0, 3.0, 0.0), None);

    run(&mut app, 90); // ~3 s of sim at 30 Hz — plenty to fall 3 m and settle

    let p = feet(&app, ch);
    assert!(
        controller(&app, ch).grounded,
        "should be grounded after landing"
    );
    assert!(
        p.y.abs() < 0.05,
        "should rest on the floor top (y≈0), got {}",
        p.y
    );
    assert!(
        controller(&app, ch).vertical_velocity.abs() < 1.0,
        "vertical velocity zeroed on ground, got {}",
        controller(&app, ch).vertical_velocity
    );
}

#[test]
fn character_walks_into_a_wall_and_stops_without_passing_through() {
    let mut app = physics_app();
    app.world_mut().spawn((floor(), Transform::default()));
    // Wall face at x = 2.
    let wall = Collider::Aabb(Aabb3d::new(
        Vec3::new(2.0, -1.0, -50.0),
        Vec3::new(3.0, 3.0, 50.0),
    ));
    app.world_mut().spawn((wall, Transform::default()));
    let ch = spawn_character(&mut app, Vec3::ZERO, Some(Vec3::new(5.0, 0.0, 0.0)));

    run(&mut app, 120);

    let p = feet(&app, ch);
    assert!(
        p.x > 1.0,
        "should have advanced toward the wall, got x {}",
        p.x
    );
    assert!(
        p.x < 1.7,
        "must stop before the wall (~x 1.6 = 2 - radius), got x {}",
        p.x
    );
    assert!(
        controller(&app, ch).grounded,
        "stays grounded while walking"
    );
}

#[test]
fn character_climbs_a_small_step() {
    let mut app = physics_app();
    app.world_mut().spawn((floor(), Transform::default()));
    // A 0.2 m ledge (below the 0.3 step height) starting at x = 1.
    let step = Collider::Aabb(Aabb3d::new(
        Vec3::new(1.0, 0.0, -50.0),
        Vec3::new(50.0, 0.2, 50.0),
    ));
    app.world_mut().spawn((step, Transform::default()));
    let ch = spawn_character(&mut app, Vec3::ZERO, Some(Vec3::new(3.0, 0.0, 0.0)));

    run(&mut app, 120);

    let p = feet(&app, ch);
    assert!(
        p.y > 0.15,
        "should have climbed onto the 0.2 ledge, got y {}",
        p.y
    );
    assert!(
        p.x > 2.0,
        "should have advanced onto the ledge, got x {}",
        p.x
    );
    assert!(controller(&app, ch).grounded);
}

#[test]
fn collider_removal_stops_blocking_the_character() {
    let mut app = physics_app();
    app.world_mut().spawn((floor(), Transform::default()));
    let wall = Collider::Aabb(Aabb3d::new(
        Vec3::new(2.0, -1.0, -50.0),
        Vec3::new(3.0, 3.0, 50.0),
    ));
    let wall_e = app.world_mut().spawn((wall, Transform::default())).id();
    let ch = spawn_character(&mut app, Vec3::ZERO, Some(Vec3::new(5.0, 0.0, 0.0)));

    run(&mut app, 60);
    assert!(feet(&app, ch).x < 1.7, "wall should block first");

    // Remove the wall; the broadphase must drop it and let the character through.
    app.world_mut().entity_mut(wall_e).despawn();
    run(&mut app, 120);
    assert!(
        feet(&app, ch).x > 3.5,
        "should pass the old wall x once removed, got {}",
        feet(&app, ch).x
    );
}

#[test]
fn simulation_is_deterministic_across_runs() {
    let scenario = |seed_x: f32| {
        let mut app = physics_app();
        app.world_mut().spawn((floor(), Transform::default()));
        let wall = Collider::Aabb(Aabb3d::new(
            Vec3::new(2.0, -1.0, -50.0),
            Vec3::new(3.0, 3.0, 50.0),
        ));
        app.world_mut().spawn((wall, Transform::default()));
        let ch = spawn_character(
            &mut app,
            Vec3::new(seed_x, 2.0, 0.0),
            Some(Vec3::new(4.0, 0.0, 1.0)),
        );
        run(&mut app, 100);
        feet(&app, ch)
    };
    assert_eq!(
        scenario(0.0),
        scenario(0.0),
        "identical inputs must yield identical state"
    );
}
