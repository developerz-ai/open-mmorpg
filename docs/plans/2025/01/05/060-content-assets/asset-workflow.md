# Asset Workflow - Hot-Swap and Loading

## Overview

Assets support hot-swapping during development. Edit an asset file and reload in-game without restarting the server or client.

## Asset Loading Flow

```
1. Load manifest.json → asset list
2. For each asset:
   a. Check cache by path + modified time
   b. If cached and unchanged → use cached
   c. If uncached or changed → load from disk
3. Asset available for content references
```

## Hot-Swap Development

### Workflow

1. **Edit asset** in `assets/` directory
2. **Save** file
3. **Reload** in game (console command or keybind)
4. **Asset updates** immediately

### Supported Operations

- **Texture edits**: Update → reload → see changes
- **Model edits**: Update glTF → reload → new geometry
- **Audio edits**: Update OGG → reload → new sound
- **New assets**: Add file → update manifest → reload

### Reload Commands

In-game console:

```
reload_assets              // Reload all assets
reload_texture <path>     // Reload specific texture
reload_mesh <path>        // Reload specific mesh
reload_audio <path>       // Reload specific audio
```

## Asset Manifest

**File**: `assets/manifest.json`

```json
{
  "id": "open-mmorpg.assets",
  "version": "0.0.0",
  "api_version": 1,
  "meshes": [
    "weapons/sword-iron.gltf",
    "armor/plate-chest.gltf"
  ],
  "textures": [
    "weapons/sword-iron.png",
    "armor/plate-chest.png"
  ],
  "audio": [
    "weapons/sword-swing.ogg",
    "footsteps/grass.ogg"
  ]
}
```

**Purpose**:
- Tracks all asset files
- Enables bulk reloading
- Provides asset validation

## Asset Caching

### Cache Key

```
cache_key = asset_path + ":" + file_mtime + ":" + file_size
```

### Cache Behavior

- **First load**: Read from disk, cache in memory
- **Subsequent loads**: Check cache key
  - Same key → use cached
  - Different key → reload from disk

### Cache Invalidation

Automatic invalidation on:
- File modification time change
- File size change
- Manual `reload_assets` command

## Asset Formats

### Meshes (glTF)

**Format**: glTF 2.0 (.gltf or .glb)

**Requirements**:
- Triangulated geometry
- Y-up orientation
- PBR material workflow
- Reasonable poly count (<100K per mesh)

**Tools**:
- Blender → export as glTF 2.0
- Meshy.ai → generate, export as glTF

**Example**: `weapons/sword-iron.gltf`

```json
{
  "asset": {"version": "2.0"},
  "scenes": [{"nodes": [0]}],
  "nodes": [{"name": "Sword", "mesh": 0}],
  "meshes": [{
    "primitives": [{
      "attributes": {"POSITION": 0},
      "indices": 0
    }]
  }],
  ...
}
```

### Textures (PNG/KTX2)

**Format**: PNG (lossless) or KTX2 (compressed)

**Requirements**:
- Power-of-two dimensions (1024x1024, 2048x2048)
- Reasonable file size (<2MB per texture)
- Appropriate color space (sRGB for albedo, linear for others)

**Tools**:
- GIMP → export as PNG
- Texture Compressor → convert to KTX2

**Example**: `weapons/sword-iron.png`

### Audio (Opus OGG)

**Format**: Opus codec in OGG container

**Requirements**:
- Opus encoding (48kHz recommended)
- Reasonable file size (<500KB per sfx)
- Appropriate length (<10s for sfx, loop for ambient)

**Tools**:
- Audacity → export as Opus OGG
- ffmpeg → `ffmpeg -i input.wav -c:a libopus output.ogg`

**Example**: `weapons/sword-swing.ogg`

## Asset References

### From Items

**File**: `content/items/weapons/sword-iron.json`

```json
{
  "id": "sword-iron",
  "name": "Iron Sword",
  "mesh": "weapons/sword-iron.gltf",
  "icon": "weapons/sword-iron.png"
}
```

### From Abilities

**File**: `content/abilities/slash.json`

```json
{
  "id": "slash",
  "name": "Slash",
  "icon": "abilities/slash.png",
  "sounds": {
    "cast": "weapons/sword-swing.ogg",
    "impact": "weapons/sword-impact.ogg"
  }
}
```

### From Zones

**File**: `content/zones/forest/zone.json`

