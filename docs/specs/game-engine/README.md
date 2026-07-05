# 🛠️ Game-Engine Specs

> **Scope:** the reusable, Unreal-class **runtime engine** — ECS, rendering, assets, animation, scene, physics, audio, UI — plus the two things that make it *ours*: an **AI-native developer experience** and a **data-driven editor**. The [game-server](../game-server/README.md) specs cover the authoritative simulation; the [client](../client/README.md) specs cover the player-facing app. **This folder is the layer both stand on, and that other games can consume standalone.**

## Thesis
Close the gap between big studios and indies. A studio's moat is a $100M engine + a 300-person tools team. Ours is **Bevy + wgpu + a lean set of AAA techniques, wrapped in an engine so introspectable that an AI agent + a solo dev out-ship a studio.** Unreal-class *look*, Godot-class *leanness*, and a first-class **agent** developer surface no incumbent has. → binding: **Rust everywhere, Bevy client** ([tech-stack](../../initial-idea/03-tech-stack.md)).

Three properties, non-negotiable:
1. **Unreal-class visuals** — GPU-driven PBR, virtual geometry, real-time GI, virtualized shadows, temporal upscaling ([rendering](rendering/README.md)). Where Bevy trails Unreal today we say so, and we say what we ship instead.
2. **Not bloated, fast, reusable** — engine-as-**crates**, not a monolith. A game depends on `bevy_*` + our plugins and gets *only* what it uses. Runs **headless from the same core** ([core](core/README.md), [ai-native-dx](ai-native-dx/README.md)).
3. **Best-in-class agent DX** — reflection, determinism, hot reload, declarative data-driven APIs, MCP-drivable editor. An LLM agent authors content and fails at compile time, not at runtime ([ai-native-dx](ai-native-dx/README.md), [editor](editor/README.md)).

## Foundation: Bevy + wgpu (and why)
Bevy is a Cargo workspace of 80+ crates with a `bevy_ecs` core usable **standalone**, a `bevy_reflect` runtime type system, a RON scene format, and a hot-reloading asset server — exactly the introspectable, modular, headless-capable substrate an agent-first engine needs. It's the most-adopted Rust engine and the only one whose ECS ships as a reusable library. → [Bevy architecture](https://bevy.org/learn/), [`bevy_ecs`](https://docs.rs/bevy_ecs/latest/bevy_ecs/), [why Bevy over Fyrox/Flax](https://gamefromscratch.com/rust-game-engines-in-2025/). We **build on Bevy, contribute upstream, and fill the AAA gaps** it hasn't reached yet.

## Index
| Spec | Subsystem | Distilled from |
|---|---|---|
| [core/](core/README.md) | ECS runtime, App/Plugin/schedule, reflection, headless — the reusable foundation | Bevy ECS + `bevy_reflect`; Unity DOTS |
| [rendering/](rendering/README.md) | GPU-driven PBR: clustered forward+, virtual geometry, GI, virtual shadows, upscaling | Unreal Nanite/Lumen/VSM/TSR → Bevy meshlet/Solari + honest gaps |
| [assets/](assets/README.md) | glTF pipeline, asset server, hot reload, LOD/imposters, KTX2 streaming | Bevy asset system; Unreal OFPA/HLOD |
| [animation/](animation/README.md) | Skeletal + GPU skinning, blend/state-machine graph, IK, crowd VAT | glTF skins; Unreal AnimGraph → Bevy animation graph |
| [scene/](scene/README.md) | Scene/prefab format, reflection-driven (de)serialization, determinism | Bevy `.scn.ron`/`.bsn`; Unity prefabs |
| [physics/](physics/README.md) | Collision, kinematic character controller, spatial/LOS queries | Rapier / Avian (Rust-native) |
| [audio/](audio/README.md) | Spatial audio, Opus/Ogg, mixer, lean all-Rust stack | Kira mixer; Bevy audio |
| [ui/](ui/README.md) | Retained HUD UI + immediate-mode tools UI, i18n substrate | `bevy_ui`/Taffy + egui; ICU/Fluent |
| [ai-native-dx/](ai-native-dx/README.md) | **The differentiator** — reflection, determinism, hot reload, MCP, headless, testability | Unreal 5.8 MCP, Unity/Blender MCP → ours, deeper |
| [editor/](editor/README.md) | Data-driven, MCP-drivable editor: outliner, reflected inspector, content browser | Unreal Editor loop → reflection-generated, agent-first |

