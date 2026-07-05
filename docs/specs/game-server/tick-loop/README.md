# Tick Loop

> The heartbeat of a shard: ingest intents → step deterministic sim → emit snapshots. Fixed timestep, no wall-clock in the sim. → [architecture/03](../../../architecture/03-netcode-and-sharding.md)

## What it does
Each shard runs one authoritative loop at a **fixed tick rate** (target **30 Hz**, `TICK_DT = 1/30 s` — see [`ecs-core`](../../../../crates/ecs-core/src/lib.rs)). Per tick:

1. **Ingest** — drain queued [`ClientMsg::Input`](../../../../crates/protocol/src/lib.rs) since last tick; validate each [`Intent`](../../../../crates/protocol/src/lib.rs) against authoritative state (range, cooldown, cost, ownership → [security](../security/README.md)). Reject invalid, don't crash.
2. **Step** — run the ECS system schedule over the world. Movement/combat/abilities resolve through [`sim::step`](../../../../crates/sim/src/lib.rs). Pure, deterministic, no I/O.
3. **Snapshot** — produce per-client [`ServerMsg::Snapshot`](../../../../crates/protocol/src/lib.rs) stamped with the [`Tick`](../../../../crates/protocol/src/lib.rs), filtered by interest ([world-model](../world-model/README.md)), delta-compressed ([netcode](../netcode/README.md)).
4. **Sink side-effects** — ownership mutations already committed via [`persistence`](../persistence/README.md) transactions inside step; ephemeral broadcasts pushed to the bus. The tick loop itself never writes ownership.

## Design
- **Fixed timestep, accumulator-driven.** Real time advances an accumulator; while `acc >= TICK_DT`, run one sim step and subtract. Decouples sim rate from send rate and from any render/frame rate. → Gaffer On Games, *Fix Your Timestep*.
- **Send rate ≤ tick rate.** Sim at 30 Hz; snapshots may go out every tick or every N ticks per client based on interest priority — bandwidth is bounded by AoI, not tick rate ([netcode](../netcode/README.md)).
- **Parallel system schedule, not a single update walk.** Bevy ECS schedules disjoint systems concurrently; ordering is explicit where systems conflict. This is the deliberate replacement for TrinityCore's single-threaded `World::Update`/`Map::Update` walk.
- **No blocking on the tick path.** `tokio::sync` only, never `std::sync::Mutex`; DB/cache calls that a system needs are issued so a slow query can't stall the tick — a shard that misses its deadline degrades gracefully (longer effective dt is bounded/logged), it does not freeze the world.
- **Overrun policy.** If a tick exceeds `TICK_DT`, do **not** spiral: clamp accumulated catch-up to a max (e.g. 5 steps) and log; a shard that can't keep up is a scale-out signal ([sharding](../sharding/README.md)), not a correctness failure.

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| TrinityCore `World::Update` + `Map::Update` | One thread walks every grid/cell/object per diff; simple, deterministic, but the whole realm is bounded by one core. Map threads help but a hot map still serializes. | **Keep** the fixed-diff authoritative loop; **replace** the single-thread walk with a parallel ECS schedule and one-zone-per-shard so scaling is horizontal, not per-core. |
| Overwatch (GDC, T. Ford) | Fixed simulation tick + command buffering; sim rate independent of client frame rate. | **Adopt** fixed-timestep buffered inputs verbatim in principle. |
| Valve Source | Server tick (`tickrate`) distinct from client cmd/update rates. | **Adopt** the tick-vs-send-rate split. |

## Rules
- Sim must stay pure/deterministic: no `Instant::now()`, no RNG without a seeded, replicated stream, no floats from platform-variant math on the authoritative path.
- Every snapshot carries the tick that produced it → client interpolation + server re-sim line up.
- Cancellation-safe: a client disconnect drops its input queue and interest subscriptions at the next tick boundary, mid-tick state stays consistent.

## Links
[netcode](../netcode/README.md) · [world-model](../world-model/README.md) · [combat](../combat/README.md) · [security](../security/README.md) · deterministic sim: [`crates/sim`](../../../../crates/sim/src/lib.rs)
