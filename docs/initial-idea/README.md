# 💡 Initial Idea — Open-MMORPG

> The founding spec. Every decision made before a line of code. An agent reads this to know **what** we're building and **why** before touching **how** ([architecture/](../architecture/README.md)).

**One line:** An open-source, AI-native, next-gen MMORPG — WoW-inspired, fully original IP, built by AI agents, in Rust, with the best-looking graphics and the fastest server. Gamers-first, not extraction-first.

## Read in order
| # | Doc | Covers |
|---|---|---|
| 00 | [overview.md](00-overview.md) | Vision, core principles, what makes us better |
| 01 | [legal-and-licensing.md](01-legal-and-licensing.md) | Original IP, MIT, trademark |
| 02 | [architecture-and-scaling.md](02-architecture-and-scaling.md) | 1M+ CCU, sharding, DDoS, anti-dupe |
| 03 | [tech-stack.md](03-tech-stack.md) | **Rust everywhere** (the binding decision) |
| 04 | [data-storage.md](04-data-storage.md) | 4-tier memory hierarchy |
| 05 | [asset-and-map-formats.md](05-asset-and-map-formats.md) | glTF / heightmap / zstd — open formats |
| 06 | [modding-and-extensibility.md](06-modding-and-extensibility.md) | Data-driven core, plugin manifests |
| 07 | [ai-agent-mcp-integration.md](07-ai-agent-mcp-integration.md) | Player AI companions via MCP |
| 08 | [feature-bar.md](08-feature-bar.md) | Match/exceed the current WoW baseline |
| 09 | [gta6-inspiration.md](09-gta6-inspiration.md) | Reactive world systems borrowed from GTA |

## The one decision that changed since the notes
The original brainstorm proposed **Go (server) + Godot (client)** for agent-friendliness. **Superseded.** The binding decision is **Rust across the whole stack** — server, client (Bevy), tooling, MCP. Rationale + the honest trade-offs live in [03-tech-stack.md](03-tech-stack.md). Where any older doc implies Go/Godot, Rust wins.

## Status
Pre-scaffold. This is the spec an agent army executes against. Next step: scaffold the monorepo per [architecture/README.md](../architecture/README.md), then `/planx` the first milestone (see [CLAUDE.md](../../CLAUDE.md)).
