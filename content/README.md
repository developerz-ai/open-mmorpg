# content/ — the data-driven game layer

**Core is compiled (`crates/`); content is data (here).** Factions, classes,
races, abilities, quests and zones are files, not code. Editing definitions
within the supported schema needs **no `cargo build`**. If altering a faction
needs a recompile, the split is wrong.

This is how operators get the full thing to customize — add maps, retune rules,
ship a total-conversion datapack — while our compiled core stays strong. The
bundle root is [`manifest.json`](manifest.json) (datapack metadata only); every
entity lives as its own small file under a per-domain directory. The
`omm-content-schema` crate assembles the tree via `load_manifest_dir()` and
validates it against the core content API version.

The shard currently loads and validates this datapack at boot and builds its
runtime ability table from it. Loading a definition does not mean every domain
is integrated into gameplay; universal runtime integration and datapack
hot-reload are not implied. Restart the shard to pick up ability-data edits.

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
├── README.md               # Current inventory and planned asset workflow
└── manifest.json           # Placeholder names only; payload files are absent
```

Asset directories, generation and hot-swapping are plans, not shipped payloads
or workflows; see [assets/README.md](../assets/README.md).

Each entity domain directory is **recursively** auto-discovered:
`load_domain_dir()` reads all nested `.json` files, so the grouping
subdirectories above are **organizational only** — JSON `id` fields are the
keys for references, never filenames. Drop a new `<id>.json`
anywhere under the right domain folder (or invent your own nesting) and it loads
— no index to edit, no array to append. Files are read in sorted-path order, so
the assembled manifest is deterministic. Moving a file within its domain can
change assembly order, but not its ID or cross-references. Economy uses fixed
roots: `economy/auction-houses/` and `economy/trading-rules/` are recursive;
starting gold is read specifically from `economy/starting.json`.

→ docs/architecture/05-ecs-and-scripting.md · docs/initial-idea/06-modding-and-extensibility.md

## Authoring Guidelines

### File Naming
- Use kebab-case: `aurelian-concord.json`, `sunken-crypts.json`
- One definition per file
- Match the filename to the JSON `id` (without extension) as an authoring convention; the loader does not enforce this match

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
- [ ] IDs are unique within their domain; filenames match IDs by authoring convention
- [ ] All cross-references resolve (no dangling IDs)
- [ ] `cargo nextest run` passes (content-schema tests)
- [ ] `bin/check` passes (full local gate, not the complete CI workflow)

## Common Pitfalls

1. **Wrong domain root**: Files outside the recognized domain roots are not loaded. Nesting anywhere under `items/` or another entity domain is supported; files there are parsed as that domain's type, not silently ignored. Economy requires the specific roots described above.
2. **Dangling cross-reference**: Referencing an ID that doesn't exist causes validation to fail.
3. **Filename mistaken for an ID**: Lookup and cross-references use the JSON `id`. A mismatched filename still loads, but violates the authoring convention and can confuse readers.
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

# Run the full local gate (CI has separate workflow steps)
bin/check
```
