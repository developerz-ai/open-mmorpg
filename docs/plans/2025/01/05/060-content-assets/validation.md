# Content Validation - Error Reference

## Overview

All content is validated on load via `omm-content-schema::load_manifest()`. Validation ensures:

1. JSON parses correctly
2. API version matches
3. No empty IDs
4. All cross-references resolve

## Validation Errors

### API Version Mismatch

**Error**: `datapack targets content API v{version}, core provides v{CONTENT_API_VERSION}`

**Cause**: Manifest `api_version` doesn't match core `CONTENT_API_VERSION`

**Fix**: Update manifest `api_version` to match core

**Example**:
```json
{
  "api_version": 2  // Wrong! Core provides v1
}
```

### Empty Manifest ID

**Error**: `manifest id is empty`

**Cause**: Manifest `id` field is empty or whitespace

**Fix**: Set a valid manifest ID

**Example**:
```json
{
  "id": ""  // Wrong!
}
```

### Empty Faction ID

**Error**: `faction id is empty`

**Cause**: Faction `id` field is empty or whitespace

**Fix**: Set a valid faction ID

### Empty Race ID

**Error**: `race id is empty`

**Cause**: Race `id` field is empty or whitespace

**Fix**: Set a valid race ID

### Empty Class ID

**Error**: `class id is empty`

**Cause**: Class `id` field is empty or whitespace

**Fix**: Set a valid class ID

### Empty Quest ID

**Error**: `quest id is empty`

**Cause**: Quest `id` field is empty or whitespace

**Fix**: Set a valid quest ID

### Empty Zone ID

**Error**: `zone id is empty`

**Cause**: Zone `id` field is empty or whitespace

**Fix**: Set a valid zone ID

### Empty Dungeon ID

**Error**: `dungeon id is empty`

**Cause**: Dungeon `id` field is empty or whitespace

**Fix**: Set a valid dungeon ID

## Cross-Reference Errors

### Faction Hostile Reference

**Error**: `faction '{faction_id}' is hostile to unknown faction '{target_id}'`

**Cause**: Faction's `hostile_to` references a non-existent faction

**Fix**: Add the referenced faction or correct the reference

**Example**:
```json
{
  "id": "dawnward",
  "hostile_to": ["nonexistent"]  // Wrong! "nonexistent" doesn't exist
}
```

### Race Faction Reference

**Error**: `race '{race_id}' faction_id '{target_id}' references unknown faction`

**Cause**: Race's `faction_id` references a non-existent faction

**Fix**: Add the referenced faction or correct the reference

**Example**:
```json
{
  "id": "my-race",
  "faction_id": "nonexistent"  // Wrong!
}
```

### Race Trait Reference

**Error**: `race '{race_id}' trait '{trait_id}' references unknown ability`

**Cause**: Race's `traits` array references a non-existent ability

**Fix**: Add the referenced ability or correct the reference

**Example**:
```json
{
  "traits": ["nonexistent"]  // Wrong!
}
```

### Class Ability Reference

**Error**: `class '{class_id}' ability '{ability_id}' references unknown ability`

**Cause**: Class's `abilities` array references a non-existent ability

**Fix**: Add the referenced ability or correct the reference

**Example**:
```json
{
  "abilities": ["nonexistent"]  // Wrong!
}
```

### Quest Prerequisite Reference

**Error**: `quest '{quest_id}' prereq '{prereq_id}' references unknown quest`

**Cause**: Quest's `prerequisites` array references a non-existent quest

**Fix**: Add the referenced quest or correct the reference

**Example**:
```json
{
  "prerequisites": ["nonexistent"]  // Wrong!
}
```

### Quest Next Quest Reference

**Error**: `quest '{quest_id}' next_quest_id '{next_id}' references unknown quest`

**Cause**: Quest's `next_quest_id` references a non-existent quest

**Fix**: Add the referenced quest or correct the reference

**Example**:
```json
{
  "next_quest_id": "nonexistent"  // Wrong!
}
```

### Quest Objective Item Reference

**Error**: `quest '{quest_id}' objective target '{target_id}' references unknown item`

**Cause**: Quest objective's `target_id` references a non-existent item (for Gather/Deliver objectives)

**Fix**: Add the referenced item or correct the reference

**Example**:
```json
{
  "objective_type": "Gather",
  "target_id": "nonexistent"  // Wrong!
}
```

### Quest Objective Zone Reference

**Error**: `quest '{quest_id}' objective target '{target_id}' references unknown zone`

**Cause**: Quest objective's `target_id` references a non-existent zone (for Explore objectives)

**Fix**: Add the referenced zone or correct the reference

**Example**:
```json
{
  "objective_type": "Explore",
  "target_id": "nonexistent"  // Wrong!
}
```

### Quest Reward Item Reference

**Error**: `quest '{quest_id}' reward item '{item_id}' references unknown item`

**Cause**: Quest's `rewards.items` or `rewards.choice_items` references a non-existent item

**Fix**: Add the referenced item or correct the reference

**Example**:
```json
{
  "rewards": {
    "items": ["nonexistent"]  // Wrong!
  }
}
```

### Zone Spawn Table Reference

**Error**: `zone '{zone_id}' spawn_table '{table_id}' references unknown spawn table`

**Cause**: Zone's `spawn_tables` array references a non-existent spawn table

**Fix**: Add the referenced spawn table or correct the reference

**Example**:
```json
{
  "spawn_tables": ["nonexistent"]  // Wrong!
}
```

### Zone Parent Reference

**Error**: `zone '{zone_id}' parent_zone_id '{parent_id}' references unknown zone`

**Cause**: Zone's `parent_zone_id` references a non-existent zone

**Fix**: Add the referenced zone or correct the reference

**Example**:
```json
{
  "parent_zone_id": "nonexistent"  // Wrong!
}
```

### Faction Capital Reference

**Error**: `faction '{faction_id}' capital '{capital_id}' references unknown zone`

**Cause**: Faction's `capital` references a non-existent zone

**Fix**: Add the referenced zone or correct the reference

**Example**:
```json
{
  "capital": "nonexistent"  // Wrong!
}
```

### Dungeon Entrance Zone Reference

**Error**: `dungeon '{dungeon_id}' entrance_zone_id '{zone_id}' references unknown zone`

**Cause**: Dungeon's `entrance_zone_id` references a non-existent zone

**Fix**: Add the referenced zone or correct the reference

**Example**:
```json
{
  "entrance_zone_id": "nonexistent"  // Wrong!
}
```

### Dungeon Loot Table Reference

**Error**: `dungeon '{dungeon_id}' loot_table '{table_id}' references unknown item`

**Cause**: Dungeon's `loot_tables` array references a non-existent item

**Fix**: Add the referenced item or correct the reference

**Example**:
```json
{
  "loot_tables": ["nonexistent"]  // Wrong!
}
```

## Debugging Tips

1. **Use descriptive IDs**: Makes errors easier to read
2. **Check manifest first**: Ensure all files are referenced
3. **Validate incrementally**: Add content piece by piece
4. **Read error carefully**: Error messages point to the exact issue
5. **Use `bin/check`**: Catches validation errors before commit

## Validation Flow

```
1. Parse JSON → failure if malformed
2. Check API version → failure if mismatch
3. Check empty IDs → failure if any empty
4. Validate cross-references → failure if any dangling
5. Success → content loads
```

## Testing Validation

```bash
# Run content-schema tests
cargo nextest run --package omm-content-schema

# Manual validation
cargo test --package omm-content-schema validation_errors

# Validate manifest
bun scripts/validate-content.ts
```
