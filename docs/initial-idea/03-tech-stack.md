# 03 — Tech Stack

> **Binding decision: Rust across the whole stack.** This supersedes the original brainstorm (which proposed Go + Godot). Where any older note implies Go/Godot, Rust wins.

## Why Rust (and the honest trade-off)
The original notes picked Go for "higher first-try LLM correctness." We're choosing **Rust anyway**, deliberately:
- **This is the hottest hot-path there is** — a real-time authoritative game server for 1M+ players. Gold-standard rule: reach for Rust when latency/throughput/correctness is the whole game. It is here. → `../gold-standards-in-ai/docs/stack/rust-apis.md`
- **One language for server/client/tooling/MCP** — client targets Bevy but currently ships a headless prediction demo. The operator web is the Bun/SolidJS exception; shared simulation keeps the client↔server model aligned.
- **Make illegal states unrepresentable** — Rust's type system + `thiserror` is our anti-dupe/anti-cheat ally. No `unwrap` on the hot path; no GC pause spiking tail latency.
- **Trade-off we accept:** slightly lower first-try LLM correctness than Go. We buy it back with a tight DX loop (`cargo check`, `clippy -D warnings`, fast tests), CodeGraph, and the quality net (tests → AI review → human review). See [../architecture/README.md](../architecture/README.md).

## Server
| Concern | Choice |
|---|---|
| Async runtime | **tokio** |
| Gateway / HTTP / auth | **axum** |
| Game netcode (realtime) | Custom binary framing/reliability in `netcode`; **UDP** transport and connection lifecycle in `transport`. QUIC and WebSocket fallback are not current implementations. |
| ECS | `ecs-core`: shared **plain Rust structs** and pure systems. `engine-core`: implemented **Bevy** app/ECS/reflection/time runtime. |
| Serialization | **serde** message types in `protocol`, with JSON round-trip tests; custom netcode framing, not a shipped `bincode`/`rkyv` message codec. |
| Scripting (moddable) | **WASM/Wasmtime** host with fuel metering and capability API implemented; **Lua** (`mlua`) not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)). Datapack script loading and persistence-backed grants remain unwired. |
| Errors | **thiserror**, typed, stable client codes |

**Server honesty note:** "Fastest server" is an engineering goal, not a measured result — no benchmark baseline is published yet, so no throughput comparison against other cores is claimed. We push it with the levers this doc already picks: a deterministic sim, UDP netcode and horizontal shards. The [client note](#client) below frames graphics the same way. Revisit-able decision — record any change as an ADR in [../architecture/](../architecture/README.md).

## Client
| Concern | Choice |
|---|---|
| Engine | Current `apps/client`: **headless prediction demo** using shared simulation. Headful **Bevy** client not yet implemented (planned: [C5/C8](../mvp/v1/40-game-client.md#pr-batches-100-prs)). |
| Rendering | **wgpu + PBR** client rendering not yet implemented (planned: [C5/C8](../mvp/v1/40-game-client.md#pr-batches-100-prs)); glTF, clustered lighting and GI remain graphics targets, not shipped client features. |
| Assets | Target formats: glTF 2.0 / heightmaps / zstd — [05-asset-and-map-formats.md](05-asset-and-map-formats.md). Asset pipeline not yet implemented (planned: [E3](../mvp/v1/20-game-engine.md#pr-batches-100-prs)); `assets/` contains a README and manifest, not the referenced payloads. |

**Client honesty note:** Bevy's renderer is not yet AAA-fidelity out of the box; "best-looking graphics" is an engineering goal we push via wgpu, PBR, and high-quality AI-generated PBR assets. **Godot (with the Rust GDExtension)** is the documented fallback if visual fidelity outpaces Bevy before launch. This is a revisit-able decision — record any change as an ADR in [../architecture/](../architecture/README.md).

## Testing
- Rust built-in `#[test]` + `cargo nextest` for speed.
- CI runs live database integration tests against a **Postgres 16 service** in GitHub Actions, not testcontainers, Yugabyte or Dragonfly. Yugabyte's Postgres-wire compatibility is the intended deployment contract, not evidence of testing Yugabyte in CI.
- Existing `proptest` coverage exercises **sim** determinism/combat and **netcode** delta reconstruction, quantization and framing. Inventory-conservation and ledger property invariants are not yet implemented (planned: [D5/D7](../mvp/v1/10-database.md#pr-batches-100-prs)).

## Extensibility layer
- **ECS** — new race/class data using existing systems needs no engine recompile; adding compiled system logic still needs a build.
- **WASM** sandbox primitives ship; abilities/quests/AI script integration and **Lua** remain targets, not yet implemented (planned: [S9](../mvp/v1/30-game-server.md#pr-batches-100-prs)).
- Detail: [06-modding-and-extensibility.md](06-modding-and-extensibility.md).
