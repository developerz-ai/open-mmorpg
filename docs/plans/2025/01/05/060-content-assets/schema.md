# Content Schema - Type Reference

## Core Types

### Manifest
The root content bundle definition.

```rust
pub struct Manifest {
    pub id: String,              // "open-mmorpg.base"
    pub version: String,         // "0.1.0"
    pub api_version: u32,        // 1
    pub factions: Vec<Faction>,
    pub races: Vec<RaceDef>,
    pub classes: Vec<ClassDef>,
    pub abilities: Vec<AbilityDef>,
    pub items: Vec<ItemDef>,
    pub quests: Vec<QuestDef>,
    pub zones: Vec<ZoneDef>,
    pub spawn_tables: Vec<SpawnTable>,
    pub dungeons: Vec<DungeonDef>,
    pub economy: EconomyData,
    pub asset_manifest_ref: Option<String>,
}
```

## Content Types

### Faction
```rust
pub struct Faction {
    pub id: String,              // "dawnward"
    pub name: String,            // "The Dawnward Pact"
    pub description: String,     // Faction ethos
    pub colors: FactionColors,    // UI colors
    pub capital: Option<String>,  // Zone ID
    pub hostile_to: Vec<String>, // Faction IDs
}
```

### RaceDef
```rust
pub struct RaceDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub faction_id: String,
    pub traits: Vec<String>,      // Ability IDs
    pub stats: StatModifiers,
}
```

### ClassDef
```rust
pub struct ClassDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub role: String,             // "tank", "healer", "dps"
    pub abilities: Vec<String>,    // Ability IDs
    pub stat_growth: StatModifiers,
    pub hp_per_level: u16,
    pub resource_per_level: u16,
}
```

### AbilityDef
```rust
pub struct AbilityDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub max_rank: u8,
    pub cooldown_sec: f32,
    pub resource_cost: u16,
    pub cast_time_sec: f32,
    pub range_yards: f32,
    pub effects: Vec<AbilityEffect>,
}
```

### AbilityEffect
```rust
pub struct AbilityEffect {
    pub effect: AbilityEffectType,
    pub magnitude: f32,
    pub scaling: f32,
    pub target: String,
}

pub enum AbilityEffectType {
    Damage, Heal, ApplyAura, Summon, Teleport,
    Buff, Debuff, Dot, Hot,
}
```

### ItemDef
```rust
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub slot: Option<String>,      // "mainhand", "head", etc.
    pub required_level: u8,
    pub stats: StatModifiers,
    pub quality: String,            // "common", "rare", "epic"
    pub value_copper: u32,
    pub icon: Option<String>,
    pub mesh: Option<String>,
    pub max_stack: u16,
}

pub enum ItemType {
    Weapon, Armor, Trinket, Consumable,
    Quest, CraftingMaterial, Misc,
}
```

### QuestDef
```rust
pub struct QuestDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: u8,
    pub prerequisites: Vec<String>,  // Quest IDs
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestRewards,
    pub giver_id: Option<String>,
    pub turn_in_id: Option<String>,
    pub next_quest_id: Option<String>,
}
```

### QuestObjective
```rust
pub struct QuestObjective {
    pub objective_type: QuestObjectiveType,
    pub target_id: String,
    pub count: u8,
    pub description: String,
}

pub enum QuestObjectiveType {
    Kill, Gather, Speak, Deliver, Explore,
}
```

### ZoneDef
```rust
pub struct ZoneDef {
    pub id: String,
    pub name: String,
    pub min_level: u8,
    pub max_level: u8,
    pub safe_locations: Vec<SafeLocation>,
    pub controlling_factions: Vec<String>,
    pub spawn_tables: Vec<String>,
    pub parent_zone_id: Option<String>,
    pub navmesh: Option<String>,
}
```

### DungeonDef
```rust
pub struct DungeonDef {
    pub id: String,
    pub name: String,
    pub min_level: u8,
    pub recommended_level: u8,
    pub max_players: u8,
    pub boss_ids: Vec<String>,
    pub trash_spawn_tables: Vec<String>,
    pub loot_tables: Vec<String>,
    pub time_limit_minutes: u32,
    pub lockout_hours: u32,
    pub entrance_zone_id: Option<String>,
    pub entrance_position: Option<[f32; 3]>,
}
```

## Economy Types

### AuctionHouseDef
```rust
pub struct AuctionHouseDef {
    pub id: String,
    pub name: String,
    pub zone_id: String,
    pub position: Option<[f32; 3]>,
    pub fee_percentage: f32,
    pub min_bid_increment: f32,
    pub max_listings_per_account: u16,
    pub listing_duration_hours: u32,
    pub deposit_percentage: f32,
}
```

### TradingRuleDef
```rust
pub struct TradingRuleDef {
    pub item_pattern: String,    // "soulbound", "quest_*"
    pub tradable: bool,
    pub auctionable: bool,
    pub mailing_allowed: bool,
}
```

## Helper Types

### StatModifiers
```rust
pub struct StatModifiers {
    pub strength: i8,
    pub dexterity: i8,
    pub constitution: i8,
    pub intelligence: i8,
    pub wisdom: i8,
    pub charisma: i8,
}
```

### SafeLocation
```rust
pub struct SafeLocation {
    pub id: String,
    pub position: [f32; 3],
    pub yaw: f32,
}
```

## Validation Rules

All content is validated on load:

1. **API Version**: Manifest must match core `CONTENT_API_VERSION`
2. **Empty IDs**: No `id` field may be empty
3. **Cross-References**:
   - Faction `hostile_to` → valid faction IDs
   - Race `faction_id` → valid faction ID
   - Race `traits` → valid ability IDs
   - Class `abilities` → valid ability IDs
   - Quest `prerequisites` → valid quest IDs
   - Quest `next_quest_id` → valid quest ID
   - Quest objective `target_id` → valid item/zone/entity IDs
   - Quest rewards → valid item IDs
   - Zone `spawn_tables` → valid spawn table IDs
   - Zone `parent_zone_id` → valid zone ID
   - Dungeon `entrance_zone_id` → valid zone ID
   - Dungeon `loot_tables` → valid item IDs

See `validation.md` for error details.
