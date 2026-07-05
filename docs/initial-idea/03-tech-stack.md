# 03 — Tech Stack

> **Binding decision: Rust across the whole stack.** This supersedes the original brainstorm (which proposed Go + Godot). Where any older note implies Go/Godot, Rust wins.

## Why Rust (and the honest trade-off)
The original notes picked Go for "higher first-try LLM correctness." We're choosing **Rust anyway**, deliberately:
- **This is the hottest hot-path there is** — a real-time authoritative game server for 1M+ players. Gold-standard rule: reach for Rust when latency/throughput/correctness is the whole game. It is here. → `../gold-standards-in-ai/docs/stack/rust-apis.md`
- **One language, whole stack** — server, client (Bevy), tooling, MCP. Agents context-switch less; the ECS mental model is shared client↔server.
- **Make illegal states unrepresentable** — Rust's type system + `thiserror` is our anti-dupe/anti-cheat ally. No `unwrap` on the hot path; no GC pause spiking tail latency.
- **Trade-off we accept:** slightly lower first-try LLM correctness than Go. We buy it back with a tight DX loop (`cargo check`, `clippy -D warnings`, fast tests), CodeGraph, and the quality net (tests → AI review → human review). See [../architecture/README.md](../architecture/README.md).

## Server
| Concern | Choice |
|---|---|
| Async runtime | **tokio** |
| Gateway / HTTP / auth | **axum** |
| Game netcode (realtime) | **UDP** + custom reliability layer (e.g. `renet`/`quinn`/QUIC); WebSocket fallback for web client |
| ECS | **Bevy ECS** (as a library, headless on the server) or custom — shared component model with client |
| Serialization | **serde** (+ a compact binary wire format: `bincode`/`rkyv`) |
| Scripting (moddable) | **WASM** (sandboxed, language-agnostic) primary; **Lua** (`mlua`) for lightweight content scripts |
| Errors | **thiserror**, typed, stable client codes |

## Client
| Concern | Choice |
|---|---|
| Engine | **Bevy** (Rust, wgpu, ECS) — Rust-native, unifies the stack, best agent DX |
| Rendering | wgpu + **PBR**, glTF assets, modern techniques (clustered lighting, GI where feasible) for best-looking graphics |
| Assets | glTF 2.0 / heightmaps / zstd — [05-asset-and-map-formats.md](05-asset-and-map-formats.md) |

**Client honesty note:** Bevy's renderer is not yet AAA-fidelity out of the box; "best-looking graphics" is an engineering goal we push via wgpu, PBR, and high-quality AI-generated PBR assets. **Godot (with the Rust GDExtension)** is the documented fallback if visual fidelity outpaces Bevy before launch. This is a revisit-able decision — record any change as an ADR in [../architecture/](../architecture/README.md).

## Testing
- Rust built-in `#[test]` + `cargo nextest` for speed.
- Integration tests against real Postgres/Yugabyte + Dragonfly via **testcontainers** in CI.
- `proptest` for invariants (combat math, inventory conservation, ledger correctness).

## Extensibility layer
- **ECS** — new race/class = new component data + system logic, no engine recompile.
- **WASM/Lua scripts** for abilities/quests/AI logic — sandboxed, safe for untrusted operator content.
- Detail: [06-modding-and-extensibility.md](06-modding-and-extensibility.md).
