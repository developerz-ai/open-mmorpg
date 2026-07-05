# Assets Directory

This directory contains game assets (3D models, textures, audio).

## Placeholder Structure

For initial development, this directory contains placeholder assets that will be replaced with AI-generated assets via Meshy.ai integration in future work.

## Current Placeholders

- **Meshes**: Simple geometric shapes (cube, sphere, plane) in glTF format
- **Textures**: Checkerboard patterns for testing UV mapping
- **Audio**: Silent 1-second OGG files

## Future AI Asset Pipeline

The plan is to integrate with Meshy.ai for automated asset generation:

1. **Meshes**: Generate 3D models from text prompts
2. **Textures**: Generate textures from text prompts
3. **Audio**: Generate sound effects and ambient audio

Assets will be generated in open formats:
- glTF for 3D models
- PNG/KTX2 for textures
- Opus OGG for audio

## Hot-Swap Plan

Assets support hot-swapping during development:
- Edit asset → reload in game (no restart needed)
- Assets reference content IDs for automatic loading
- Asset manifest tracks version for cache invalidation

## Directory Structure

```
assets/
├── manifest.json           # Asset manifest (tracks versions)
├── meshes/                 # 3D models (glTF)
├── textures/               # Textures (PNG/KTX2)
│   └── solid-colors/       # Solid color textures
└── audio/                  # Audio files (Opus OGG)
```

## Adding New Assets

1. Place asset file in appropriate directory
2. Update `assets/manifest.json` to include the new asset
3. Reference asset ID from content definitions (items, abilities, etc.)
4. Reload game to pick up changes
