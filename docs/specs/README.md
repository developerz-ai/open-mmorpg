# 📐 Specs

> **What each subsystem is** — tight enough to implement against. The [architecture](../architecture/README.md) folder says *how the system is shaped*; the [initial-idea](../initial-idea/README.md) folder says *what & why*; this folder says *how each subsystem behaves*. The [v1.0.0 plan](../mvp/v1/README.md) says *what to build, in what order, and who builds it* against these specs.

## Domains
| Domain | Covers | Owner track |
|---|---|---|
| [game-engine/](game-engine/README.md) | The reusable Bevy-based runtime: ECS core, rendering, assets, animation, scene, physics, audio, UI, AI-native DX, editor | [20](../mvp/v1/20-game-engine.md) |
| [game-server/](game-server/README.md) | The authoritative runtime: tick loop, netcode, world model, sharding, persistence, combat, economy, scripting, security | [30](../mvp/v1/30-game-server.md) · [10](../mvp/v1/10-database.md) |
| [client/](client/README.md) | The player-facing Bevy app: prediction core, networking, rendering, input, HUD, audio, platform | [40](../mvp/v1/40-game-client.md) |
| [web-client/](web-client/README.md) | The operator web portal (Bun + SolidJS): shell, design system, i18n, data layer, auth, armory, AH, feed, branding | [50](../mvp/v1/50-web-client.md) |
| [gameplay/](gameplay/README.md) | The game itself as **data**: world/cosmology, factions, races, classes, progression, itemization, endgame, world systems | [60](../mvp/v1/60-content-and-assets.md) |

## How to read
Each domain README indexes its subsystem docs and states the non-negotiable principles inherited from [CLAUDE.md](../../CLAUDE.md). Every subsystem doc is ≤ ~1 screen: what it does, the design, the distilled lesson, honest gaps, the rules, and links. Grows past that → split it. Same SRP rule as the code.

## Links
[architecture/](../architecture/README.md) · [initial-idea/](../initial-idea/README.md) · [v1.0.0 plan](../mvp/v1/README.md) · [CLAUDE.md](../../CLAUDE.md)
