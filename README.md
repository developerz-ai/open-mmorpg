# 🐉 Open-MMORPG

> **The best open-core MMORPG engine.** WoW-inspired, fully original IP, built by AI agents, in Rust. Engineering goals — fastest server, best-looking graphics, greatest DX for the agents that build it — not measured results yet: no published benchmark baseline (see the [server](docs/initial-idea/03-tech-stack.md#server) and [client](docs/initial-idea/03-tech-stack.md#client) honesty notes). MIT. Gamers-first, not extraction-first.

Open-MMORPG is an **open-source MMORPG core in development**, targeting a horizontally-scalable engine, a playable rendered client and a hostable operator website. Today it includes server libraries and app scaffolds, a headless prediction demo, a data-driven datapack and a Bun/SolidJS operator site. The direction: **original IP only** (never extracted game assets), **data-driven** extensions and **AI-native** development; player AI companions remain a planned integration.

## ✨ Why it's different
Design goals, not a claim that the full game ships today:

| | Open-MMORPG | Traditional WoW-like cores |
|---|---|---|
| IP | original IP required; AI asset generation planned, payloads not shipped | emulates Blizzard data → latent legal risk |
| Scale | horizontal architecture; autoscaled shards and **no realm queues** are targets | fixed realm caps, 20-year-old architecture |
| Zones | **seamless**, no loading screens — planned | retrofitted streaming |
| Content | factions/classes/quests are **data** — extend without forking | hardcoded, fork-to-change |
| Assets | open **glTF / heightmap / zstd** pipeline planned | closed MPQ/WMO/ADT, reverse-engineered tooling |
| Players | **AI companion agents** via MCP — bounded-action scaffold today | none |
| World | reactive NPC/faction AI and weather planned; world-feed HTTP surface exists | scripted, static |
| Stack | **Rust** end-to-end (server · client · tooling) | C++ core, mixed |

## 🧱 What's in the box
- **Server core** — Rust (tokio/axum), deterministic sim, netcode and a UDP transport library. The shard binary's tick loop has no transport wired yet; autoscaling is planned.
- **Game client** — headless Rust prediction demo using shared `sim`. Bevy/wgpu/PBR headful rendering is **not yet implemented (planned: C5/C8)** — [client batches](docs/mvp/v1/40-game-client.md#pr-batches-100-prs). The separate `engine-core` library already uses Bevy subcrates for its headless runtime.
- **Operator website** — Bun + SolidJS, dark-theme and i18n; separate from the Cargo workspace.
- **Companion MCP** — bounded-action scaffold; attaching a player agent to a live companion is planned (never the main character).
- **Modding** — committed datapack manifests, recursive loading/validation and shard boot-time ability loading. A fuel-metered WASM host exists; datapack script binding remains a scaffold. Lua is **not yet implemented (planned: S9)** — [server batches](docs/mvp/v1/30-game-server.md#pr-batches-100-prs).
- **Assets** — only [`assets/README.md`](assets/README.md) and [`assets/manifest.json`](assets/manifest.json) ship; referenced placeholder payloads are absent. Generation, streaming and hot-swapping are planned.

## 📚 Documentation
| Start here | For |
|---|---|
| [docs/initial-idea/](docs/initial-idea/README.md) | The founding spec — **what** we're building and **why** |
| [docs/architecture/](docs/architecture/README.md) | The engineering design — **how** it's built (Rust-concrete) |
| [CLAUDE.md](CLAUDE.md) | The project brain — the contract every AI agent reads first |

Key decisions at a glance:
- **Rust everywhere** — [initial-idea/03-tech-stack.md](docs/initial-idea/03-tech-stack.md)
- **1M+ CCU sharding** — [architecture/02-server-topology.md](docs/architecture/02-server-topology.md)
- **Anti-dupe contract** — [architecture/04-data-and-consistency.md](docs/architecture/04-data-and-consistency.md)
- **Extensibility line** (compiled vs. data) — [architecture/05-ecs-and-scripting.md](docs/architecture/05-ecs-and-scripting.md)

## 🤖 Built by AI agents
This repo is engineered so an AI coding agent ships production-quality work from the first prompt — the whole loop is **great DX → fast agents → more shipped**. Conventions, the exact commands, and where everything goes live in [CLAUDE.md](CLAUDE.md), following the [gold standards](../gold-standards-in-ai/README.md) for AI-first development. Humans review, steer, and operate; agents write the code.

## 🚀 Status
**Implemented foundations; integration still in progress.** The [Cargo workspace](Cargo.toml) contains **14 libraries under `crates/` + five Rust apps + two compiled modules** (`hello-world`, `milestones`). The Bun/SolidJS operator website is a separate workspace. The `content/` datapack, CI definitions and DX scripts are present; this inventory is not a claim of a completed playable realm or a current green CI run.

```
bin/setup      # prereqs, Bun deps, Rust build warm-up, optional Docker, Playwright browsers
bin/dev        # default: gateway + shard + web
bin/dev client # explicit targets: gateway | shard | worldsvc | client | web | all
bin/check      # Rust fmt/clippy/tests (all features), content validation + bench compile, web lint/typecheck/test/build
```

`bin/setup` does not run migrations. Rust dev targets use `cargo run`, not a reload watcher; `all` starts only the default trio above. `bin/check` does not run Playwright browser tests.

- **Game server base** — `apps/shard` advances the authoritative deterministic tick loop and loads datapack abilities and compiled modules. `apps/gateway` supplies edge/auth and `apps/worldsvc` the HTTP read/social plane; this is not yet a wired gateway→shard gameplay path. Persistence and cache libraries exist but are not shard dependencies. The Dragonfly cache backend is **not yet implemented (planned: D6)** — [database batches](docs/mvp/v1/10-database.md#pr-batches-100-prs); `MemoryCache` exists today.
- **Client** — `apps/client`, Linux-first headless prediction today; **headless or headful from one core** remains the design target.
- **Operator website** — `apps/web`, the operator-facing site, with shared `packages/i18n` and `packages/ui`.
- **Moddable by design** — factions/classes/quests are data in [`content/`](content/README.md), not compiled definitions. Rust gameplay extensions live in [`modules/`](docs/architecture/10-modules.md) and require a rebuild.

Next: deployment CI/CD (`.github/workflows/docker.yml` is the image-build base, currently manual). See [architecture/01-monorepo-layout.md](docs/architecture/01-monorepo-layout.md) and [CLAUDE.md](CLAUDE.md).

## 📜 License
MIT. Fork freely, commercial use welcome. We *ask* — never force — that meaningful improvements come back upstream, so we can build the best place for games together. → [initial-idea/01-legal-and-licensing.md](docs/initial-idea/01-legal-and-licensing.md)