## Non-negotiable principles (inherited → [CLAUDE.md](../../../CLAUDE.md))
1. **Engine is a library, not a framework you live inside.** Compose crates + plugins; ship only what's used. No monolith ([core](core/README.md)).
2. **One core runs headless or headful.** The same `App`/ECS/schedule drives CI, agents, and the rendered client — rendering is a plugin you add, not a dependency you can't remove ([core](core/README.md), [client](../client/README.md)).
3. **Content is data, engine is compiled.** New material/prefab/animation is a reflected asset, no recompile — the same rule the server holds ([scene](scene/README.md), [content-scripting](../game-server/content-scripting/README.md)).
4. **Open formats only.** glTF 2.0 / PNG16-EXR / KTX2 / Ogg-Opus / zstd. No proprietary tool in the pipeline ([assets](assets/README.md), [world-and-assets](../../architecture/06-world-and-assets.md)).
5. **Determinism where it's shared.** Engine sim logic reused by the server is bit-deterministic; rendering may be non-deterministic, sim may not ([core](core/README.md), [prediction-core](../client/prediction-core/README.md)).
6. **Every subsystem is agent-introspectable.** If a tool or an LLM can't enumerate and edit it via reflection, it's not done ([ai-native-dx](ai-native-dx/README.md)).

## Honest gaps vs Unreal (we document, we don't pretend)
Bevy trails Unreal in specific, known places. We ship the lean alternative and record the gap — same honesty as the [client note](../../initial-idea/03-tech-stack.md#client). Detail per doc; summary:

| Area | Unreal | Ours (Bevy-based) | Our stance |
|---|---|---|---|
| Virtual geometry | Nanite (production) | Bevy `meshlet` (experimental, compute-only) | Adopt for hero assets; LOD chains elsewhere ([rendering](rendering/README.md)) |
| Real-time GI | Lumen (SW+HW, any GPU) | Solari (HW-RT, NVIDIA-only today) + baked probes | Baked irradiance volumes as default; Solari opt-in ([rendering](rendering/README.md)) |
| Shadows | Virtual Shadow Maps | CSM + contact shadows | Ship CSM; track VSM upstream ([rendering](rendering/README.md)) |
| Upscaling | TSR (cross-vendor) + DLSS/FSR/XeSS | DLSS only; MSAA/SMAA/TAA/CAS otherwise | Native-desktop DLSS; spatial AA elsewhere ([rendering](rendering/README.md)) |
| World streaming / HLOD | World Partition + HLOD | none built-in | We build cell streaming + proxy LOD at app layer ([assets](assets/README.md), [world-model](../game-server/world-model/README.md)) |
| Editor | Full GUI, mature | none first-party | We build a **reflection-driven, MCP-first** editor ([editor](editor/README.md)) |

The trade we accept: **web/WebGPU can't run the high-end path** (no bindless/RT/mesh shaders in browsers) — the browser client gets the baseline renderer; native desktop gets the AAA path ([platform](../client/platform/README.md)).

## Reference index
**Internal:** [architecture/](../../architecture/README.md) · [initial-idea/](../../initial-idea/README.md) · [game-server specs](../game-server/README.md) · [client specs](../client/README.md) · crates under [`crates/`](../../../crates) · [gold standards](../../../CLAUDE.md#standards).
**External** (per-spec citations inline): [Bevy engine](https://bevy.org) & [release notes](https://bevy.org/news/) · [wgpu](https://github.com/gfx-rs/wgpu) · Unreal [Nanite](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine)/[Lumen](https://dev.epicgames.com/documentation/unreal-engine/lumen-technical-details-in-unreal-engine)/[VSM](https://dev.epicgames.com/documentation/unreal-engine/virtual-shadow-maps-in-unreal-engine)/[World Partition](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition-in-unreal-engine) · [jms55 rendering blog](https://jms55.github.io/) · [Rapier](https://rapier.rs) · [Unreal 5.8 MCP](https://byteiota.com/unreal-engine-5-8-ships-mcp-server-ai-agents-can-now-drive-the-editor/).

> Each subsystem doc is ≤ ~1 screen: what it does, the design, the distilled lesson, honest gaps, the rules, and links. Grows past that → split it. Same SRP rule as code.
