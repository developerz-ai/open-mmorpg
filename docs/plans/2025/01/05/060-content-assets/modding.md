# Modding - Creating Custom Datapacks

## Overview

Open-MMORPG supports total-conversion modding through datapacks. A datapack is a self-contained content bundle that can add or replace game content without requiring any code changes.

## Datapack Structure

A datapack is a directory with:

```
my-datapack/
├── manifest.json           # Required: datapack metadata
├── factions/               # Optional: factions
├── races/                  # Optional: races
├── classes/                # Optional: classes
├── abilities/              # Optional: abilities
├── items/                  # Optional: items
├── quests/                 # Optional: quests
├── zones/                  # Optional: zones
├── dungeons/               # Optional: dungeons
└── assets/                 # Optional: custom assets
    ├── manifest.json
    ├── meshes/
    ├── textures/
    └── audio/
```

## Creating a Datapack

### 1. Create Directory Structure

```bash
mkdir -p my-datapack/{factions,races,classes,abilities,items,quests,zones,dungeons}
```

### 2. Create Manifest

**File**: `my-datapack/manifest.json`

```json
{
  "id": "my-mod",
  "version": "1.0.0",
  "api_version": 1,
  "factions": [
    {"$ref": "factions/my-faction.json"}
  ],
  "races": [],
  "classes": [],
  "abilities": [],
  "items": [],
  "quests": [],
  "zones": [],
  "spawn_tables": [],
  "dungeons": [],
  "economy": {
    "auction_houses": [],
    "trading_rules": [],
    "starting_gold_copper": 0
  }
}
```

**Important**:
- `id` must be unique (reverse domain notation recommended: `com.author.modname`)
- `version` follows semantic versioning
- `api_version` must match the core game version

### 3. Add Content

Create content files following the authoring guide (see `authoring-guide.md`).

**Example**: `my-datapack/factions/my-faction.json`

```json
{
  "id": "my-faction",
  "name": "My Custom Faction",
  "description": "A faction added by my mod",
  "colors": {
    "primary": "#ff0000",
    "secondary": "#800000"
  },
  "capital": "my-zone",
  "hostile_to": []
}
```

### 4. Add Custom Assets (Optional)

**File**: `my-datapack/assets/manifest.json`

```json
{
  "id": "my-mod.assets",
  "version": "1.0.0",
  "api_version": 1,
  "meshes": [
    "custom-weapon.gltf"
  ],
  "textures": [
    "custom-texture.png"
  ],
  "audio": [
    "custom-sound.ogg"
  ]
}
```

Place asset files in appropriate directories:
- `assets/meshes/` - glTF models
- `assets/textures/` - PNG/KTX2 textures
- `assets/audio/` - Opus OGG audio

### 5. Validate

```bash
# Validate JSON syntax
find my-datapack -name '*.json' -exec echo "Checking {}" \; -exec json.tool {} \;

# Optional: validate against schema if tools are available
```

## Distribution

### Packaging

Create a compressed archive:

```bash
# Create tar.gz
tar -czf my-mod-1.0.0.tar.gz my-datapack/

# Or zip
zip -r my-mod-1.0.0.zip my-datapack/
```

### Metadata File (Optional)

Include a `README.md` with:

```markdown
# My Mod

A custom content pack for Open-MMORPG.

## Installation
1. Download `my-mod-1.0.0.tar.gz`
2. Extract to `datapacks/` directory
3. Enable in game settings

## Compatibility
- Requires Open-MMORPG 0.1.0+
- API version 1

## Content
- 1 new faction
- 2 new races
- 5 new items
- 3 new quests
```

## Installation

Players install datapacks by:

1. **Download** the datapack archive
2. **Extract** to the game's `datapacks/` directory
3. **Enable** in game settings or launcher
4. **Restart** the game to load content

## Loading Priority

When multiple datapacks are loaded:

1. **Base game** content loads first
2. **Datapacks** load in enable order
3. **Later datapacks** override earlier ones (same ID = replacement)

This allows mods to override base content intentionally.

## Best Practices

### 1. ID Namespacing

Use unique IDs to avoid conflicts:

```json
{
  "id": "mymod_my-faction",  // Prefix with mod ID
  "name": "My Faction"
}
```

