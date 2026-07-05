# 30 — Game Server · Wave 2

> **Big idea.** The **authoritative runtime** — everything between a client packet and durable state. Deterministic tick loop, snapshot/delta netcode with prediction, spatial world model, **autoscaled stateless shards** (no realm caps, no queues), data-driven combat & economy, sandboxed scripting, server authority everywhere. A **game-agnostic framework** — our MMORPG is one game on it. **Owner: [`server`](../../../.claude/agents/server.md).** → [game-server specs](../../specs/game-server/README.md)

## Reads
[game-server README](../../specs/game-server/README.md) and each subsystem: [tick-loop](../../specs/game-server/tick-loop/README.md) · [netcode](../../specs/game-server/netcode/README.md) · [world-model](../../specs/game-server/world-model/README.md) · [sharding](../../specs/game-server/sharding/README.md) · [persistence](../../specs/game-server/persistence/README.md) · [combat](../../specs/game-server/combat/README.md) · [economy](../../specs/game-server/economy/README.md) · [content-scripting](../../specs/game-server/content-scripting/README.md) · [security](../../specs/game-server/security/README.md) · [02 topology](../../architecture/02-server-topology.md) · [03 netcode-and-sharding](../../architecture/03-netcode-and-sharding.md) · [08 security](../../architecture/08-security-anticheat.md).

## Depends on
[Foundation](00-foundation.md) `protocol`/`errors`/`ecs-core`/`content-schema`; [Database](10-database.md) `persistence`/`cache` (its ownership txn + ephemeral APIs); [Engine](20-game-engine.md) shared deterministic sim logic (`ecs-core`, physics LOS). Builds `crates/sim` + `crates/netcode` + `crates/scripting` and the `apps/gateway` `apps/shard` `apps/worldsvc` binaries.

## PR batches (~100 PRs)
| Batch | Scope | Spec | Definition of Done |
|---|---|---|---|
| **S1 · `sim` deterministic** | Fixed-timestep ECS schedule, movement/combat state, **same ordered inputs → bit-identical state**. No wall-clock/rng on sim path. | [tick-loop](../../specs/game-server/tick-loop/README.md) | Replay test: input log → identical state hash twice. Shared with [client](40-game-client.md). |
| **S2 · `netcode` transport** | UDP reliability, framing, connection lifecycle, congestion. | [netcode](../../specs/game-server/netcode/README.md) | Lossy-link test delivers reliably; handshake with [client](40-game-client.md). |
| **S3 · Snapshots + delta + AoI** | Server snapshots, delta compression, interest-filtered by area. | [netcode](../../specs/game-server/netcode/README.md) · [world-model](../../specs/game-server/world-model/README.md) | Client receives only AoI-relevant deltas; bandwidth bounded. |
| **S4 · World model** | Quadtree spatial index (one index for interest + streaming + shard boundaries), navmesh (Recast), streaming. | [world-model](../../specs/game-server/world-model/README.md) | Entities queried by area; navmesh pathing in the slice zone. |
| **S5 · `apps/gateway`** | axum edge: auth, session tokens, routing to shards, DDoS/rate-limit edge, HTTP control plane. | [security](../../specs/game-server/security/README.md) · [02](../../architecture/02-server-topology.md) | Client authenticates, gets a session, is routed to a shard. Web reads realm status here. |
| **S6 · `apps/shard`** | Headless zone server: tokio + `ecs-core` + `netcode` + `sim`, one tick loop per zone. | [tick-loop](../../specs/game-server/tick-loop/README.md) | A player connects, moves, is simulated authoritatively; state persists via [persistence](10-database.md). |
| **S7 · Combat pipeline** | Data-driven ability/spell resolution: effect dispatch, auras, cooldowns, threat — **engine, not content**. | [combat](../../specs/game-server/combat/README.md) | An `AbilityDef` from [content](60-content-and-assets.md) resolves deterministically; threat table works. |
| **S8 · Economy plumbing** | Transactional trade/mail/AH built on the [ledger](10-database.md); idempotent, anti-dupe. | [economy](../../specs/game-server/economy/README.md) | AH list/buy moves value with zero dupes under contention. |
| **S9 · Scripting sandbox** | `crates/scripting`: WASM (primary) + Lua, fuel-metered, capability API — **untrusted content can't hang a shard or touch the DB**. | [content-scripting](../../specs/game-server/content-scripting/README.md) | A content script runs sandboxed; an infinite loop is fuel-killed, not a shard stall. |
| **S10 · Sharding & handoff** | Autoscale stateless shards, player handoff across zone boundary, merge/split. | [sharding](../../specs/game-server/sharding/README.md) | Two shards autoscale; a player crosses a boundary seamlessly. |
| **S11 · `apps/worldsvc`** | Cross-shard services: chat, guild, auction house, world feed — the read/social plane the web consumes. | [02](../../architecture/02-server-topology.md) | Web armory/AH/feed read from worldsvc projections. |
| **S12 · Security & authority** | Movement validation, rate limits, input validation, re-sim anti-cheat. **Client never trusted.** | [security](../../specs/game-server/security/README.md) · [08](../../architecture/08-security-anticheat.md) | Forged movement/intents rejected; re-sim catches divergence. |

## Interfaces this track owns
- **`crates/sim`** — the deterministic simulation, **shared verbatim with [client](40-game-client.md)** for prediction.
- **Gateway/worldsvc HTTP contracts** — the server↔[web](50-web-client.md) contract (realm status, armory, AH, feed).
- **The replicated component set + `Intent` handling** — built on [`protocol`](../../../crates/protocol).
- **Content-facing hooks** — the `AbilityDef`/script capability surface [content](60-content-and-assets.md) targets.

## Rules (hard)
- **Server-authoritative, always** — client sends `Intent`, never state; validate everything.
- **`sim` is deterministic** — replay, lockstep validation, anti-cheat re-sim all depend on it.
- **Ownership → one Yugabyte txn** via [`persistence`](10-database.md), never `cache`/bus.
- **Content is data** — new faction/class/ability ships with no `cargo build` of the core.
- **Horizontal from day 1** — stateless shards, autoscale, no realm caps, no queues.
- **Files ≤300 LOC, one crate = one reason to change, no `unwrap` on the tick path.**

## Definition of Done (track)
A player connects through the gateway to a shard, is simulated authoritatively, casts a data-driven ability, trades on the AH with no dupes, and hands off across a shard boundary; forged intents are rejected; the web reads realm/armory/AH from worldsvc; the whole thing is exercised headless in CI.

## Links
[game-server specs](../../specs/game-server/README.md) · [database](10-database.md) · [engine](20-game-engine.md) · [client](40-game-client.md) · [web](50-web-client.md) · [`server` subagent](../../../.claude/agents/server.md)
