# 60 — Content & Assets · Wave 1→3

> **Big idea.** The **data-driven game layer** + the **AI asset pipeline**. Factions, races, classes, abilities, quests, and the slice zone/dungeon are **files, not code** — change one and the game changes with **no `cargo build`**. Assets (meshes, textures, audio) are **AI-generated through Meshy.ai → open formats** (glTF / KTX2 / Ogg-Opus / zstd), authored and verified by agents. **Owner: [`content`](../../../.claude/agents/content.md).** → [gameplay specs](../../specs/gameplay/README.md) · [content/](../../../content/README.md) · [06 world-and-assets](../../architecture/06-world-and-assets.md)

## Reads
[gameplay README](../../specs/gameplay/README.md) + [00 design-pillars](../../specs/gameplay/00-design-pillars.md) · [01 world-and-cosmology](../../specs/gameplay/01-world-and-cosmology.md) · [factions](../../specs/gameplay/factions/README.md) · [classes](../../specs/gameplay/classes/README.md) · [itemization](../../specs/gameplay/itemization.md) · [content-scripting](../../specs/game-server/content-scripting/README.md) · [content/](../../../content/README.md) · [05 asset-and-map-formats](../../initial-idea/05-asset-and-map-formats.md) · [06 world-and-assets](../../architecture/06-world-and-assets.md) · [engine assets](../../specs/game-engine/assets/README.md).

## Depends on
[Foundation](00-foundation.md) `content-schema` (types + `CONTENT_API_VERSION` + `validate()`); [Server](30-game-server.md) `AbilityDef`/script capability hooks + loader; [Engine](20-game-engine.md) asset/scene formats + hot reload. Data (C-batches) can start in Wave 1 against the schema; assets land as the engine renderer comes online.

## The canon — build to it (don't reinvent names)
The world is **Auralon**, the world-soul the **Emberheart**; factions the **Aurelian Concord** and the **Wildreach Pact**; six forces Lattice/Rift/Bloom/Hollow/Dawn/Deep ([01 world-and-cosmology](../../specs/gameplay/01-world-and-cosmology.md)). **Original IP only — never extract any existing game's asset, name, or table** ([legal](../../initial-idea/01-legal-and-licensing.md)).

## PR batches (~100 PRs)
| Batch | Scope | Definition of Done |
|---|---|---|
| **K1 · Schema-validated seed** | Fill [`content/manifest.json`](../../../content/manifest.json) + minimal typed defs so `content-schema` validates a bootable datapack. | Boot-time `validate()` passes; a datapack loads. |
| **K2 · Two factions + races** | [Concord](../../specs/gameplay/factions/concord.md) + [Pact](../../specs/gameplay/factions/pact.md) with a starter race each, as data. | Factions/races load; a character can be created in each. |
| **K3 · Slice class set** | A handful of classes (roles/resources), abilities as `AbilityDef` data the [combat engine](../../specs/game-server/combat/README.md) resolves. | Abilities resolve deterministically server-side; no core recompile. |
| **K4 · Starter zone (data)** | One zone: spawn tables, NPCs, nodes, navmesh source, streaming cells — data + scene. | Zone loads, is walkable, populated; renders on the [engine](20-game-engine.md). |
| **K5 · One dungeon + quests** | A small instanced dungeon + a starter quest chain via typed quest defs + sandboxed scripts. | Quests track/complete; dungeon instances; scripts run [fuel-metered](../../specs/game-server/content-scripting/README.md). |
| **K6 · Itemization slice** | A starter item set honoring the [30-level relevance window](../../specs/gameplay/itemization.md); loot tables; AH-tradeable. | Items drop, equip, sell on AH with no dupes ([economy](30-game-server.md)). |
| **K7 · AI asset pipeline (Meshy.ai)** | Repeatable pipeline: prompt → **Meshy.ai** mesh → glTF → KTX2 textures / Opus audio → zstd bundle → asset-server import. Scripted, versioned, agent-runnable. | An agent regenerates a slice asset from a prompt to a loadable glTF, no manual tooling. |
| **K8 · Asset integration** | Wire generated meshes/anims/textures/audio to the zone/classes; LOD/imposters; hot-reload verified. | Slice looks and sounds like a game; edit prompt → hot-reload result. |
| **K9 · Modding walkthrough** | `docs/guide/` datapack authoring guide + schema docs; a total-conversion example stub. | A stranger authors a loadable datapack from the docs. |

## The AI asset pipeline (Meshy.ai)
The content agent's superpower: **describe an asset, get an open-format asset**. Pipeline stages, each scripted and reproducible under `assets/` + a pipeline tool:
1. **Prompt** — structured brief tied to the [canon](#the-canon--build-to-it-dont-reinvent-names) (faction look, force palette).
2. **Generate** — Meshy.ai produces the mesh (+ PBR maps); other AI gens textures/audio as needed.
3. **Normalize to open formats** — glTF 2.0 mesh/skin, KTX2 textures, Ogg-Opus audio, PNG16/EXR heightmaps. **No proprietary tool in the pipeline.**
4. **Bundle** — zstd, content-addressed, described in the [manifest](../../../content/README.md).
5. **Import & verify** — engine asset server loads it; an agent asserts it renders/plays headless where possible. Hot-reload closes the loop.

## Interfaces this track owns / consumes
- **Owns** everything under [`content/`](../../../content/) + [`assets/`](../../../assets) + the asset-pipeline tool.
- **Consumes** `content-schema` (validates its data), the [combat](30-game-server.md)/[scripting](30-game-server.md) hooks (what abilities/scripts may do), the [engine](20-game-engine.md) asset/scene formats. **Adds zero lines to the compiled core.**

## Rules (hard)
- **Content is data, core is compiled** — if changing a faction/class/ability needs a `cargo build`, escalate to the owning code track; the split is wrong.
- **Open formats only** — glTF / KTX2 / Opus / PNG16-EXR / zstd. No proprietary tooling.
- **Original IP only** — never extract/rip any game's asset, name, or table.
- **Fail loud** — content that doesn't match the schema fails `validate()` at boot, not silently at runtime.
- **Agent-verifiable** — every asset regenerates from a prompt; every data change is testable headless.

## Definition of Done (track)
Two factions, a starter race + class set, a walkable populated zone, a dungeon, a quest chain, and a starter item set — all data — load and play on the finished engine/server; every mesh/texture/sound is AI-generated in open formats through the reproducible Meshy.ai pipeline; a modder can author a datapack from the docs; the core never recompiled to add any of it.

## Links
[gameplay specs](../../specs/gameplay/README.md) · [content/](../../../content/README.md) · [content-scripting](../../specs/game-server/content-scripting/README.md) · [06 world-and-assets](../../architecture/06-world-and-assets.md) · [`content` subagent](../../../.claude/agents/content.md)
