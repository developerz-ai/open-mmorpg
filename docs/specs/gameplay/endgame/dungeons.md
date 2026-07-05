# Dungeons — 5-player & the Ascendant Keys

> Five players, one instance, on demand — the pick-up-and-go pillar. On top sits **Ascendant Keys**: an infinitely-scaling, timed, affix-modified keystone system (the Mythic+ archetype), where difficulty and reward climb together with no ceiling. WoW got the *engineering* right — scalable dungeons, cross-realm tech — and the *sociology* wrong: anonymous auto-queue [dungeon finder](../progression/ages.md) hollowed out server community. We keep the tech, fix the erosion.

## The dungeon ladder
| Mode | Difficulty | Reward band ([itemization](../itemization.md)) | Notes |
|---|---|---|---|
| **Normal / Heroic** | Fixed | Above world, entry | Learn the routes; catch-up friendly |
| **Ascendant** (keyed) | **Infinite scaling** | Climbs with key level toward raid-adjacent | Timed, weekly affixes, the endgame loop |

## Ascendant Keys (the scaling system)
- **A key names a dungeon + a level.** Complete in time → the key **upgrades** (climbs); fall short → it still counts, drops a step. No hard wall, no ceiling.
- **Level scales enemy health/damage** and, past thresholds, adds **affix-modifiers** — rotating weekly world-state twists themed to the six [forces](../01-world-and-cosmology.md) (e.g. a **Rift** affix that fractures on death, a **Hollow** affix that raises slain foes, a **Lattice** affix that armors packs in order). Affixes change *how* you play, not just the numbers.
- **Gear scales with key level** — higher keys drop higher iLvl, deterministic and bad-luck-protected ([itemization](../itemization.md) #13). The dungeon end-cache and the [weekly cache](raids.md) both pull from your best clears.
- **Timed, but not lockout-gated** — infinitely repeatable; you push when *you* choose, not when a reset permits ([rails](../progression/rails.md)).

## Group finder — an option, engineered *against* erosion
The **Hollow King (Age III)** keep/fix note ([ages](../progression/ages.md)): the dungeon finder was a genuine QoL win *and* the thing that dissolved server identity — anonymous cross-realm strangers you'd never see again killed the reason to be known. Our finder keeps the convenience and rebuilds the community incentive:
- **Opt-in, never the default path.** Solo queue exists; it is *a* door, not *the* door.
- **Reward grouping.** Premade/guild/community groups earn a standing bonus (renown, deterministic-loot momentum) — being known pays.
- **First-class premade & guild tools** — build a roster, save comps, find *named* people; cross-realm ([sharding](../../game-server/sharding/README.md)) connects without anonymizing.
- **Cross-faction** — [Concord](../factions/README.md) + [Pact](../factions/README.md) in one key ([pillars](../00-design-pillars.md) #4).

## Distilled from
| WoW system | Reimagined as | Keep / fix |
|---|---|---|
| **LFD** anonymous auto-queue | Opt-in finder + grouping rewards + premade tools | **Keep** convenience & cross-realm; **fix** the community erosion (Age III note) |
| **Challenge Modes** (timed, fixed) | Timed runs as the core skill test | **Keep** — timing is a clean skill signal |
| **Mythic+** infinite keys + affixes | **Ascendant Keys** — scaling, affixed, gear-scaled | **Keep** the scaling loop; **fix** RNG loot → deterministic ([itemization](../itemization.md) #13) |
| Cross-realm tech | Named cross-realm grouping | **Keep** the engineering, drop the anonymity |

## Rules
- **Ascendant Keys scale infinitely** — timed, weekly [force](../01-world-and-cosmology.md)-themed affixes, gear climbs with key level.
- **Loot deterministic & bad-luck-protected**; higher keys → higher fixed iLvl ([itemization](../itemization.md)).
- **Group finder is opt-in** and **rewards grouping** — never anonymous-auto-queue-as-default ([pillars](../00-design-pillars.md) #15).
- **Cross-faction, cross-realm, era-preserved** ([sharding](../../game-server/sharding/README.md), [era-servers](../progression/era-servers.md)).
- **Dungeons are a full gear rail** to raid-adjacent bands — parallel, not a raid prerequisite ([rails](../progression/rails.md)).
- **Content is data** — dungeons, keys, affixes ship without a recompile ([content-scripting](../../game-server/content-scripting/README.md)).

## Links
[README](README.md) · [raids](raids.md) · [delves](delves.md) · [pvp](pvp.md) · [itemization](../itemization.md) · [rails](../progression/rails.md) · [ages](../progression/ages.md) · [era-servers](../progression/era-servers.md) · [sharding](../../game-server/sharding/README.md) · [factions](../factions/README.md) · [pillars](../00-design-pillars.md)
