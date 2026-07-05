# Content & Scripting

> The extensibility contract: operators add factions/classes/abilities/quests with **no `cargo build`**. The line between compiled core and data content is the product. → [architecture/05](../../../architecture/05-ecs-and-scripting.md)

## What it does
Loads, validates, and runs game **content as data** — [`content/`](../../../../content) definitions validated against [`content-schema`](../../../../crates/content-schema/src/lib.rs), plus behavior scripts run in a sandboxed VM ([`scripting`](../../../../crates/scripting/src/lib.rs)). Invalid content **fails loud at boot**, not at runtime. The client mirrors the active [`manifest.json`](../../../../content/manifest.json) on connect — a new faction ships with no client patch.

## The tiered model (distilled from TrinityCore)
TrinityCore proved a 3-tier split: declarative data for the common case, compiled escape hatch, community scripting. We keep the shape, fix the ergonomics:

| Tier | TrinityCore | Ours | For |
|---|---|---|---|
| Declarative data | SmartScripts (31-col SQL `Event→Action→Target`) | **typed, schema-validated content** ([content-schema](../../../../crates/content-schema/src/lib.rs)) | 90% of NPC/quest/ability behavior |
| Compiled core | `ScriptMgr` C++ hooks | **ECS systems** in [`crates/`](../../../../crates) | bespoke, hot-path logic |
| Community scripting | Eluna/Lua bolted to C++ | **WASM (primary) + Lua**, sandboxed | untrusted operator mods |

- **Keep** the `Event → Action → Target` vocabulary — a good, readable template for data-driven behavior.
- **Replace** the opaque fixed 31-column SQL schema with a typed format that's authorable and validated.
- **Replace** Lua-glued-to-C++ with a **deterministic, sandboxed** scripting layer so content stays replayable for anti-cheat re-sim ([combat](../combat/README.md)).

## Static game data (distilled from DBC/DB2)
TrinityCore loads immutable rules from client **DBC/DB2** blobs + a MySQL `world` DB; `hotfixes` overrides them live. We replace all of it with versioned [`content/`](../../../../content) data + [`content-schema`](../../../../crates/content-schema/src/lib.rs), and hotfixes become **hot-reloadable content**. Static rules (definitions) and dynamic state (ownership) stay cleanly separated — the latter is Yugabyte-only ([persistence](../persistence/README.md)).

## Scripting sandbox
| Layer | Use | Sandbox |
|---|---|---|
| **WASM** (primary) | untrusted operator content: abilities, quests, faction mechanics, world events | full sandbox, **fuel-metered**, no host access beyond a capability API |
| **Lua** (`mlua`) | first-party/content scripts, fast iteration | sandboxed, restricted stdlib |

- Scripts get a **narrow capability API**: spawn entity, query nearby, apply effect, grant item **via `persistence`**. They **cannot** touch the DB, cache, filesystem, or network directly — the economy stays behind the transactional contract ([persistence](../persistence/README.md), [security](../security/README.md)).
- **Fuel-metered** so a bad script is starved, not able to hang a shard.
- **Deterministic** — no wall-clock/uncontrolled RNG, so scripted outcomes replay ([tick-loop](../tick-loop/README.md)).

## Manifest & versioning
- [`content/manifest.json`](../../../../content/manifest.json) is Minecraft-datapack-style, **version-locked to a stable core API version** (`api_version`). A core change that breaks the API bumps the version; old mods **refuse to load** rather than corrupt state.
- Client pulls the active manifest + asset bundles on connect ([world-model](../world-model/README.md)).

## Rules
- If changing a faction/class/ability needs a recompile, the compiled/data line is in the wrong place — fix the line, not the content.
- Content validates at boot; invalid content is a loud boot failure, never a runtime surprise.
- Scripts never get a privileged path to state — same server-authoritative route as any client ([security](../security/README.md)).

## Links
[combat](../combat/README.md) · [persistence](../persistence/README.md) · [security](../security/README.md) · [`content/`](../../../../content) · [`crates/scripting`](../../../../crates/scripting/src/lib.rs) · [`crates/content-schema`](../../../../crates/content-schema/src/lib.rs)