### 2. Incremental Content

Don't replace base content unless necessary:

```json
{
  // Good: Add new content
  "factions": [
    {"$ref": "factions/my-faction.json"}
  ]

  // Avoid: Replace base content
  // "factions": [
  //   {"$ref": "factions/dawnward.json"}  // Overrides base!
  // ]
}
```

### 3. Asset References

Reference assets with full paths:

```json
{
  "mesh": "datapacks/my-mod/assets/meshes/custom.gltf"
}
```

### 4. Dependencies

Document required base content in README:

```markdown
## Dependencies
- Requires base game zones: "starting-zone", "ember-city"
- Compatible with mods: "other-mod-1.0.0"
```

### 5. Version Compatibility

Only support one API version:

```json
{
  "api_version": 1  // Pin to specific version
}
```

## Conflict Resolution

### ID Conflicts

When two mods define the same ID:

1. **Later mod wins** (last enabled takes precedence)
2. **Log warning** on console
3. **Players** can reorder mods to resolve

### Asset Conflicts

When two mods provide assets with same path:

1. **Base game assets** have lowest priority
2. **Mod assets** override base game
3. **Later mods** override earlier mods

### Cross-Reference Conflicts

When content references missing IDs:

1. **Validation fails** on load
2. **Error logged** to console
3. **Mod disabled** or load aborted

## Testing Your Mod

### 1. Local Testing

```bash
# Copy to datapacks directory
cp -r my-datapack ~/.config/open-mmorpg/datapacks/

# Run game
open-mmorpg

# Check console for errors
```

### 2. Validation

```bash
# Check JSON syntax
find my-datapack -name '*.json' | xargs -I {} json.tool {}

# Check for missing references
# (Requires game to provide validation tool)
```

### 3. Compatibility Testing

Test with:
- Base game only
- Base game + popular mods
- Multiple mods together

## Distribution Channels

### Recommended

- **ModDB** - Large gaming mod community
- **GitHub Releases** - Direct distribution
- **CurseForge** - Game-specific mod hosting
- **Nexus Mods** - Popular mod platform

### Include in Release

- `my-mod-1.0.0.tar.gz` - Main archive
- `README.md` - Documentation
- `CHANGELOG.md` - Version history
- `LICENSE` - License file

## Legal Considerations

### Original IP Requirement

Open-MMORPG requires original IP only:

- ❌ **Don't** rip assets from other games
- ❌ **Don't** use trademarked faction names
- ❌ **Don't** copy quest chains from other MMOs
- ✅ **Do** create original factions
- ✅ **Do** use original names
- ✅ **Do** design original quests

### Licensing

Choose an appropriate license:

- **MIT** - Permissive, allows anything
- **CC-BY-SA** - Requires attribution, share-alike
- **Proprietary** - All rights reserved

Document license in `LICENSE` file.

## Example: Complete Mini-Mod

```
example-mod/
├── README.md
├── LICENSE
├── manifest.json
├── factions/
│   └── shadow-faction.json
├── races/
│   └── shadowborn.json
└── items/
    └── shadow-dagger.json
```

**manifest.json**:
```json
{
  "id": "com.example.shadow-mod",
  "version": "1.0.0",
  "api_version": 1,
  "factions": [{"$ref": "factions/shadow-faction.json"}],
  "races": [{"$ref": "races/shadowborn.json"}],
  "classes": [],
  "abilities": [],
  "items": [{"$ref": "items/shadow-dagger.json"}],
  "quests": [],
  "zones": [],
  "spawn_tables": [],
  "dungeons": [],
  "economy": {"auction_houses": [], "trading_rules": [], "starting_gold_copper": 0}
}
```

## Troubleshooting

### Mod Won't Load

1. Check `api_version` matches game
2. Validate all JSON files
3. Check for missing cross-references
4. Review game console for errors

### Assets Not Showing

1. Check asset manifest paths
2. Verify asset files exist
3. Check file formats (glTF, PNG, Opus OGG)
4. Test with base game assets

### Content Missing In-Game

1. Verify manifest references are correct
2. Check IDs are unique
3. Ensure no cross-reference errors
4. Try reordering mod load order
