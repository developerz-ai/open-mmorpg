# 70 — Integration & Release · Wave 4

> **Big idea.** Where the seven tracks **meet and become a game**. This track builds nothing new in isolation — it **wires the vertical slice end-to-end, proves it, documents it, and tags v1.0.0**. The bar isn't "features exist"; it's "an agent plays the whole slice and asserts state, two shards autoscale and hand off, the web shows the character, and `bin/check` is green everywhere." **Owner: shared** (integration is everyone's). → [master plan](README.md) · [08 feature-bar](../../initial-idea/08-feature-bar.md)

## Depends on
Every track's Definition of Done: [DB](10-database.md), [Engine](20-game-engine.md), [Server](30-game-server.md), [Client](40-game-client.md), [Web](50-web-client.md), [Content](60-content-and-assets.md).

## PR batches (~100 PRs)
| Batch | Scope | Definition of Done |
|---|---|---|
| **R1 · Vertical slice wiring** | Boot the whole stack (`bin/dev`): Yugabyte + Dragonfly + gateway + 2 shards + worldsvc + client + web + slice content. | One command brings up a playable realm locally. |
| **R2 · Headless E2E** | The [agent-drive harness](40-game-client.md#c9--agent-drive-harness) plays the slice: create char → move → fight → loot → AH → relog, asserting authoritative state at each step. | Deterministic E2E test green in CI, no GPU. |
| **R3 · Scale & handoff proof** | Two shards autoscale under load; a player crosses a zone boundary seamlessly; no dupes under concurrent economy load. | Load test passes; handoff + anti-dupe asserted. |
| **R4 · Web ↔ game proof** | The E2E character appears on the [armory](50-web-client.md); its AH listing is browsable; slice events hit the world feed. | Cross-plane test: game state → worldsvc → web, verified. |
| **R5 · Security pass** | [Security review](../../specs/game-server/security/README.md): forged intents/movement rejected, rate limits hold, no secrets leaked, sandbox contains a hostile script. | Adversarial tests pass; `/security-review` clean on the diff. |
| **R6 · Docs & reference** | Finalize the three doc homes ([workflow §docs](01-workflow-and-parallelization.md#documentation)): dev/rustdoc, `docs/operations/`, `docs/guide/`. Truthful specs. | A stranger can run a realm, play the slice, and author a datapack from docs alone. |
| **R7 · Release gate + tag** | Green `bin/check` across all crates+apps; all batches' DoD met; version bump; changelog; **tag `v1.0.0`**. | The gate below passes; v1.0.0 tagged and its artifacts built. |

## The v1.0.0 release gate (all must be green)
- [ ] `bin/check` green across every crate and app (fmt + clippy `-D warnings` + nextest + biome/tsc/bun test).
- [ ] Headless agent plays the full slice E2E and asserts authoritative state (R2).
- [ ] `sim` is bit-deterministic — replay produces identical state hashes ([S1](30-game-server.md)).
- [ ] Ownership survives relog + shard handoff with **zero dupes** under concurrent load ([D7](10-database.md)/[R3](70-integration-and-release.md)).
- [ ] Two shards autoscale and hand a player across a boundary (R3).
- [ ] Web armory/AH/feed reflect real game state (R4).
- [ ] Client runs native + WebGPU baseline; the same core runs headless (C8/C1).
- [ ] Slice content (2 factions, class set, zone, dungeon, quests, items, AI assets) loads with **no core recompile** ([60](60-content-and-assets.md)).
- [ ] Security pass clean (R5).
- [ ] Docs complete for developer, operator, and player/modder (R6).

## Post-v1 backlog (explicitly NOT in v1.0.0)
Logged here so scope stays honest — these are **content/features on the finished foundation**, not v1.0.0 work:
the [full 1→300 arc & 25 Ages](../../specs/gameplay/progression/ages.md); [era servers](../../specs/gameplay/progression/era-servers.md); full [talent/hero trees](../../specs/gameplay/classes/talents-and-hero-trees.md); [raids](../../specs/gameplay/endgame/raids.md)/[delves](../../specs/gameplay/endgame/delves.md)/[full PvP](../../specs/gameplay/endgame/pvp.md); [housing](../../specs/gameplay/world-systems/housing.md); deep [professions](../../specs/gameplay/world-systems/professions.md); the AAA rendering path breadth; total-conversion tooling. → [feature-bar](../../initial-idea/08-feature-bar.md).

## Rules
- **Integration finds the boundary bugs** — when it does, fix them in the owning track, don't patch around them here.
- **The gate is binary** — every box green or it's not v1.0.0. No "mostly."
- **Docs are a gate item**, not an afterthought.

## Definition of Done (track)
The release gate is fully green and `v1.0.0` is tagged: a real, playable, horizontally-scaled, server-authoritative, AI-native MMORPG slice — foundation-complete, so v1.x is content, not rewrites.

## Links
[master plan](README.md) · [08 feature-bar](../../initial-idea/08-feature-bar.md) · [workflow](01-workflow-and-parallelization.md) · all tracks [00](00-foundation.md)–[60](60-content-and-assets.md)