```json
{
  "id": "forest",
  "ambient_audio": "ambient/forest-wind.ogg",
  "skybox": "skyboxes/forest-sky.gltf"
}
```

## Asset Pipeline

### Development

1. **Create/Edit** asset file
2. **Test** in-game with hot-swap
3. **Iterate** until satisfied
4. **Commit** to repository

### Production

1. **Optimize** assets (compress textures, audio)
2. **Validate** formats and sizes
3. **Update** asset manifest
4. **Deploy** with content update

## Asset Optimization

### Textures

```bash
# Optimize PNG with pngcrush
pngcrush -ow texture.png

# Convert to KTX2 with Basis Universal
basisu -ktx2 -qpus texture.png texture.ktx2
```

### Audio

```bash
# Re-encode Opus at lower bitrate (128kbps → 64kbps)
ffmpeg -i input.ogg -c:a libopus -b:a 64k output.ogg
```

### Meshes

```bash
# Use glTF-Pipeline for compression
gltf-pipeline -i model.gltf -o model-draco.gltf -d
```

## Missing Assets

### Behavior

When an asset reference fails to load:

1. **Log warning** to console
2. **Use fallback asset** (placeholder)
3. **Continue loading** (don't crash)

### Fallback Assets

- **Meshes**: `placeholder-cube.gltf`
- **Textures**: `textures/checker-512.png`
- **Audio**: Silent playback (no error)

### Example Error

```
[WARN] Failed to load asset "weapons/custom-sword.gltf": file not found
[FALLBACK] Using placeholder mesh instead
```

## Asset Bundles

For distribution, assets can be bundled:

```bash
# Create asset bundle
tar -czf assets-0.1.0.tar.gz assets/

# Or create per-zone bundles
tar -czf zone-forest-assets.tar.gz assets/meshes/forest/ assets/textures/forest/ assets/audio/forest/
```

**Bundle loading**:
- Extract to `assets/` directory
- Reload assets
- Content picks up new assets automatically

## Validation

### Asset Manifest Validation

```bash
# Validate all assets exist
find assets/ -name '*.gltf' -o -name '*.png' -o -name '*.ogg' | \
  while read f; do
    if [ ! -f "assets/$f" ]; then
      echo "Missing asset: $f"
    fi
  done

# Validate manifest references exist
bun scripts/validate-assets.ts
```

### Format Validation

```bash
# Validate glTF files
gltf-validator assets/meshes/*.gltf

# Validate PNG files
file assets/textures/*.png | grep -v "PNG image data"

# Validate Opus files
ogginfo assets/audio/*.ogg
```

## Performance Considerations

### Load Times

- **Small assets** (<1MB): Load in <10ms
- **Medium assets** (1-10MB): Load in 10-100ms
- **Large assets** (>10MB): Load in >100ms

**Optimization**:
- Compress textures (KTX2 with Basis)
- Compress audio (lower Opus bitrate)
- Use glTF DRACO compression for meshes

### Memory Usage

- **Textures**: ~1MB per 1024x1024 uncompressed
- **Meshes**: ~100KB per 10K triangles
- **Audio**: ~100KB per second at 128kbps

**Optimization**:
- Use texture streaming for large worlds
- Use mesh LODs for distant objects
- Stream audio rather than load entirely

## Troubleshooting

### Asset Won't Reload

1. **Check file saved** (some editors delay save)
2. **Check file permissions** (readable by game)
3. **Check manifest updated** (for new assets)
4. **Check file format** (valid glTF/PNG/Opus)

### Asset Appears Garbled

1. **Validate file format** (use gltf-validator)
2. **Check color space** (sRGB vs linear)
3. **Check coordinate system** (Y-up vs Z-up)
4. **Check compression settings** (DRACO requires decoder)

### Audio Won't Play

1. **Validate Opus format** (ogginfo)
2. **Check sample rate** (48kHz recommended)
3. **Check file size** (not empty)
4. **Check audio system** (device available)

## Future Work

### AI Asset Generation

Integration with Meshy.ai for automated asset generation:

```
Text prompt → Meshy.ai → glTF model → assets/
Text prompt → Meshy.ai → texture PNG → assets/
Text prompt → Meshy.ai → audio OGG → assets/
```

### Asset Streaming

Dynamic loading based on player position:

```
Player moves → Load nearby assets → Unload distant assets
```

### Asset Versioning

Track asset versions for cache invalidation:

```
Asset manifest includes version → Cache checks version → Reload if changed
```
