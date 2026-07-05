# Content Authoring Guide

## Getting Started

Creating new content for Open-MMORPG is straightforward: edit JSON files and update the manifest.

## File Organization

```
content/
├── manifest.json           # Root manifest (REQUIRED)
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
└── economy/                # Economy definitions
```

## Authoring Workflow

1. Create or edit a JSON file in the appropriate directory
2. Update `content/manifest.json` to reference the new file
3. Run `bin/check` to validate
4. Test in-game

## File Naming Conventions

- Use kebab-case: `aurelian-concord.json`, `sunken-crypts.json`
- One definition per file
- ID must match filename (without `.json`)

## Creating a Faction

**File**: `content/factions/my-faction.json`

```json
{
  "id": "my-faction",
  "name": "My Faction",
  "description": "A new player faction",
  "colors": {
    "primary": "#4a90e2",
    "secondary": "#2c5f8d"
  },
  "capital": "capital-zone",
  "hostile_to": ["enemy-faction"]
}
```

**Update manifest**:
```json
{
  "factions": [
    {"$ref": "factions/my-faction.json"},
    ...existing
  ]
}
```

## Creating a Race

**File**: `content/races/my-race.json`

```json
{
  "id": "my-race",
  "name": "My Race",
  "description": "A playable race",
  "faction_id": "my-faction",
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

**Update manifest**:
```json
{
  "races": [
    {"$ref": "races/my-race.json"},
    ...existing
  ]
}
```

## Creating a Class

**File**: `content/classes/my-class.json`

```json
{
  "id": "my-class",
  "name": "My Class",
  "description": "A new character class",
  "role": "tank",
  "abilities": ["basic-attack", "defensive-stance"],
  "stat_growth": {
    "strength": 1,
    "dexterity": 0,
    "constitution": 1,
    "intelligence": 0,
    "wisdom": 0,
    "charisma": 0
  },
  "hp_per_level": 12,
  "resource_per_level": 0
}
```

**Update manifest**:
```json
{
  "classes": [
    {"$ref": "classes/my-class.json"},
    ...existing
  ]
}
```

## Creating an Ability

**File**: `content/abilities/my-ability.json`

```json
{
  "id": "my-ability",
  "name": "My Ability",
  "description": "A powerful ability",
  "max_rank": 1,
  "cooldown_sec": 10.0,
  "resource_cost": 50,
  "cast_time_sec": 2.0,
  "range_yards": 25.0,
  "effects": [
    {
      "effect": "Damage",
      "magnitude": 100.0,
      "scaling": 0.8,
      "target": "enemy"
    }
  ]
}
```

**Update manifest**:
```json
{
  "abilities": [
    {"$ref": "abilities/my-ability.json"},
    ...existing
  ]
}
```

## Creating an Item

**File**: `content/items/weapons/my-weapon.json`

```json
{
  "id": "my-weapon",
  "name": "My Weapon",
  "description": "A powerful weapon",
  "item_type": "Weapon",
  "slot": "mainhand",
  "required_level": 10,
  "stats": {
    "strength": 5,
    "dexterity": 2,
    "constitution": 0,
    "intelligence": 0,
    "wisdom": 0,
    "charisma": 0
  },
  "quality": "rare",
  "value_copper": 1000,
  "icon": "items/my-weapon.png",
  "mesh": "weapons/my-weapon.gltf",
  "max_stack": 1
}
```

**Update manifest**:
```json
{
  "items": [
    {"$ref": "items/weapons/my-weapon.json"},
    ...existing
  ]
}
```

## Creating a Quest

**File**: `content/quests/my-quest.json`

```json
{
  "id": "my-quest",
  "name": "My Quest",
  "description": "Complete objectives for rewards",
  "level": 5,
  "prerequisites": ["previous-quest"],
  "objectives": [
    {
      "objective_type": "Kill",
      "target_id": "enemy-npc",
      "count": 10,
      "description": "Defeat 10 enemies"
    },
    {
      "objective_type": "Gather",
      "target_id": "quest-item",
      "count": 5,
      "description": "Collect 5 items"
    }
  ],
  "rewards": {
    "experience": 1000,
    "gold_copper": 500,
    "choice_items": ["reward-item-a", "reward-item-b"],
    "items": ["mandatory-item"]
  },
  "giver_id": "quest-giver-npc",
  "turn_in_id": "quest-turn-in-npc",
  "next_quest_id": "next-quest"
}
```

**Update manifest**:
```json
{
  "quests": [
    {"$ref": "quests/my-quest.json"},
    ...existing
  ]
}
```

## Creating a Zone

**File**: `content/zones/my-zone/zone.json`

```json
{
  "id": "my-zone",
  "name": "My Zone",
  "min_level": 10,
  "max_level": 20,
  "safe_locations": [
    {
      "id": "zone-entrance",
      "position": [0.0, 0.0, 0.0],
      "yaw": 0.0
    }
  ],
  "controlling_factions": ["my-faction"],
  "spawn_tables": ["my-zone-mobs"],
  "navmesh": "nav/my-zone.nav"
}
```

**Update manifest**:
```json
{
  "zones": [
    {"$ref": "zones/my-zone/zone.json"},
    ...existing
  ]
}
```

## Creating a Dungeon

**File**: `content/dungeons/my-dungeon/dungeon.json`

```json
{
  "id": "my-dungeon",
  "name": "My Dungeon",
  "min_level": 15,
  "recommended_level": 18,
  "max_players": 5,
  "boss_ids": ["boss-1", "boss-2", "boss-3"],
  "trash_spawn_tables": ["dungeon-trash"],
  "loot_tables": ["boss-loot-1", "boss-loot-2", "boss-loot-3"],
  "time_limit_minutes": 120,
  "lockout_hours": 24,
  "entrance_zone_id": "entrance-zone",
  "entrance_position": [100.0, 0.0, 100.0]
}
```

**Update manifest**:
```json
{
  "dungeons": [
    {"$ref": "dungeons/my-dungeon/dungeon.json"},
    ...existing
  ]
}
```

## Best Practices

1. **Always validate** after changes: `bin/check`
2. **Use descriptive IDs** that match filenames
3. **Balance carefully**: test progression curves
4. **Reference properly**: all IDs must exist in manifest
5. **Document intent**: use description fields
6. **Version control**: commit content changes with clear messages

## Common Mistakes

1. **Forgot manifest update**: New file won't load
2. **ID mismatch**: Filename ≠ ID causes load failure
3. **Dangling reference**: Referenced ID doesn't exist
4. **Invalid JSON**: Syntax error breaks parser
5. **Wrong type**: Used `"Damage"` instead of `"damage"` (case-sensitive)

## Testing Content

```bash
# Validate content
bin/check

# Run content-schema tests
cargo nextest run --package omm-content-schema

# Run benchmarks
cargo bench --package omm-content-schema
```
