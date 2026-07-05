# Animation

> Skeletal animation from glTF, **GPU-skinned**, driven by a blend/state-machine **graph**, scaling to MMO crowds via **vertex-animation textures**. Hero characters get full skeletal fidelity; a city full of NPCs gets baked-instanced cheapness. → [engine README](../README.md)

## What it does
Deforms skinned meshes: glTF **skins + animation channels** load as `AnimationClip`s; linear-blend skinning runs on the **GPU** (vertex shader). A per-character **`AnimationGraph`** blends clips — weighted blends, additive layers (swing over walk), and masks (upper/lower body independent) — plus animation events. → [Bevy animation graph (0.15)](https://bevy.org/news/bevy-0-15/), [`bevy::animation`](https://docs.rs/bevy/latest/bevy/animation/index.html).

## Design
- **GPU skinning by default.** Per-vertex joint blending in the vertex shader; no CPU skinning on the hot path. GPU-driven skinned draw (Bevy 0.16+) keeps CPU cost flat as character count rises ([rendering](../rendering/README.md)).
- **Graph-driven, data-authored.** Locomotion is a blend space / state machine, authored as data, not code — a new emote or gait is a graph edit, not a recompile ([scene](../scene/README.md)). This is the [content-is-data](../../../../CLAUDE.md) rule applied to motion.
- **Crowd scale via VAT.** For hundreds of distant animated characters, bake per-frame vertex positions into a **vertex-animation texture**: the mesh becomes a static instance animated entirely in the shader, GPU-instanced, no per-character CPU skinning. Trade: fixed baked clips, no per-instance blend/IK — correct for background crowds, not for the local player. → [VAT technique](https://medium.com/tech-at-wildlife-studios/texture-animation-techniques-1daecb316657), [OpenVAT](https://openvat.org/).
- **Tiered by distance:** near = full skeletal + graph + IK; mid = skeletal, reduced bones; far = VAT instance. Driven by the same LOD/AoI budget as [assets](../assets/README.md).

## Honest gaps vs Unreal
Bevy core has **no state machines, blend trees, IK, or root motion** — Unreal's AnimGraph ships all of these. We fill via the community [`bevy_animation_graph`](https://github.com/mbrea-c/bevy_animation_graph) (FSM nodes, blend spaces, two-bone IK, visual editor) and [`bevy_mod_inverse_kinematics`](https://github.com/Kurble/bevy_mod_inverse_kinematics) (FABRIK), pinning versions and contributing upstream. **Root motion has no standard solution** — for a server-authoritative game, motion is driven by [predicted](../../client/prediction-core/README.md) transforms anyway, so root-motion authority stays server-side; the client animates *to* it. Where the engine is already strong: glTF skinned animation + GPU skinning + additive/masked blending are solid out of the box.

## Distilled from the references
| Source | Adopt |
|---|---|
| glTF skins/animations | Portable skeletal source; skin on GPU |
| Bevy animation graph | Weighted/additive/masked blending as the authoring primitive |
| Unreal AnimGraph | State machines + blend spaces + IK as the *target* feature set (via crates today) |
| VAT (Wildlife/OpenVAT) | Baked-texture instancing for MMO-scale crowds — the technique studios use for armies |

## Rules
- **Local player = full skeletal + graph.** Never VAT the entity the user controls; it needs per-frame blend/IK ([client rendering](../../client/rendering/README.md)).
- **Motion is cosmetic on the client.** Authoritative position comes from the server; animation interpolates toward it, never asserts it ([prediction-core](../../client/prediction-core/README.md), [security](../../game-server/security/README.md)).
- **Animation is data.** New clips/graphs ship as reflected assets, no recompile ([scene](../scene/README.md)).

## Links
[rendering](../rendering/README.md) · [assets](../assets/README.md) · [physics](../physics/README.md) · [client rendering](../../client/rendering/README.md) · [prediction-core](../../client/prediction-core/README.md)
