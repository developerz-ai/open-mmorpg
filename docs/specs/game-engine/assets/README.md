# Assets

> The pipeline from **open-format files → GPU-ready, streamable, hot-reloadable** engine data. glTF in, KTX2/meshlet out, no proprietary converter anywhere. Built for an **AI asset firehose**: generated content only has to *fit the slot*. → [engine README](../README.md) · [world-and-assets](../../../architecture/06-world-and-assets.md)

## What it does
Loads and hot-reloads assets via the async **asset server**: meshes/materials/animations from **glTF 2.0/glB**, textures from **KTX2/DDS/Basis**, terrain from **PNG16/EXR heightfields**, audio from **Ogg/Opus** — all zstd-bundled with a `manifest.json` ([formats](../../../architecture/06-world-and-assets.md)). Enabling the file-watcher makes the server live-reload on disk change — the fast iterate-observe loop agents and artists both need. → [Bevy asset system](https://bevy-cheatbook.github.io/assets/hot-reload.html), [KTX2/Basis support](https://github.com/bevyengine/bevy/pull/3884).

## Design
- **glTF is the mesh/material/anim source of truth.** Metallic-roughness PBR maps straight to `StandardMaterial` ([rendering](../rendering/README.md)); skins/animations to `AnimationClip` ([animation](../animation/README.md)). Any Blender/Houdini/AI-gen tool that emits glTF drops in with **zero conversion** — the open-format unlock ([legal](../../../initial-idea/01-legal-and-licensing.md)).
- **Textures ship as KTX2 + zstd supercompression**, transcoded at load to the GPU's native block format (BCn/ASTC/ETC2). Basis Universal for "one file, all GPUs." Small on disk, GPU-native in memory. A 200 MB uncompressed glTF is a bug ([world-and-assets](../../../architecture/06-world-and-assets.md)).
- **Manifest-driven asset slots.** Each race/class/item declares a slot spec (model + texture set + rig). AI-generated content is validated against the slot spec → zstd bundle → dropped into `assets/` + manifest. No bespoke importer per asset ([AI pipeline](../../../architecture/06-world-and-assets.md)).
- **Content-addressed, out of git.** Large binaries live in object storage (R2-style); the tree stays lean. glTF/EXR/PNG/Ogg are git-ignored by rule.

## LOD, imposters, streaming (the MMO-scale gap we fill)
Bevy has no built-in LOD/imposter/HLOD or camera-driven mesh streaming — we build these at the app layer:
- **Discrete LOD chains** — decimated mesh tiers swapped by screen size; the far tier is an **octahedral imposter** (single quad, atlas of pre-rendered views) for trees/rocks. → [octahedral imposters](https://shaderbits.com/blog/octahedral-impostors).
- **HLOD proxies** — clusters of distant static meshes merged into one proxy mesh+material to collapse draw calls, the [Unreal HLOD](https://docs.unrealengine.com/4.27/en-US/BuildingWorlds/HLOD/HowTo/BuildingHLODs) concept, generated offline from our glTF.
- **Camera-position streaming** — tiles = glTF + heightmap chunks loaded/unloaded by the [quadtree world model](../../game-server/world-model/README.md), under an explicit memory budget. This is the client half of the same partition the server uses for AoI — no loading screens ([world streaming](../../../architecture/06-world-and-assets.md)).

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| Bevy asset server + hot reload | Async load + file-watch live reload is the iterate loop | **Adopt** wholesale; it powers agent + artist iteration |
| Bevy KTX2/Basis | GPU-compressed, transcode-at-load is the right on-disk format | **Adopt** KTX2+zstd as the shipping texture format |
| Unreal OFPA / World Partition | One-file-per-actor + cell streaming enables seamless worlds + parallel edits | **Adopt the concept**: cell-granular streamed assets, built app-side (Bevy lacks it) |
| Unreal HLOD / imposters | Distant clusters → proxy/imposter to kill draw calls | **Build** offline proxy + imposter bake from our glTF |

## Rules
- **Open formats only** — glTF/EXR/PNG16/KTX2/Ogg/zstd. No proprietary tool in the chain, ever ([legal](../../../initial-idea/01-legal-and-licensing.md)).
- **Every asset traces to an original generation or authored source.** Never an extracted game asset ([legal](../../../initial-idea/01-legal-and-licensing.md)).
- **Optimize before commit** — compressed textures, mesh LODs, imposters for far tiers. Large binaries → object storage, not git.
- Asset load is async and fail-loud — a missing/invalid asset is a visible error, not a silent blank ([ui](../ui/README.md)).

## Links
[rendering](../rendering/README.md) · [animation](../animation/README.md) · [scene](../scene/README.md) · [world-model](../../game-server/world-model/README.md) · [world-and-assets](../../../architecture/06-world-and-assets.md)
