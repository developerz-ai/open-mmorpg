# 20 — Game Engine · Wave 1→2

> **Big idea.** The **reusable, Unreal-class runtime** both the client and server stand on — and that other games can consume standalone. Bevy + wgpu + a lean set of AAA techniques, wrapped so **introspectable and headless-testable that an AI agent + a solo dev out-ship a studio.** Engine is a **library of crates**, not a monolith. **Owner: [`gameengine`](../../../.claude/agents/gameengine.md).** → [game-engine specs](../../specs/game-engine/README.md)

## Reads
[game-engine README](../../specs/game-engine/README.md) and every subsystem: [core](../../specs/game-engine/core/README.md) · [rendering](../../specs/game-engine/rendering/README.md) · [assets](../../specs/game-engine/assets/README.md) · [animation](../../specs/game-engine/animation/README.md) · [scene](../../specs/game-engine/scene/README.md) · [physics](../../specs/game-engine/physics/README.md) · [audio](../../specs/game-engine/audio/README.md) · [ui](../../specs/game-engine/ui/README.md) · [ai-native-dx](../../specs/game-engine/ai-native-dx/README.md) · [editor](../../specs/game-engine/editor/README.md) · [06 world-and-assets](../../architecture/06-world-and-assets.md).

## Depends on
[Foundation](00-foundation.md): `ecs-core` shell (shared components), workspace. **Render/scene batches wait on nothing external** — engine is the substrate.

## PR batches (~100 PRs)
| Batch | Scope | Spec | Definition of Done |
|---|---|---|---|
| **E1 · Core (headless)** | App/Plugin/schedule wrappers, `bevy_reflect` type registry (mandatory), fixed-timestep, **runs headless from the same core**. | [core](../../specs/game-engine/core/README.md) | The whole engine boots headless in CI, no GPU; every gameplay type is registered. |
| **E2 · Scene/prefab** | Reflection-driven RON scene (de)serialization, prefab compose, deterministic load. | [scene](../../specs/game-engine/scene/README.md) | Spawn-from-data; agent authors a prefab and it loads & validates loud. |
| **E3 · Assets pipeline** | glTF import, asset server, **hot reload** (file-watch), KTX2 streaming, LOD/imposters. Open formats only. | [assets](../../specs/game-engine/assets/README.md) | glTF→scene→render; edit asset → hot-reload in ms. |
| **E4 · Rendering (baseline)** | Clustered forward+ PBR, CSM shadows, baked GI probes, spatial AA. Honest gaps vs Unreal documented. | [rendering](../../specs/game-engine/rendering/README.md) | Slice zone renders on native + WebGPU baseline path. |
| **E5 · Rendering (AAA, native)** | `meshlet` virtual geometry for hero assets, Solari opt-in, DLSS on native desktop. | [rendering](../../specs/game-engine/rendering/README.md) | Hero asset path on native; web falls back to E4 cleanly. |
| **E6 · Animation** | Skeletal + GPU skinning, blend/state-machine graph, IK, crowd VAT. | [animation](../../specs/game-engine/animation/README.md) | Character animates from glTF skin; deterministic where shared. |
| **E7 · Physics** | Collision, kinematic character controller, spatial/LOS queries (Rapier/Avian). **Sim-shared logic bit-deterministic.** | [physics](../../specs/game-engine/physics/README.md) | Controller moves; LOS query reused by server world-model. |
| **E8 · Audio** | Spatial audio, Opus/Ogg, mixer, lean all-Rust stack. | [audio](../../specs/game-engine/audio/README.md) | 3D emitters + listener; AoI-scoped mixing hook. |
| **E9 · UI substrate** | Retained HUD UI + immediate-mode tools UI, i18n substrate. | [ui](../../specs/game-engine/ui/README.md) | HUD widgets + a tools panel; strings via i18n keys. |
| **E10 · AI-native DX + MCP editor** | Reflection introspection over MCP, headless drive+verify, hot-reload loop; data-driven editor (outliner, reflected inspector, content browser). | [ai-native-dx](../../specs/game-engine/ai-native-dx/README.md) · [editor](../../specs/game-engine/editor/README.md) | [`apps/mcp`](../../../apps/mcp) lists/edits entities via reflection; anything the editor does, an agent does headless. |

## Interfaces this track owns
- **Engine crates (`bevy_*` + our plugins)** — consumed by [client](40-game-client.md) (headful) and shared with [server](30-game-server.md) (headless sim logic via `ecs-core`).
- **Scene/prefab + asset formats** — consumed by [content](60-content-and-assets.md).
- **MCP editor surface** ([`apps/mcp`](../../../apps/mcp)) — the agent's authoring/verify surface, = the editor surface.

## Rules
- **Engine is a library, not a framework you live inside** — compose crates + plugins, ship only what's used.
- **One core runs headless or headful** — rendering is a plugin you *add*, never a dependency you can't remove.
- **Reflection is not optional** — a gameplay type not in the registry is invisible to agents; that's a bug.
- **Determinism where shared** — sim logic reused by the server is bit-deterministic; rendering may not be.
- **Document honest gaps vs Unreal** — ship the lean alternative, record the trade ([rendering](../../specs/game-engine/rendering/README.md)).

## Definition of Done (track)
The engine boots headless in CI and headful on native + WebGPU baseline; an agent enumerates and edits any component via MCP reflection; a glTF asset hot-reloads; the slice zone renders; sim-shared logic is deterministic. Reusable by a *different* game with no core fork.

## Links
[game-engine specs](../../specs/game-engine/README.md) · [client track](40-game-client.md) · [server track](30-game-server.md) · [content track](60-content-and-assets.md) · [`gameengine` subagent](../../../.claude/agents/gameengine.md)
