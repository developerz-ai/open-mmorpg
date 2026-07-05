# 🎮 Game-Server Specs

> **Scope:** the authoritative runtime — everything between a client packet and durable state. The [architecture](../../architecture/README.md) folder says *how the system is shaped*; this folder says *how each server subsystem behaves*, tight enough to implement against. The renderer/client lives in [game-engine/](../game-engine/README.md) + [client/](../client/README.md); web lives elsewhere.

Every spec here distills three proven references — **AzerothCore/TrinityCore** (what a full WoW-scale server actually needs, and where the 2004 design breaks), **GTA Online** (seamless single-world population without loading), **retail WoW ≥ Dragonflight** (sharding/layering/CRZ done at scale) — plus canonical real-time netcode theory. We take the *concept*, implement it *server-authoritative in Rust*, original IP only ([legal](../../initial-idea/01-legal-and-licensing.md)).

## Index
| Spec | Subsystem | Distilled from |
|---|---|---|
| [tick-loop/](tick-loop/README.md) | Deterministic fixed-timestep sim loop, system schedule | TrinityCore `World::Update` → replace with ECS schedule |
| [netcode/](netcode/README.md) | Transport, reliability, snapshots+delta, prediction, AoI | Quake3/Source snapshot model, Gambetta, Overwatch |
| [world-model/](world-model/README.md) | Spatial partition, interest management, navmesh, streaming | ACore grid/cell + MMaps/VMaps → quadtree + Recast |
| [sharding/](sharding/README.md) | Zone shards, autoscale, handoff, merge/split | WoW sharding/layering + GTA session-merge |
| [persistence/](persistence/README.md) | 4-tier data, ownership txns, anti-dupe, migrations | ACore 3-DB split → one strongly-consistent store |
| [combat/](combat/README.md) | Ability/spell pipeline, threat, cooldowns, resolution | ACore spell/aura system → data-driven + deterministic |
| [economy/](economy/README.md) | Trade, mail, auction house, double-entry ledger | ACore dupe post-mortems → transactional + idempotent |
| [content-scripting/](content-scripting/README.md) | Data-driven content, WASM/Lua sandbox, manifest | DBC/DB2 + SmartScripts → typed schema + fuel-metered VM |
| [security/](security/README.md) | Server authority, movement validation, rate limits, DDoS | ACore anti-cheat gaps + GTA P2P failures |

## Non-negotiable principles (inherited, do not re-litigate → [CLAUDE.md](../../../CLAUDE.md))
1. **Server-authoritative, always.** Client sends [`Intent`](../../../crates/protocol/src/lib.rs), never state.
2. **`crates/sim` is deterministic.** Same ordered inputs → bit-identical state. Enables replay, lockstep validation, anti-cheat re-sim.
3. **Ownership writes → one YugabyteDB transaction.** Never through cache/bus. The dupe path must not compile ([persistence](persistence/README.md)).
4. **Content is data, core is compiled.** New faction/class/ability ships with no `cargo build` ([content-scripting](content-scripting/README.md)).
5. **Horizontal from day 1.** Autoscaled shards, no realm caps, no queues ([sharding](sharding/README.md)).

## What we deliberately do differently from AzerothCore
| ACore/TrinityCore (2004-era design) | Ours | Why |
|---|---|---|
| One `worldserver` process = one realm; scales up, not out | Autoscaled stateless shards per zone/instance | Realm caps & queues are the failure mode we refuse ([sharding](sharding/README.md)) |
| Single-threaded map update loop | ECS parallel system schedule per shard | A slow map must not stall the world ([tick-loop](tick-loop/README.md)) |
| Grid/cell visibility, hand-tuned | Quadtree AoI shared with streaming + shard boundaries | One spatial index for interest, streaming, and sharding ([world-model](world-model/README.md)) |
| 3 SQL DBs (auth/characters/world), MySQL, eventual dupe bugs | Yugabyte (durable, Raft) + Dragonfly (ephemeral); ownership only in txn | Anti-dupe by type, not by convention ([persistence](persistence/README.md)) |
| Static game data in client DBC/DB2 + SQL `world` DB | Typed `content-schema`, validated at boot, client mirrors manifest | Moddable core, fail-loud content ([content-scripting](content-scripting/README.md)) |
| SmartScripts/Eluna as bolt-on | WASM (primary) + Lua, sandboxed, fuel-metered, capability API | Untrusted operator content can't hang a shard or touch the DB |

## Reference index
**Internal:** [architecture/](../../architecture/README.md) · [initial-idea/](../../initial-idea/README.md) · crate sources under [`crates/`](../../../crates) · [gold standards](../../../CLAUDE.md#standards).
**External** (per-spec citations inline): AzerothCore & TrinityCore wikis/source · Gabriel Gambetta *Fast-Paced Multiplayer* · Gaffer On Games (Glenn Fiedler) · Valve Source multiplayer networking · Overwatch GDC netcode talk (Tim Ford) · Recast/Detour navmesh · WoW dev interviews on sharding/layering.

> Each subsystem doc is ≤ ~1 screen: what it does, the design, the distilled lesson, the rules, and links. If a doc grows past that, split it — same SRP rule as code.
