//! Headless shard integration test: two loopback clients connect, both send
//! `Move`, and each receives only the AoI-relevant delta.
//!
//! Drives the full pipeline — session registry, World tick, snapshot egress —
//! against in-memory loopback transports. No sockets, no background tasks.
//! Three scenarios:
//! 1. Both clients nearby → each sees the other in its delta.
//! 2. Clients far apart → each sees only itself (AoI exclusion).
//! 3. Same inputs run twice → identical tick hash (anti-cheat oracle).

use std::collections::{BTreeMap, BTreeSet};

use omm_ecs_core::Team;
use omm_netcode::{DeltaFrame, Loopback, Transport};
use omm_protocol::{ClientMsg, Intent, ServerMsg, Tick, Vec3};
use omm_shard::replicate::{snapshot_msg, world_entities, ClientReplication, DEFAULT_BUDGET_BYTES};
use omm_shard::session::SessionRegistry;
use omm_sim::combat::Actor;
use omm_sim::World;
use omm_transport::ConnId;

// ── helpers ───────────────────────────────────────────────────────────────────

fn actor_at(x: f32, z: f32) -> Actor {
    Actor::new(Vec3 { x, y: 0.0, z }, Team(1), 100, 100)
}

fn move_dir(x: f32, z: f32) -> Intent {
    Intent::Move {
        dir: Vec3 { x, y: 0.0, z },
    }
}

fn ids_in_delta(delta: &DeltaFrame) -> Vec<u64> {
    delta.changed.iter().map(|e| e.id).collect()
}

// ── test: nearby clients both see each other ──────────────────────────────────

/// Both clients adjacent → first delta contains both entities for each viewer.
/// The snapshot round-trips through the loopback transport and is decoded on the
/// "client" side.
#[tokio::test]
async fn nearby_clients_see_each_other_in_first_delta() {
    // Loopback pairs: (client endpoint, server endpoint).
    let (cli_a, srv_a) = Loopback::pair(ConnId::new(1));
    let (cli_b, srv_b) = Loopback::pair(ConnId::new(2));

    // Bind both connections → spawn actors adjacent to each other.
    let mut world = World::new();
    let mut registry = SessionRegistry::new();
    let ent_a = registry.bind(ConnId::new(1), &mut world, actor_at(0.0, 0.0));
    let ent_b = registry.bind(ConnId::new(2), &mut world, actor_at(5.0, 0.0));

    // Clients send Move intents.
    let input_a = ClientMsg::Input {
        tick: Tick(0),
        intent: move_dir(1.0, 0.0),
    };
    let input_b = ClientMsg::Input {
        tick: Tick(0),
        intent: move_dir(-1.0, 0.0),
    };
    cli_a
        .send(&serde_json::to_vec(&input_a).unwrap())
        .await
        .unwrap();
    cli_b
        .send(&serde_json::to_vec(&input_b).unwrap())
        .await
        .unwrap();

    // Server drains each connection's inbox.
    let raw_a = srv_a.try_recv().await.unwrap().unwrap();
    let raw_b = srv_b.try_recv().await.unwrap().unwrap();
    let decoded_a: ClientMsg = serde_json::from_slice(&raw_a).unwrap();
    let decoded_b: ClientMsg = serde_json::from_slice(&raw_b).unwrap();

    // Build the server-side input batch (EntityId-sorted, server-resolved ids).
    let mut inputs = Vec::new();
    if let ClientMsg::Input { intent, .. } = decoded_a {
        inputs.push((registry.entity_of(ConnId::new(1)).unwrap(), intent));
    }
    if let ClientMsg::Input { intent, .. } = decoded_b {
        inputs.push((registry.entity_of(ConnId::new(2)).unwrap(), intent));
    }
    inputs.sort_by_key(|(id, _)| *id);

    // Tick once.
    world.step(&inputs, &BTreeMap::new());
    assert_eq!(world.now(), Tick(1));

    // AoI: both clients are nearby → full mutual interest.
    let all = world_entities(&world);
    let near_both: BTreeSet<u64> = [ent_a.raw(), ent_b.raw()].into_iter().collect();

    let mut cur_a = ClientReplication::new(ent_a, DEFAULT_BUDGET_BYTES);
    let mut cur_b = ClientReplication::new(ent_b, DEFAULT_BUDGET_BYTES);
    let delta_a = cur_a.delta_for(Tick(1), 0, &all, &near_both);
    let delta_b = cur_b.delta_for(Tick(1), 0, &all, &near_both);

    // First delta from an empty baseline → every visible entity is "changed".
    assert_eq!(delta_a.changed.len(), 2, "A sees both entities");
    assert_eq!(delta_b.changed.len(), 2, "B sees both entities");
    assert!(delta_a.removed.is_empty());
    assert!(delta_b.removed.is_empty());

    let ids_a = ids_in_delta(&delta_a);
    let ids_b = ids_in_delta(&delta_b);
    assert!(ids_a.contains(&ent_a.raw()), "A's self is in its delta");
    assert!(ids_a.contains(&ent_b.raw()), "A sees B");
    assert!(ids_b.contains(&ent_b.raw()), "B's self is in its delta");
    assert!(ids_b.contains(&ent_a.raw()), "B sees A");

    // Encode to wire and round-trip through the loopback transport.
    let snap_a = snapshot_msg(&delta_a).unwrap();
    let snap_b = snapshot_msg(&delta_b).unwrap();
    srv_a
        .send(&serde_json::to_vec(&snap_a).unwrap())
        .await
        .unwrap();
    srv_b
        .send(&serde_json::to_vec(&snap_b).unwrap())
        .await
        .unwrap();

    let rx_a: ServerMsg = serde_json::from_slice(&cli_a.recv().await.unwrap()).unwrap();
    let rx_b: ServerMsg = serde_json::from_slice(&cli_b.recv().await.unwrap()).unwrap();

    match rx_a {
        ServerMsg::Snapshot { tick, delta } => {
            assert_eq!(tick, Tick(1));
            let frame: DeltaFrame = serde_json::from_slice(&delta).unwrap();
            assert_eq!(frame.changed.len(), 2);
        }
        other => panic!("expected Snapshot for A, got {other:?}"),
    }
    match rx_b {
        ServerMsg::Snapshot { tick, delta } => {
            assert_eq!(tick, Tick(1));
            let frame: DeltaFrame = serde_json::from_slice(&delta).unwrap();
            assert_eq!(frame.changed.len(), 2);
        }
        other => panic!("expected Snapshot for B, got {other:?}"),
    }
}

