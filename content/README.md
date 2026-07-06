# content/ — the data-driven game layer

**Core is compiled (`crates/`); content is data (here).** Factions, classes,
races, abilities, quests and zones are files, not code. Change one and the game
changes with **no `cargo build`**. If altering a faction needs a recompile, the
split is wrong.

This is how operators get the full thing to customize — add maps, retune rules,
ship a total-conversion datapack — while our compiled core stays strong. The
bundle root is [`manifest.json`](manifest.json) (datapack metadata only); every
entity lives as its own small file under a per-domain directory. The
`omm-content-schema` crate assembles the tree via `load_manifest_dir()` and
validates it against the core content API version.

```
content/
├── manifest.json           # Datapack metadata root (id, version, api_version)
├── factions/               # One Faction per file
├── races/                  # One Race per file
├── classes/                # One Class per file
├── abilities/              # One Ability per file, grouped by who references it:
│   ├── <class-id>/         #   referenced by that class's abilities[] (arcanist/, occultist/, …)
│   ├── racial/             #   referenced only by a race's traits[]
│   └── unassigned/         #   not (yet) referenced by any class or race
├── items/                  # One Item per file, grouped by item_type then level tier:
│   ├── weapons/            #   Weapon
│   ├── armor/              #   Armor
│   ├── consumables/        #   Consumable
│   ├── crafting-materials/ #   CraftingMaterial
│   ├── trinkets/           #   Trinket
│   ├── quest/              #   Quest
│   └── misc/               #   Misc  (each type splits into level tiers below)
│       ├── lvl-01-10/      #     required_level 1–10
│       ├── lvl-11-20/      #     required_level 11–20
│       └── lvl-21plus/     #     required_level 21+   (tier dirs created only when populated)
├── quests/                 # One Quest per file, grouped by level:
│   ├── lvl-01-10/          #   level 1–10
│   ├── lvl-11-20/          #   level 11–20
│   └── lvl-21-30/          #   level 21–30  (created only when populated)
├── zones/                  # One Zone per file, grouped by min_level:
│   ├── lvl-01-10/          #   min_level ≤ 10
│   └── lvl-11-50/          #   min_level > 10
├── spawn-tables/           # One SpawnTable per file, grouped by referencing zone:
│   ├── <zone-id>/          #   referenced by that zone's spawn_tables[] (first zone wins)
│   ├── dungeons/           #   referenced only by a dungeon's trash_spawn_tables[]
│   └── unzoned/            #   referenced by neither  (created only when populated)
├── dungeons/               # One Dungeon per file
└── economy/                # Economy, split across:
    ├── auction-houses/     #   one AuctionHouse per file
    ├── trading-rules/      #   one TradingRule per file
    └── starting.json       #   starting_gold_copper

assets/
├── manifest.json           # Asset manifest
├── meshes/                 # 3D models (glTF)
├── textures/               # Textures (PNG/KTX2)
└── audio/                  # Audio (Opus OGG)
```

Each domain directory is **recursively** auto-discovered: `load_domain_dir()`
globs `**/*.json`, so the subdirectories above are **organizational only** — the
loader keys every entity by its `id`, never its path. Drop a new `<id>.json`
anywhere under the right domain folder (or invent your own nesting) and it loads
— no index to edit, no array to append. Files are read in sorted-path order, so
the assembled manifest is deterministic. Because grouping is by `id`, moving a
file between subdirs never changes what loads or how it cross-references.

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
- All references are validated by `load_manifest_dir()` (the split-tree loader)
- Faction `hostile_to` arrays must reference valid faction IDs
- Faction `capital` must reference a valid zone ID
- Race `faction_id` must reference a valid faction
- Race `traits` arrays must reference valid ability IDs
- Class `abilities` arrays must reference valid ability IDs
- Quest `prerequisites` must reference valid quest IDs
- Quest objective `target_id` must reference valid item IDs (for Gather/Deliver) or zone IDs (for Explore)
- Quest reward items must reference valid item IDs
- Zone `spawn_tables` arrays must reference valid spawn table IDs
- Zone `parent_zone_id` must reference a valid zone ID
- Dungeon `entrance_zone_id` must reference a valid zone ID
- Dungeon `loot_tables` must reference valid item IDs

## Validation Checklist

Before committing content changes, verify:

- [ ] New entities are dropped as `<id>.json` in the right domain directory
- [ ] All IDs are unique and match their filenames
- [ ] All cross-references resolve (no dangling IDs)
- [ ] `cargo nextest run` passes (content-schema tests)
- [ ] `bin/check` passes (full CI gate)

## Common Pitfalls

1. **Wrong directory**: A file placed outside its domain directory (or nested wrongly under `items/` / `economy/`) won't be discovered and is silently ignored.
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

2. That's it — the file is auto-discovered by `load_manifest_dir()`. (Bump
   `version` in `manifest.json` only when shipping a new datapack release.)

3. Run validation: `cargo nextest run`

## Testing Content Locally

```bash
# Run content-schema tests
cargo nextest run

# Run full check
bin/check
```
