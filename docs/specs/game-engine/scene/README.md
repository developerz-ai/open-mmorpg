# Scene & Prefabs

> A **reflection-driven** scene/prefab format: entities + components serialized to a text format an artist, an editor, or an **AI agent** can read and write. Content is data; the engine is compiled. → [engine README](../README.md)

## What it does
Serializes a snapshot of the ECS world — entities and their reflected components — to a text scene (`.scn.ron`, moving toward the richer `.bsn`) via `bevy_reflect`. Only types **registered** in the type registry (de)serialize; that registry is the single contract shared by scene I/O, the [editor](../editor/README.md) inspector, and [agent tooling](../ai-native-dx/README.md). → [Bevy scenes](https://taintedcoders.com/bevy/scenes), [Bevy reflection](https://taintedcoders.com/bevy/reflection), [0.19 next-gen scenes](https://bevy.org/news/).

## Design
- **Prefabs = scenes as templates.** A prefab is a scene fragment instantiated many times (a mob, a prop, a UI panel). Composition via required components / relationships ([core](../core/README.md)), not deep inheritance.
- **Reflection is the format.** No hand-written serializers per type — `#[derive(Reflect)]` + registry entry is the whole contract. Add a field → it appears in scenes, the inspector, and to agents automatically. This is why an LLM can author a valid prefab: the schema is introspectable, and invalid data fails at load, loudly ([ai-native-dx](../ai-native-dx/README.md)).
- **Game content is scenes + data, never code.** A new faction camp, quest giver, or item is a scene/prefab + [content-schema](../../game-server/content-scripting/README.md) data — no `cargo build`. Same rule the server holds; the engine enforces the client half.
- **Deterministic instantiation.** Spawning a scene into the world is order-stable so a headless run reproduces a headful one ([core](../core/README.md), [prediction-core](../../client/prediction-core/README.md)).

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| Bevy `.scn.ron` / `.bsn` | Reflection-serialized entities as a text, diffable, hot-reloadable format | **Adopt**; text scenes are git-diffable and agent-editable |
| Unity prefabs / ScriptableObjects | Data assets separate content from behavior; designers tune without touching code | **Adopt the split**: behavior compiled, data authored |
| Unreal Data Assets + `UPROPERTY` reflection | Reflection auto-generates the editor UI for data | **Adopt**: our inspector is generated from `bevy_reflect`, not hand-built ([editor](../editor/README.md)) |

## Rules
- **Register every gameplay component in the type registry.** Unregistered = not serializable, not inspectable, not agent-visible — a silent hole ([core](../core/README.md)).
- **Content changes are scene/data edits, not recompiles.** If adding content needs `cargo build`, the split is wrong ([CLAUDE.md](../../../../CLAUDE.md)).
- Scenes are text and diffable; large binary payloads belong in [assets](../assets/README.md), referenced by handle, not inlined.
- Invalid scene data fails loud at load ([ui](../ui/README.md)) — never a silent partial spawn.

## Links
[core](../core/README.md) · [assets](../assets/README.md) · [editor](../editor/README.md) · [ai-native-dx](../ai-native-dx/README.md) · [content-scripting](../../game-server/content-scripting/README.md)
