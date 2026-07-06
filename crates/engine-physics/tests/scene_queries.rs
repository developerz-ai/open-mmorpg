//! Integration test: use the scene-query API the way the **server world-model**
//! would — build a collider set from world geometry (as if decoded from the
//! open glTF/heightmap VMaps), wrap it in a [`SceneQuery`], and answer
//! line-of-sight, targeting, and clearance probes. No ECS, no GPU: the same
//! deterministic code the client runs for prediction, exercised headless.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use bevy_math::Vec3;
use omm_engine_physics::{Aabb3d, Capsule, Collider, Ray, SceneQuery, Sphere};

/// A room: a wall on `x ≈ 5` split by a doorway (the gap `z ∈ (-1, 1)`), a round
/// pillar, and a target dummy — keyed as the world-model would key world objects.
fn room() -> Vec<(u64, Collider)> {
    vec![
        // Wall segment below the doorway (spans z in [-5, -1]).
        (
            10,
            Collider::Aabb(Aabb3d::new(
                Vec3::new(4.5, 0.0, -5.0),
                Vec3::new(5.5, 4.0, -1.0),
            )),
        ),
        // Wall segment above the doorway (spans z in [1, 5]).
        (
            11,
            Collider::Aabb(Aabb3d::new(
                Vec3::new(4.5, 0.0, 1.0),
                Vec3::new(5.5, 4.0, 5.0),
            )),
        ),
        // A round pillar off to one side.
        (
            20,
            Collider::Capsule(Capsule::new(
                Vec3::new(8.0, 0.0, 3.0),
                Vec3::new(8.0, 4.0, 3.0),
                0.5,
            )),
        ),
        // A target dummy on the far side of the wall.
        (
            30,
            Collider::Sphere(Sphere::new(Vec3::new(10.0, 1.0, 0.0), 0.5)),
        ),
    ]
}

#[test]
fn line_of_sight_is_blocked_by_wall_but_clear_through_the_doorway() {
    let colliders = room();
    let query = SceneQuery::new(&colliders);

    // Aiming across the wall (segment 11) at the dummy: the wall is in the way.
    assert!(!query.line_of_sight(Vec3::new(0.0, 1.0, 3.0), Vec3::new(10.0, 1.0, 0.0)));

    // Threading the doorway gap (z ≈ 0.5) to a point past the wall: unobstructed.
    assert!(query.line_of_sight(Vec3::new(0.0, 1.0, 0.5), Vec3::new(7.0, 1.0, 0.5)));
}

#[test]
fn targeting_raycast_returns_the_nearest_world_object() {
    let colliders = room();
    let query = SceneQuery::new(&colliders);
    // On the z = 3 line the wall (x ≈ 4.5) stands in front of the pillar (x = 8).
    let ray = Ray::new(Vec3::new(0.0, 1.0, 3.0), Vec3::X).expect("valid ray");
    let (key, hit) = query.raycast(&ray, 100.0).expect("something is hit");
    assert_eq!(key, 11, "the wall is the nearest object on that line");
    assert!((hit.toi - 4.5).abs() < 1e-3);
    assert!(hit.normal.dot(Vec3::X) < 0.0); // normal faces the shooter
}

#[test]
fn thick_probe_cannot_pass_a_doorway_narrower_than_itself() {
    let colliders = room();
    let query = SceneQuery::new(&colliders);
    // The doorway gap spans z in (-1, 1): 2.0 wide, so its half-width is 1.0.
    let from = Vec3::new(0.0, 1.0, 0.0);
    let to = Vec3::new(8.0, 1.0, 0.0);
    // A thin ray and a slim probe thread the gap...
    assert!(query.line_of_sight(from, to));
    assert!(query.thick_line_of_sight(from, to, 0.5));
    // ...but a 1.5-radius probe (3.0 wide) cannot fit the 2.0-wide doorway.
    assert!(!query.thick_line_of_sight(from, to, 1.5));
}

#[test]
fn queries_are_replay_stable_regardless_of_collider_order() {
    let mut colliders = room();
    let ray = Ray::new(Vec3::new(0.0, 1.0, 3.0), Vec3::X).expect("valid ray");
    let first = SceneQuery::new(&colliders).raycast(&ray, 100.0);
    colliders.reverse();
    let reversed = SceneQuery::new(&colliders).raycast(&ray, 100.0);
    assert_eq!(first, reversed);
}
