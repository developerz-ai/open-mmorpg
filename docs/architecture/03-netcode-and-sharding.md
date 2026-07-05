# 03 — Netcode & Sharding

## Transport
| Traffic | Transport | Why |
|---|---|---|
| Realtime game state | **UDP** + custom reliability (`renet`/`quinn`/QUIC) | low latency, ordered-where-needed, unreliable-where-not |
| Web client | **WebTransport/WebSocket** fallback | browser reach |
| Auth/session/HTTP | **axum over TLS** (gateway) | standard request/response |

## Tick loop (per shard)
- Fixed **server tick** (e.g. 20–30 Hz). Server is authoritative.
- Per tick: ingest inputs → run `sim` systems (deterministic) → produce **snapshot**.
- Send **delta-compressed snapshots** filtered by **interest management** (area-of-interest / grid or quadtree) — a client only receives entities near it.
- Client does **prediction + reconciliation**; server corrects. Client never asserts state.

## Determinism
`crates/sim` is deterministic (same inputs → same state). Enables replay, lockstep validation, and cheap server-side re-simulation for anti-cheat ([08](08-security-anticheat.md)).

## Seamless zones & session merge-split
- World partitioned by **quadtree**; zones stream in/out with no loading screen ([06](06-world-and-assets.md)).
- **Handoff:** when a player crosses a shard boundary, the source shard serializes the player's transient state and hands off to the target shard via the bus; durable state already lives in Yugabyte, so handoff moves only transient sim state.
- **Merge/split (GTA-inspired concept):** low pop → merge instances into fewer shards; high pop → split a hot zone into more instances. One logical world, variable shard count. Design goal: **no player-visible loading** during merge/split. → [../initial-idea/09-gta6-inspiration.md](../initial-idea/09-gta6-inspiration.md)

## Rules
- Cancellation-safe: client disconnect → drop the task → clean up interest subscriptions.
- No `std::sync::Mutex` on the tick path — `tokio::sync` only; a slow shard must not block the bus.
- Wire types live in `crates/protocol` and are the single source of truth server↔client.
