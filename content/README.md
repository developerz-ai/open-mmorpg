# content/ — the data-driven game layer

**Core is compiled (`crates/`); content is data (here).** Factions, classes,
races, abilities, quests and zones are files, not code. Change one and the game
changes with **no `cargo build`**. If altering a faction needs a recompile, the
split is wrong.

This is how operators get the full thing to customize — add maps, retune rules,
ship a total-conversion datapack — while our compiled core stays strong. Every
bundle is described by [`manifest.json`](manifest.json), validated by the
`omm-content-schema` crate against the core content API version.

```
content/
├── manifest.json           # Root manifest (required)
├── factions/               # Faction definitions
├── races/                  # Race definitions
├── classes/                # Class definitions
├── abilities/              # Ability definitions
├── items/                  # Item definitions
│   ├── weapons/
│   ├── armor/
│   ├── consumables/
│   └── quest/
├── quests/                 # Quest definitions
├── zones/                  # Zone definitions
├── dungeons/               # Dungeon definitions
├── scripts/                # Script files (WASM/Lua, future)
└── economy/                # Economy (auction house, trading rules)

assets/
├── manifest.json           # Asset manifest
├── meshes/                 # 3D models (glTF)
├── textures/               # Textures (PNG/KTX2)
└── audio/                  # Audio (Opus OGG)
```

→ docs/architecture/05-ecs-and-scripting.md · docs/initial-idea/06-modding-and-extensibility.md

## Authoring Guidelines

### File Naming
- Use kebab-case: `aurelian-concord.json`, `sunken-crypts.json`
- One definition per file
- IDs must match filename (without extension)

### ID Stability
- IDs are stable references across the entire content system
- Never change an ID once published
- For replacements, deprecate old IDs and add new ones

### Cross-References
- All references are validated by `load_manifest()`
- Faction `hostile_to` arrays must reference valid faction IDs
- Race `faction_id` must reference a valid faction
- Class `abilities` arrays must reference valid ability IDs
- Quest `prerequisites` must reference valid quest IDs

## Validation Checklist

Before committing content changes, verify:

- [ ] `manifest.json` includes all new files in appropriate arrays
- [ ] All IDs are unique and match their filenames
- [ ] All cross-references resolve (no dangling IDs)
- [ ] `cargo nextest run` passes (content-schema tests)
- [ ] `bin/check` passes (full CI gate)

## Common Pitfalls

1. **Forgot to update manifest.json**: Adding a file without updating the manifest array will cause it to be ignored.
2. **Dangling cross-reference**: Referencing an ID that doesn't exist causes validation to fail.
3. **ID mismatch**: Filename must match the ID inside the file (or the file won't be found).
4. **Missing array defaults**: Forgetting `#[serde(default)]` on optional fields causes parse errors.

## Example: Adding a New Race

1. Create `content/races/my-race.json`:
```json
{
  "id": "my-race",
  "name": "My Race",
  "description": "A playable race",
  "faction_id": "some-faction",
  "traits": ["racial-trait-ability"],
  "stats": {
    "strength": 2,
    "dexterity": 0,
    "constitution": 1,
    "intelligence": 0,
    "wisdom": 0,
    "charisma": 0
  }
}
```

2. Update `content/manifest.json`:
```json
{
  "races": [
    ...existing,
    {"$ref": "races/my-race.json"}
  ]
}
```

3. Run validation: `cargo nextest run`

## Testing Content Locally

```bash
# Run content-schema tests
cargo nextest run

# Run full check
bin/check
```
