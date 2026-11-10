# 05 — ECS & Scripting (the extensibility contract)

> This is what makes it the **best open core**: operators add factions/classes/quests without forking. The line between compiled and data is the product.

## The line
This is the target extensibility boundary; the status notes below distinguish implemented primitives from planned integrations.

| Compiled core (`crates/`) | Data/scripts (`content/`) |
|---|---|
| ECS runtime, tick loop, netcode | which components a race/class has |
| combat *resolution engine* | combat *formulas & ability definitions* |
| DB/transaction layer | item/quest/faction/zone definitions |
| Wasmtime host + fuel/capability sandbox (implemented) | WASM module bytes supplied to the host; datapack script loading is not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)) |
| Lua/`mlua` host + sandbox — not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)) | planned Lua behavior scripts |
| interest management, sharding | encounter/spawn tables, world-event scripts |

If changing content requires `cargo build`, the line is in the wrong place — fix the line, not the content.

## ECS (Bevy ECS)
- Entities = players, NPCs, items, projectiles, companions. Components = data. Systems = behavior.
- **New race/class = new content data using existing systems** — no engine recompile. New compiled systems still require a build; script integration remains a target, not a shipped datapack-loading feature.
- Shared `ecs-core` components are **plain Rust structs** with pure systems, used by the authoritative simulation and client prediction; this crate does not re-export Bevy ECS.
- `engine-core` already uses Bevy app/ECS/reflection/time subcrates for its headless engine runtime. `apps/client` is currently a headless prediction demo; the headful Bevy/wgpu client is not yet implemented (planned: [C5/C8](../mvp/v1/40-game-client.md#pr-batches-100-prs)).

## Scripting
| Layer | Use | Sandbox |
|---|---|---|
| **WASM** (primary, implemented via Wasmtime) | compiles module bytes and runs an exported entry function; gameplay/datapack integration remains planned | fuel-metered, no host access beyond the capability API |
| **Lua** (`mlua`) — not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)) | target: lightweight first-party/content scripts, fast iteration | target: sandboxed, restricted stdlib |

- WASM scripts get a **narrow capability API** (spawn entity, query nearby, apply effect, request an item grant). `grant_item` expresses **intent**, not an ownership write; the reference `RecordingHost` records it. Routing grants through `persistence` is not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)). Scripts **cannot** touch the DB or cache directly — the host must preserve the transactional contract ([04](04-data-and-consistency.md)).
- WASM execution is **fuel-metered**; an infinite loop exhausts its budget instead of hanging the caller.

## Content schema & mod manifest
- `crates/content-schema` defines typed schemas; `content/` files validate against them at load. Invalid content fails loud at boot, not at runtime.
- `scripting::load_datapack` currently validates the manifest and returns its faction count; it does **not** resolve, compile, or load scripts. That integration is not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)), unlike the working Wasmtime module-byte API.
- **Mod manifest** (`content/manifest.json`) is Minecraft-datapack-style, **version-locked to a stable core API version**. Core bumps that break the API bump the version; old mods refuse to load rather than corrupt state.
- **Client mirrors server (target)**: fetching the active manifest + asset bundles on connect is not yet implemented (planned: [C2](../mvp/v1/40-game-client.md#pr-batches-100-prs)/[E3](../mvp/v1/20-game-engine.md#pr-batches-100-prs)). The goal is to ship a new faction with no client patch.
