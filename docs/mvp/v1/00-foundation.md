# 00 — Foundation (Wave 0) · blocks everything

> **Big idea.** Before any track opens, the workspace must **compile, test, and gate green on an empty skeleton**. This track builds the *primitives every other track imports*: the Cargo workspace, the wire protocol, the newtypes, the typed errors, the content schema shell, and the `bin/` DX trio + CI. **Owner: shared** — but land it *first and small*, because a wobble here breaks all seven tracks. → [monorepo layout](../../architecture/01-monorepo-layout.md)

## Why this is one track
These are the types and scripts that appear in every other track's `use` statements and every CI run. They have **one reason to change** and must be **stable early**, so they ship as a single coordinated wave, not drip-fed. Get the [contracts](01-workflow-and-parallelization.md#contracts-first) right here and Waves 1–4 rarely touch each other.

## Reads
[01 monorepo-layout](../../architecture/01-monorepo-layout.md) · [03 tech-stack](../../initial-idea/03-tech-stack.md) · [protocol spec](../../specs/game-server/netcode/README.md) · gold standards `stack/rust-apis.md`, `architecture/solid-srp.md`.

## PR batches (~100 PRs)
These batches describe roadmap scope and acceptance goals, not a claim that every goal ships today. Current implementation notes are called out where the original target differs.

| Batch | Scope | Definition of Done |
|---|---|---|
| **F1 · Workspace** | Root `Cargo.toml` workspace, all 9 crate + 6 app skeletons compile as empty libs/bins; `rust-toolchain.toml`; `.gitignore`. | `cargo build` green, zero warnings. |
| **F2 · `errors`** | [`crates/errors`](../../../crates/errors): `thiserror` enums, stable client-facing codes, no leaked internals. One error type per domain boundary. | Every other crate depends on this for its `Result`. |
| **F3 · Newtypes** | `AccountId`, `CharacterId`, `ItemId`, `ShardId`, `ZoneId`, `EntityId`, `Tick` — in [`protocol`](../../../crates/protocol) or a `types` module. `serde`, `Display`, `From` guarded. | No raw `u64` crosses a public API in any track. |
| **F4 · `protocol`** | [`crates/protocol`](../../../crates/protocol): shared **serde message types**, the single source of truth server↔client. Current round-trip tests use JSON; custom binary framing lives in `netcode`, not a shipped `bincode` codec in `protocol`. | Versioned `Intent`/`Snapshot`/`Delta` contracts and round-trip coverage; [30](30-game-server.md) & [40](40-game-client.md) both build against them. |
| **F5 · `content-schema` shell** | [`crates/content-schema`](../../../crates/content-schema): `CONTENT_API_VERSION`, the top-level `Manifest`/`Datapack` types, a `validate()` entry point that fails loud. | [60](60-content-and-assets.md) can add typed defs; boot-time validation compiles. |
| **F6 · `ecs-core` shell** | [`crates/ecs-core`](../../../crates/ecs-core): shared **plain-struct components and pure systems — no I/O**, not a Bevy ECS re-export. Bevy app/ECS/reflection/time are used separately by `engine-core`. | [20](20-game-engine.md)/[30](30-game-server.md) share components; dep graph acyclic. |
| **F7 · DX trio + CI** | `bin/setup`, `bin/dev`, `bin/check`, `bin/fmt`; Docker compose for Yugabyte + Dragonfly. Current GitHub Actions runs separate Rust/web commands plus a live-Postgres integration job, not `bin/check`. | Local `bin/check` green: Rust fmt + all-feature clippy/tests, content validation + schema benchmark compilation, web lint/typecheck/Bun tests/build. CI's separate jobs must also pass. |
| **F8 · Docs seed** | `README.md` at repo root pointing at this plan; `CONTRIBUTING`/agent-onboarding note; the [doc conventions](01-workflow-and-parallelization.md#documentation) that every later PR follows. | A fresh agent can clone → `bin/setup` → `bin/check` → pick a track. |

## Interfaces this track owns (the contracts everyone imports)
- **`crates/protocol`** — wire types. Changing these is a cross-track event; version it ([netcode](../../specs/game-server/netcode/README.md)).
- **`crates/errors`** — the shared `Error`/`Result` vocabulary.
- **`crates/content-schema`** — `CONTENT_API_VERSION` + manifest types ([content-scripting](../../specs/game-server/content-scripting/README.md)).
- **`bin/check`** — the local Rust/content/web gate ([step definitions](../../../scripts/lib/gate.ts)); Rust tests use nextest when available, otherwise `cargo test`. It compiles content-schema benchmarks but does not execute them or run Playwright. No committed local hook invokes it. [CI](../../../.github/workflows/ci.yml) runs separate commands and additionally exercises live-Postgres integration tests; a green local gate does not establish a green CI run.

## Rules
- **`ecs-core` stays pure** (types + logic, no I/O). `content-schema` types, validation and byte-based `load_manifest` are pure; its `load_manifest_dir` split-tree loader performs filesystem I/O. Absence of `tokio`/`sqlx` dependencies does not enforce absence of I/O.
- **No `unwrap`/`expect`** outside `main`/tests, from PR #1. The clippy config forbids it.
- **Newtypes over primitives** land in F3 so no track ever ships a raw-`u64` API to rip out later.
- **This track finishes before Wave 1 opens.** Its interfaces may still *evolve*, but they must *exist and compile*.

## Definition of Done (track)
`bin/check` green on the skeleton; every downstream track's spec can name the exact `protocol`/`errors`/`content-schema`/`ecs-core` symbol it will build against; a clean clone bootstraps in one command.

## Links
[layout](../../architecture/01-monorepo-layout.md) · [workflow](01-workflow-and-parallelization.md) · [database](10-database.md) · [engine](20-game-engine.md) · [server](30-game-server.md) · [CLAUDE.md](../../../CLAUDE.md)
