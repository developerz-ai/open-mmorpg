# 06 — World & Assets

> Open formats + AI generation = the "AI-first content" unlock. Any Blender/Bevy/Godot tool works; any AI 3D-gen pipeline drops in with zero conversion. → [../initial-idea/05-asset-and-map-formats.md](../initial-idea/05-asset-and-map-formats.md)

## Formats
| Kind | Format |
|---|---|
| Static meshes / objects | **glTF 2.0 / glB** |
| Terrain | **PNG16/EXR heightfields + splat maps** |
| Container/bundle | `manifest.json` + dir, **zstd**-compressed (or content-addressed) |
| Audio | Ogg/Opus |

## World streaming
- World partitioned by **quadtree**; tiles = glTF meshes + heightmap chunks.
- Client streams tiles on demand by camera position → **no loading screens** ([03](03-netcode-and-sharding.md)).
- Server uses the same partition for **interest management** and shard/zone boundaries.

## AI asset pipeline
- **Manifest-driven asset slots**: each race/class/item declares a slot spec (model + textures + anim rig). AI-generated content only has to *fit the slot*.
- Workflow: AI 3D-gen → glTF → validate against slot spec → zstd bundle → drop into `assets/` + manifest entry. No custom converter step.
- PBR textures (albedo/normal/roughness/metallic) for "best-looking graphics" on the wgpu/PBR renderer ([../initial-idea/03-tech-stack.md](../initial-idea/03-tech-stack.md)).
- Store large binaries via content-addressing / object storage (R2-style), not fat git — keep the source tree lean.

## Rules
- Every asset traces to an original AI generation or original authored source. **Never** an extracted game asset ([../initial-idea/01-legal-and-licensing.md](../initial-idea/01-legal-and-licensing.md)).
- Optimize before commit (compress textures, mesh LODs). A 200MB uncompressed glTF is a bug.