// ── test: AoI exclusion ───────────────────────────────────────────────────────

/// Clients far apart → each interest set holds only self; the delta must not
/// contain the other entity.
#[tokio::test]
async fn far_clients_receive_only_self_in_aoi_delta() {
    let mut world = World::new();
    let mut registry = SessionRegistry::new();
    // 500 units apart — well outside any reasonable AoI radius.
    let ent_a = registry.bind(ConnId::new(1), &mut world, actor_at(0.0, 0.0));
    let ent_b = registry.bind(ConnId::new(2), &mut world, actor_at(500.0, 0.0));

    let inputs = vec![(ent_a, move_dir(1.0, 0.0)), (ent_b, move_dir(-1.0, 0.0))];
    world.step(&inputs, &BTreeMap::new());

    let all = world_entities(&world);

    // Each client's interest set holds only itself.
    let only_a: BTreeSet<u64> = [ent_a.raw()].into_iter().collect();
    let only_b: BTreeSet<u64> = [ent_b.raw()].into_iter().collect();

    let mut cur_a = ClientReplication::new(ent_a, DEFAULT_BUDGET_BYTES);
    let mut cur_b = ClientReplication::new(ent_b, DEFAULT_BUDGET_BYTES);
    let delta_a = cur_a.delta_for(Tick(1), 0, &all, &only_a);
    let delta_b = cur_b.delta_for(Tick(1), 0, &all, &only_b);

    assert_eq!(delta_a.changed.len(), 1, "A sees only itself");
    assert_eq!(delta_a.changed[0].id, ent_a.raw());
    assert_eq!(delta_b.changed.len(), 1, "B sees only itself");
    assert_eq!(delta_b.changed[0].id, ent_b.raw());
}

// ── test: deterministic tick hash ────────────────────────────────────────────

/// Running the exact same inputs on two independent worlds produces bit-identical
/// tick hashes — the oracle anti-cheat re-simulation compares each tick.
#[test]
fn tick_hash_is_stable_across_runs() {
    let run = || {
        let mut world = World::new();
        let mut registry = SessionRegistry::new();
        let ent_a = registry.bind(ConnId::new(1), &mut world, actor_at(0.0, 0.0));
        let ent_b = registry.bind(
            ConnId::new(2),
            &mut world,
            Actor::new(
                Vec3 {
                    x: 5.0,
                    y: 0.0,
                    z: 0.0,
                },
                Team(2),
                80,
                80,
            ),
        );

        // Three ticks with changing directions.
        let sequences: &[(&Intent, &Intent)] = &[
            (&move_dir(1.0, 0.0), &move_dir(0.0, -1.0)),
            (&move_dir(0.5, 0.5), &move_dir(-1.0, 0.0)),
            (&move_dir(0.0, 1.0), &move_dir(0.5, 0.5)),
        ];
        let mut hashes = Vec::new();
        for (ia, ib) in sequences {
            let inputs = vec![(ent_a, (*ia).clone()), (ent_b, (*ib).clone())];
            world.step(&inputs, &BTreeMap::new());
            hashes.push(world.state_hash());
        }
        hashes
    };

    let first = run();
    let second = run();
    assert_eq!(first, second, "same inputs must produce identical hashes");

    // Each tick's hash must differ from the previous (the world actually moves).
    for w in first.windows(2) {
        assert_ne!(w[0], w[1], "consecutive ticks with motion must diverge");
    }
}
