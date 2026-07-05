# 05 — Asset & Map Formats (replaces MPQ/WMO/ADT)

## Why replace them
MPQ/WMO/ADT are closed, 2004-era, DCC-hostile formats needing reverse-engineered tools (Noggit, WMV). We want **standard, tool-compatible, AI-pipeline-compatible** formats from day 1. This is the actual "AI-first content" unlock.

## Format choices
| Replaces | We use | Why |
|---|---|---|
| MPQ (container) | **zip/tar + content-addressed** (Git-LFS/IPFS-style hash) **or** `manifest.json` + dir, **zstd**-compressed | zstd beats MPQ-era compression on speed+ratio, zero custom code |
| WMO (static meshes/objects) | **glTF 2.0 / glB** | true open standard; native Blender/Maya export; default output of AI 3D-gen pipelines |
| ADT (terrain) | **PNG16/EXR heightfields + splat maps** | standard in Unreal/Unity/Godot/Bevy; huge tooling support |
| Chunk streaming | **Quadtree zone streaming** of glTF + heightmap tiles | same effect as WMO/ADT chunk-loading, on open formats |

## Why this wins
- Any operator uses off-the-shelf Blender/Godot/Bevy tooling — **no custom format tooling required**.
- **AI asset-gen pipelines already output glTF** — assets drop in with zero conversion. This is the real "AI-first" lever.
