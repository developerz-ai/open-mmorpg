# 🐉 Open-MMORPG

> **The best open-core MMORPG engine.** WoW-inspired, fully original IP, built by AI agents, in Rust. Fastest server, best-looking graphics, greatest DX for the agents that build it. MIT. Gamers-first, not extraction-first.

Open-MMORPG is a **next-gen, open-source MMORPG core** — a clean, modern, horizontally-scalable engine plus a playable client and a hostable operator website. It's what AzerothCore/TrinityCore would be if it were built today: **original IP** (zero legal exposure), **data-driven** (operators extend without forking), and **AI-native** (agents write the code; players get AI companions).

## ✨ Why it's different
| | Open-MMORPG | Traditional WoW-like cores |
|---|---|---|
| IP | 100% original, AI-generated assets — MIT-safe to commercialize | emulates Blizzard data → latent legal risk |
| Scale | horizontal from day 1, autoscaled shards, **no realm queues** | fixed realm caps, 20-year-old architecture |
| Zones | **seamless**, no loading screens by design | retrofitted streaming |
| Content | factions/classes/quests are **data** — extend without forking | hardcoded, fork-to-change |
| Assets | open **glTF / heightmap / zstd** — any tool, any AI pipeline | closed MPQ/WMO/ADT, reverse-engineered tooling |
| Players | **AI companion agents** via MCP | none |
| World | reactive NPC/faction AI, weather, world feed (GTA-inspired) | scripted, static |
| Stack | **Rust** end-to-end (server · client · tooling) | C++ core, mixed |

## 🧱 What's in the box
- **Server core** — Rust (tokio/axum), autoscaled zone shards, deterministic sim, UDP netcode.
- **Game client** — Rust (Bevy), wgpu + PBR, streams open-format assets.
- **Operator website** — Bun + SolidJS, dark-theme, i18n, fully brandable — every operator can host their realm's site out of the box.
- **Companion MCP** — players attach their own AI agent to a bounded companion (never the main char).
- **Modding** — Minecraft-datapack-style manifests; WASM/Lua scripts; add a faction with config + assets.

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
**Scaffolded & green.** The Cargo workspace (9 crates + 5 apps), the Bun/SolidJS operator website, the data-driven `content/` datapack, CI (fmt · clippy `-D warnings` · tests · web lint/typecheck/build), and the DX scripts are all in place and passing.

```
bin/setup      # fresh clone → ready (deps, build, docker)
bin/dev        # boot gateway + shard + web
bin/check      # the gate: fmt + clippy -D warnings + tests (+ web)
```

- **Game server base** — `apps/shard` (headless authoritative zone, deterministic tick loop) behind `apps/gateway` (edge/auth) and `apps/worldsvc` (cross-shard), powered by `crates/{sim,ecs-core,netcode,protocol,persistence,cache}`.
- **Client** — `apps/client`, Linux-first, runs **headless or headful from one core** (deterministic, test- and agent-drivable).
- **Operator website** — `apps/web`, the brandable site each operator hosts for their realm.
- **Moddable by design** — factions/classes/quests are data in [`content/`](content/README.md); operators extend or fork the client without touching the compiled core.

Next: deployment CI/CD (`.github/workflows/docker.yml` is the image-build base, currently manual). See [architecture/01-monorepo-layout.md](docs/architecture/01-monorepo-layout.md) and [CLAUDE.md](CLAUDE.md).

## 📜 License
MIT. Fork freely, commercial use welcome. We *ask* — never force — that meaningful improvements come back upstream, so we can build the best place for games together. → [initial-idea/01-legal-and-licensing.md](docs/initial-idea/01-legal-and-licensing.md)
