# ADR-0001: Engine Crate Family and Feature Strategy

**Status:** Accepted  
**Date:** 2026-07-05  
**Deciders:** architecture team

---

## Context

The engine must run headless (CI, server-side sim re-use, agent harness) and headful (rendered desktop client). We have to decide how to take the Bevy dependency: as the single umbrella `bevy` crate or as individually selected sub-crates.

## Decision

**Depend on individual Bevy sub-crates, not the `bevy` umbrella crate.**

The workspace pins **Bevy 0.19** across all engine crates (see `[workspace.dependencies]` in `Cargo.toml`).

### Crate family

| Sub-crate | `default-features` | Extra features enabled | Why |
|---|---|---|---|
| `bevy_app` | off | `bevy_reflect`, `std` | App/Plugin lifecycle — headless entry point |
| `bevy_ecs` | off | `bevy_reflect`, `std`, `multi_threaded` | ECS core; reflection needed for agent introspection |
| `bevy_reflect` | off | `std` | Type-system introspection; serialization; agent DX |
| `bevy_math` | off | `std` | Vec/Quat/Mat — shared between sim and rendering |
| `bevy_transform` | off | `std` | Transform component; required by physics & scene |
| `bevy_time` | off | `std` | Fixed-timestep tick scheduler |
| `bevy_state` | off | `std` | State machine (game state transitions) |
| `bevy_scene` | off | `bevy_reflect`, `std` | `.scn.ron` scene (de)serialization; BSN on roadmap |
| `bevy_asset` | off | `std`, `multi_threaded` | Asset server, hot reload |
| `bevy_render` | off | *(rendering plugin only)* | GPU-side rendering — headful builds only |
| `bevy_pbr` | off | *(rendering plugin only)* | PBR materials, clustered forward+ |
| `bevy_audio` | off | `std` | Audio plugin |
| `bevy_ui` | off | `std` | Retained HUD / Taffy layout |
| `bevy_winit` | off | *(headful only)* | Window/event loop — excluded in headless |

`bevy_render`, `bevy_pbr`, and `bevy_winit` are **feature-gated** inside the engine and compiled only when the `render` Cargo feature is enabled. Headless targets (server sim, CI) never activate `render`.

### Sub-crate vs umbrella

The umbrella `bevy` crate is a convenience re-export. Depending on it pulls **all** optional sub-crates with their defaults, including windowing, audio, and platform-specific renderer backends, which are useless (and expensive) in a headless context. Pulling sub-crates individually means:

1. Compile times scale with what you use.
2. Headless targets have **zero GPU/windowing dependencies** — the crate graph stays headless without implying `no_std`.
3. Feature drift is explicit: adding a Bevy feature is a conscious PR-level decision, not a transitive surprise.

### DLSS / all-features exclusion

Enabling Bevy's `bevy_render/webgpu` or any DLSS integration (vendor-specific, proprietary SDK) is **permanently excluded** from `default-features = false`. Rationale:

- The WebGPU backend adds WASM dependencies that break native-only targets.
- DLSS requires the NVIDIA NGX SDK under a non-MIT license; including it by default would contaminate the MIT license of this open-core codebase.
- Upstream Bevy marks DLSS as an optional feature behind `bevy_render/dlss`. We never activate it in the workspace default feature set. A downstream operator building a commercial native binary may enable it in their own overlay `Cargo.toml`, but the open-core repo does not.

Any PR that adds `all-features = true` to `cargo check`/`clippy`/`nextest` calls in `bin/check` **must be rejected** for this reason.

### Determinism boundary

The determinism contract — *same inputs → same output across runs and machines* — applies to:

- `crates/sim` and every crate it imports.
- Any `bevy_ecs` system added to the `FixedUpdate` schedule that the server also runs (re-sim path).

It does **not** apply to:

- Rendering systems (`bevy_render`, `bevy_pbr`, GPU shaders) — floating-point rounding, GPU driver, and temporal effects are permitted to vary.
- Asset loading / decompression paths (I/O timing is non-deterministic by design).
- UI layout (`bevy_ui` / Taffy) — pixel output may vary by platform.

To keep the boundary enforceable:

1. `crates/sim` has **no** dependency on `bevy_render`, `bevy_pbr`, or `bevy_winit`.
2. Systems that run in `FixedUpdate` and touch shared state use **integer or fixed-point arithmetic** (`i32`, `i64`, `u64`, or `fixed` crate) where cross-platform determinism is required. `f32`/`f64` are allowed only when the output is cosmetic (animation interpolation, particle positions).
3. CI runs `cargo nextest run --workspace` **without** the `render` feature, confirming the headless subset compiles and tests pass on every PR.

## Consequences

- Every new engine crate added to `crates/` states explicitly which Bevy sub-crates it needs and why.
- `render` feature gating is tested: `bin/check` runs both `--no-default-features` (headless) and `--features render` (headful) compile checks.
- No Bevy sub-crate version is bumped without a PR that explains the change — version pinning is workspace-wide, not per-crate.

## Alternatives Considered

| Alternative | Rejected because |
|---|---|
| `bevy` umbrella, all features on | Forces GPU/windowing into headless sim; non-MIT DLSS entanglement risk |
| Forking Bevy | Maintenance cost; we contribute upstream instead |
| Godot (via `gdext`) | Rust-first but not a Rust-native crate library; ECS is opaque |
| Engine-from-scratch | 18-month delay; Bevy already ships the hard parts (ECS, reflect, asset server) |

---

*See also:* [game-engine spec README](../../specs/game-engine/README.md) · [tech-stack decision](../../initial-idea/03-tech-stack.md) · [Bevy 0.19 release](https://bevy.org/news/bevy-0-19/)
