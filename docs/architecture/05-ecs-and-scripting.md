# 05 — ECS & Scripting (the extensibility contract)

> This is what makes it the **best open core**: operators add factions/classes/quests without forking. The line between compiled and data is the product.

## The line
| Compiled core (`crates/`) | Data/scripts (`content/`) |
|---|---|
| ECS runtime, tick loop, netcode | which components a race/class has |
| combat *resolution engine* | combat *formulas & ability definitions* |
| DB/transaction layer | item/quest/faction/zone definitions |
| WASM/Lua host + sandbox | the WASM/Lua behavior scripts themselves |
| interest management, sharding | encounter/spawn tables, world-event scripts |

If changing content requires `cargo build`, the line is in the wrong place — fix the line, not the content.

## ECS (Bevy ECS)
- Entities = players, NPCs, items, projectiles, companions. Components = data. Systems = behavior.
- **New race/class = new component data + a system** (or a script) — no engine recompile.
- Shared `ecs-core` component definitions are used **both** by `apps/shard` (authoritative) and `apps/client` (prediction/render) — one mental model.

## Scripting
| Layer | Use | Sandbox |
|---|---|---|
| **WASM** (primary) | untrusted operator content: abilities, quests, faction mechanics, world events | full sandbox, fuel-metered, no host access beyond a capability API |
| **Lua** (`mlua`) | lightweight first-party/content scripts, fast iteration | sandboxed, restricted stdlib |

- Scripts get a **narrow capability API** (spawn entity, query nearby, apply effect, grant item *via `persistence`*). They **cannot** touch the DB or cache directly — economy stays behind the transactional contract ([04](04-data-and-consistency.md)).
- **Fuel-metered** so a bad script can't hang a shard.

## Content schema & mod manifest
- `crates/content-schema` defines typed schemas; `content/` files validate against them at load. Invalid content fails loud at boot, not at runtime.
- **Mod manifest** (`content/manifest.json`) is Minecraft-datapack-style, **version-locked to a stable core API version**. Core bumps that break the API bump the version; old mods refuse to load rather than corrupt state.
- **Client mirrors server**: client pulls the active manifest + asset bundles on connect. New faction ships with no client patch.
