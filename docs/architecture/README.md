# 🏗️ Architecture

> How we build the best **open-core MMORPG engine**. The [initial idea](../initial-idea/README.md) says *what/why*; this says *how*. Every choice defaults to the [gold standards](../../CLAUDE.md#standards) and deviates only with a reason (recorded as an ADR).

## ADRs

| Decision | Covers |
|---|---|
| [decisions/0001-engine-crate-family-and-features.md](decisions/0001-engine-crate-family-and-features.md) | Bevy 0.19 sub-crate vs umbrella, DLSS/all-features exclusion, determinism boundary |

## Index
| Doc | Covers |
|---|---|
| [01-monorepo-layout.md](01-monorepo-layout.md) | Cargo workspace: `crates/` + `apps/` + `content/` |
| [02-server-topology.md](02-server-topology.md) | Gateway → shards → cross-shard bus; autoscaling |
| [03-netcode-and-sharding.md](03-netcode-and-sharding.md) | UDP netcode, tick loop, zone/session merge-split |
| [04-data-and-consistency.md](04-data-and-consistency.md) | 4-tier memory, transactional ownership, anti-dupe |
| [05-ecs-and-scripting.md](05-ecs-and-scripting.md) | Bevy ECS core, WASM/Lua modding, data-driven content |
| [06-world-and-assets.md](06-world-and-assets.md) | glTF/heightmap pipeline, streaming, AI asset gen |
| [07-mcp-companions.md](07-mcp-companions.md) | Player companion MCP server design |
| [08-security-anticheat.md](08-security-anticheat.md) | Server-authority, DDoS, dupe prevention, sandboxing |
| [09-operator-web.md](09-operator-web.md) | Base operator website — Bun + SolidJS, dark-only, i18n |
| [10-modules.md](10-modules.md) | Compiled modules — AzerothCore-style, pure Cargo, fork without merge conflicts |

## Binding decisions (the ones an agent must not re-litigate)
1. **Rust everywhere** — server, client (Bevy), tooling, MCP. → [../initial-idea/03-tech-stack.md](../initial-idea/03-tech-stack.md)
2. **Server-authoritative, always.** Client never trusted for state.
3. **Ownership writes → YugabyteDB directly.** Never through cache. #1 anti-dupe rule.
4. **Content is data, core is compiled.** The line in [05](05-ecs-and-scripting.md) is the extensibility contract.
5. **Open formats only** — glTF / heightmap / zstd. No proprietary tooling.
6. **Horizontal from day 1.** No fixed realm caps.

## Design values (from gold standards)
- **Low undefined behavior** — typed errors (`thiserror`), no `unwrap` on hot paths, make illegal states unrepresentable.
- **SOLID/SRP** — Rust files ≤300 LOC, one crate = one reason to change.
- **Tests always** — unit for pure logic, integration via testcontainers, `proptest` for invariants (inventory conservation, ledger correctness).
- **Fast inner loop** — `cargo check`/`nextest`/`clippy` gate; CodeGraph for structure.

## How to extend this folder
New cross-cutting decision → add an ADR under `decisions/NNN-<slug>.md`, link it here. Never edit a shipped decision's meaning silently — supersede it.
