# 🚀 v1.0.0 — The Big Plan

> **What this folder is.** The **execution plan** for shipping Open-MMORPG **v1.0.0**. The [specs](../../specs/README.md) say *what each subsystem is*; the [architecture](../../architecture/README.md) says *how the system is shaped*; **this folder says what to build, in what order, and who builds it** — sized for a fleet of AI agents working in parallel.
>
> One file = one **big idea** = a **track** ≈ **~100 agent PRs**. Read the master plan here, pick your track, read its spec, ship **fat, green, machine-reviewed PRs** — one per batch.

## The shape of v1.0.0
A **vertical slice that is a real, playable, horizontally-scaled MMORPG**: log in on the [web](50-web-client.md), download the [Bevy client](40-game-client.md), connect through the [gateway](30-game-server.md) to an authoritative [shard](30-game-server.md), move/fight/loot with [server-authoritative sim](30-game-server.md) + client prediction, persist ownership to [Yugabyte](10-database.md) with no dupes, in a world built from [data-driven content](60-content-and-assets.md) and [AI-generated assets](60-content-and-assets.md). Two factions, a handful of classes, one starting zone, one dungeon, an auction house. Not feature-complete — **foundation-complete**: every load-bearing primitive shipped, tested, and reusable so v1.x is *content*, not *rewrites*.

**v1.0.0 is done when** the [release gate](70-integration-and-release.md) passes: a headless agent connects, plays the slice end-to-end, asserts state; the web portal shows that character's armory; two shards autoscale and hand a player across a boundary; and `bin/check` is green across every crate and app.

## The seven tracks
Ordered by dependency. Foundation first, then engines, then clients, then content, then integration.

| # | Track | Owner subagent | Big idea | Reads |
|---|---|---|---|---|
| [00](00-foundation.md) | **Foundation** | *all* (shared) | The workspace, primitives, wire types, errors, CI gate. **Everything else builds on this.** | [layout](../../architecture/01-monorepo-layout.md) |
| [10](10-database.md) | **Database & Persistence** | [`db`](../../../.claude/agents/db.md) | The only place ownership is written — Yugabyte txns, anti-dupe by type, migrations, Dragonfly ephemeral. | [persistence](../../specs/game-server/persistence/README.md) · [04](../../architecture/04-data-and-consistency.md) |
| [20](20-game-engine.md) | **Game Engine** | [`gameengine`](../../../.claude/agents/gameengine.md) | The reusable Bevy-based runtime: ECS core, rendering, assets, scene, animation, physics, audio, UI, MCP editor. | [game-engine](../../specs/game-engine/README.md) |
| [30](30-game-server.md) | **Game Server** | [`server`](../../../.claude/agents/server.md) | The authoritative runtime: tick loop, netcode, world model, sharding, combat, economy, scripting, security. | [game-server](../../specs/game-server/README.md) |
| [40](40-game-client.md) | **Game Client** | [`gameclient`](../../../.claude/agents/gameclient.md) | The thin, honest player app: shared deterministic sim, prediction, input→intent, HUD, headless-first. | [client](../../specs/client/README.md) |
| [50](50-web-client.md) | **Web Client** | [`webclient`](../../../.claude/agents/webclient.md) | The operator portal: Bun+SolidJS, account/auth, armory, auction browser, world feed, dark, i18n, brandable. | [web-client](../../specs/web-client/README.md) |
| [60](60-content-and-assets.md) | **Content & Assets** | [`content`](../../../.claude/agents/content.md) | The data-driven game layer + the **AI asset pipeline** (Meshy.ai → glTF/KTX2/Opus): factions, classes, zone, dungeon. | [gameplay](../../specs/gameplay/README.md) · [content](../../../content/README.md) |
| [70](70-integration-and-release.md) | **Integration & Release** | *all* (shared) | The vertical slice, E2E tests, the release gate, the tag. Where the tracks meet. | [feature-bar](../../initial-idea/08-feature-bar.md) |

## Wave sequencing — how ~700 PRs land without gridlock
Tracks are gated, not free-for-all. A wave opens when its dependencies expose stable interfaces (not when they're *finished* — see [contract-first](01-workflow-and-parallelization.md#contracts-first)).

```
Wave 0  ── Foundation (00) ─────────────────────────────────────────────┐  BLOCKS ALL
                                                                          ▼
Wave 1  ── Database (10) ── Engine-core (20) ── Content-schema seed (60) ──┐  parallel
                │                │                        │                 ▼
Wave 2  ── Server (30) ────── Engine-render/scene (20) ── Content data (60) ┐  parallel
                │                │                        │                  ▼
Wave 3  ── Client (40) ─────── Web-client (50) ───────── AI assets (60) ─────┐  parallel
                                                                             ▼
Wave 4  ── Integration & Release (70) ── vertical slice · E2E · v1.0.0 tag
```

- **Wave 0 blocks everything.** No track starts until the workspace compiles and `bin/check` is green on an empty skeleton.
- **Within a wave, tracks are independent** — different crates/apps, different subagents, no shared files. That's the whole point of the [crate split](../../architecture/01-monorepo-layout.md).
- **Cross-track needs go through published interfaces** ([`protocol`](../../../crates/protocol), [`content-schema`](../../../crates/content-schema), the gateway/worldsvc HTTP contracts), never by reaching into another track's internals.

## How an agent uses this plan
1. **Pick a track** (or be assigned one — usually your subagent domain).
2. **Read its doc + its spec.** The track doc lists the exact specs. Don't invent design; the specs already decided it.
3. **Take the next open batch** — each track slices its ~100 fat PRs into ordered batches, each with a Definition of Done.
4. **Ship one fat, green PR per batch.** Feature-complete, tests included; files stay ≤300 LOC ([conventions](01-workflow-and-parallelization.md#pr-conventions)).
5. **Merge via the loop** with [`/merge-pr`](../../../.claude/skills/merge-pr/SKILL.md) — CI + CodeRabbit reviewed and fixed until mergeable.

→ **Working rules, parallelization, and PR conventions: [01-workflow-and-parallelization.md](01-workflow-and-parallelization.md).**

## Non-negotiables (inherited — do not re-litigate → [CLAUDE.md](../../../CLAUDE.md))
1. **Rust everywhere** except [`apps/web`](50-web-client.md) (Bun+SolidJS).
2. **Server-authoritative, always** — client sends [`Intent`](../../../crates/protocol/src/lib.rs), never state.
3. **Ownership → one Yugabyte txn** ([10](10-database.md)). The dupe path must not compile.
4. **Content is data, core is compiled** — [60](60-content-and-assets.md) never triggers a `cargo build` of the core.
5. **Determinism where shared** — [`sim`](../../../crates/sim/src/lib.rs) is bit-identical client and server.
6. **Everything headless-testable, AI-first** — if an agent can't assert it in CI without a GPU, it isn't done.
7. **Horizontal from day 1** — autoscaled shards, no realm caps, no queues.

## Scope discipline — what v1.0.0 is NOT
Level cap for the slice is small (not the [full 1→300 arc](../../specs/gameplay/progression/ages.md)); one zone, one dungeon, one raid-lite; no housing, no professions depth, no era-server tooling, no full talent trees. Those are **v1.x content on the finished foundation** — logged in [70](70-integration-and-release.md#post-v1-backlog), not built now. **We build primitives once; we build content forever.**

> Each track doc ≤ ~2 screens: the big idea, the PR batches, the interfaces it owns, the Definition of Done, and links. Grows past that → split it. Same SRP rule as the code.
