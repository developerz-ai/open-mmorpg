# Engine Core

> The reusable foundation: an **ECS runtime** (`App` + `Plugin` + parallel schedule), a **reflection** type system, and a **headless-first** design so the same core runs in CI, an agent harness, and the rendered client. Everything else in the engine is a plugin on top. → [engine README](../README.md)

## What it does
Owns the app lifecycle and the ECS. State lives in **components** on **entities**; behavior lives in **systems** the scheduler runs in parallel by data-access signature. Features (rendering, audio, physics, our game) are **plugins** that register components/systems/resources at build time. This is Bevy's model, chosen because `bevy_ecs` is usable **standalone** and the whole engine composes from crates rather than a monolith. → [`bevy_ecs`](https://docs.rs/bevy_ecs/latest/bevy_ecs/), [App/Plugin lifecycle](https://deepwiki.com/bevyengine/bevy/3.1-app-lifecycle-and-plugin-architecture).

## Design
- **App = plugins + schedules.** A game is `App::new().add_plugins((EnginePlugins, GamePlugin)).run()`. Server and headless client swap `RenderPlugin`/`WinitPlugin` out; nothing else changes ([platform](../../client/platform/README.md)).
- **Parallel schedule.** The executor runs systems concurrently when their `&`/`&mut` access is disjoint; ordering only where declared (`.before()`/`.after()`/system sets). A slow system can't silently serialize the frame — the same discipline the [shard tick loop](../../game-server/tick-loop/README.md) needs.
- **Fixed vs render timestep.** Simulation lives in **`FixedUpdate`** (fixed dt, deterministic); rendering/input in `Update` (per-frame). Decoupling them is what makes headless == headful results ([prediction-core](../../client/prediction-core/README.md)).
- **Reflection (`bevy_reflect`).** Runtime introspection of any `#[derive(Reflect)]` type: enumerate fields, get/set by name, (de)serialize without static knowledge. This is the substrate for [scene](../scene/README.md) I/O, the [editor](../editor/README.md) inspector, and [agent introspection](../ai-native-dx/README.md). A type not registered in the type registry is invisible to tools — registration is mandatory, not optional.
- **Newtypes over primitives.** `EntityId` mirrors of protocol IDs, not raw `u64` — the [Rust convention](../../../../CLAUDE.md) applies in engine code too; the type system is the first correctness net.

## Engine-as-library (leanness)
| Concern | Rule |
|---|---|
| Dependencies | A game pulls `bevy_*` subsystem crates + our plugins by **feature flag**; unused subsystems don't compile in. |
| Binary size / compile time | Minimal app = `bevy_ecs` + a schedule. Full client adds render/audio/UI plugins. Never force-link the renderer to run logic. |
| Reuse boundary | Engine crates know nothing about the MMORPG. Game rules live in the [client](../../client/README.md)/[server](../../game-server/README.md), not here. One crate = one reason to change. |

## Distilled from the references
| Source | Adopt |
|---|---|
| Bevy ECS + App/Plugin | Composition via plugins; standalone `bevy_ecs` headless on the server |
| Bevy required components / relationships (0.15/0.16+) | Model composition via `#[require(...)]` and built-in entity relations, not fat bundles ([0.16 notes](https://bevy.org/news/bevy-0-16/)) |
| `bevy_reflect` | Reflection-first: register every gameplay type; tools/editor/agent read the registry |
| Unity DOTS (data-oriented) | Data-oriented layout for cache-friendly, parallel simulation |

## Rules
- **One core, two heads.** Never branch logic on "are we rendering?" — add/remove the render plugin instead ([platform](../../client/platform/README.md)).
- **Sim in `FixedUpdate`, deterministic.** No wall-clock, no `Math.random`, stable system order where the server reuses it ([core determinism](../../game-server/tick-loop/README.md)).
- **Register every reflected gameplay type.** Unregistered → invisible to editor, scene I/O, and agents ([ai-native-dx](../ai-native-dx/README.md)).
- **No `std::sync::Mutex` on the schedule path** — ECS access is the concurrency model; use it. `tokio::sync` for async I/O only ([CLAUDE.md](../../../../CLAUDE.md)).

## Links
[scene](../scene/README.md) · [ai-native-dx](../ai-native-dx/README.md) · [rendering](../rendering/README.md) · [platform](../../client/platform/README.md) · [prediction-core](../../client/prediction-core/README.md) · [tick-loop](../../game-server/tick-loop/README.md)
