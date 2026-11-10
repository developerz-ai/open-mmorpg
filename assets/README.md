# Assets Directory

This directory currently contains only this README and an asset manifest.
No mesh, texture or audio payload files ship here.

## Placeholder Structure

The committed inventory is:

```text
assets/
├── README.md
└── manifest.json
```

## Current Placeholders

[`manifest.json`](manifest.json) lists placeholder filenames, not shipped assets:

- **Meshes**: `placeholder-cube.gltf`, `placeholder-sphere.gltf`, `placeholder-plane.gltf`
- **Textures**: `checker-512.png`, `checker-1024.png`
- **Audio**: `silent-1s.ogg`

None of these payload files is present. The manifest's version field does not
establish a working asset loader or cache-invalidation workflow.

## Future AI Asset Pipeline

The automated generation pipeline is not yet implemented (planned: [K7](../docs/mvp/v1/60-content-and-assets.md#pr-batches-100-prs)):

1. **Meshes**: Generate original 3D models and PBR maps from text prompts via Meshy.ai
2. **Textures**: Generate additional textures as needed
3. **Audio**: Generate sound effects and ambient audio with appropriate audio tooling

Assets will be generated in open formats:
- glTF for 3D models
- PNG/KTX2 for textures
- Opus OGG for audio

The intended pipeline normalizes these open formats, bundles them with zstd,
then imports and verifies them in the engine. **Original IP only** — never
extract or reuse another game's assets, names or tables.

## Hot-Swap Plan

Asset integration and hot-swapping are not yet implemented (planned: [K8](../docs/mvp/v1/60-content-and-assets.md#pr-batches-100-prs)). The intended workflow is:
- Edit asset → reload in game (no restart needed)
- Link assets and content IDs for automatic loading
- Use manifest versions for cache invalidation

## Directory Structure

Planned layout, not the current inventory; the directories below do not yet exist:

```text
assets/
├── README.md
├── manifest.json           # Intended asset/version catalog
├── meshes/                 # 3D models (glTF)
├── textures/               # Textures (PNG/KTX2)
│   └── solid-colors/       # Solid color textures
└── audio/                  # Audio files (Opus OGG)
```

## Adding New Assets

Planned workflow once the generation and integration pipeline exists; adding a
manifest entry today does not make an asset load or hot-reload:

1. Place asset file in appropriate directory
2. Update `assets/manifest.json` to include the new asset
3. Reference asset ID from content definitions (items, abilities, etc.)
4. Reload game to pick up changes
