# Factions — the two-faction model

> Two peoples, one world. A faction is who you *are* and whose story you inherit — **not** a cage. [Auralon](../01-world-and-cosmology.md) ships with two, but the count is data ([modding](../../../initial-idea/06-modding-and-extensibility.md)). All names original ([legal](../../../initial-idea/01-legal-and-licensing.md)).

## What it is
A **faction** is a playable alliance of peoples with a shared ethos, colors, home, and place in the [Emberheart Saga](../01-world-and-cosmology.md). It sets your starting story, your capital, your NPC allies and rivals — a legible identity in a two-sided rivalry as old as the genre. We ship exactly two, because two makes the conflict *readable*: a player knows instantly who they are and who they aren't.

## The two factions
| Faction | Archetype | Ethos | Force lean | Colors |
|---|---|---|---|---|
| **[The Aurelian Concord](concord.md)** ("the Concord") | Alliance | Civilization, duty, the light of reason — a pluralist coalition of peoples bound by law and the dawn-faith | **[Lattice](../01-world-and-cosmology.md) + [Dawn](../01-world-and-cosmology.md)** | Gold / azure |
| **[The Wildreach Pact](pact.md)** ("the Pact") | Horde | Freedom, kinship, endurance — a confederation of the exiled and the reclaimed, bound by survival and honor | **[Bloom](../01-world-and-cosmology.md) + [Hollow](../01-world-and-cosmology.md)** | Crimson / bronze |

Neither is "the good guys." Both are coalitions, not empires — the Concord is an alliance of free peoples, the Pact a confederation of the untamed. The lean is a *center of gravity*, not a wall: nearly any race can be nearly any [class](../classes/README.md) (pillar #14).

## The reversal — factions are identity, not a prison
The defining decision ([pillars](../00-design-pillars.md) #4). Blizzard walled Alliance and Horde apart for ~15 years — no shared groups, no shared chat, a war players never asked to keep fighting. It was the single most-hated structural choice in the genre. We reverse it on day 1.

| Concern | Blizzard | Us |
|---|---|---|
| Cross-faction grouping | Blocked for ~15 years | **Allowed from launch** — dungeons, raids, delves, world |
| The war | Mandatory, eternal, story-forced | Real and **ideological**, never mandatory |
| World-threats | Factions bicker mid-apocalypse | Against the **[Emberheart Saga](../01-world-and-cosmology.md)**, they cooperate |
| Rolling an alt of the "other side" | Feels like betrayal | A different *view* of the same world |

The rivalry stays — different capitals, different quests, real PvP ([pvp](../endgame/README.md)), a genuine philosophical divide. What we drop is the *segregation*: a friend on the other faction is a groupmate, not a stranger behind a wall.

## Data-driven count
Two is the **reference** roster, not a hard limit. Factions are content assets — ethos, races, colors, capital, starting story ([modding](../../../initial-idea/06-modding-and-extensibility.md)). An operator can ship a third, or a one-faction PvE world, or N, with no `cargo build` ([CLAUDE.md](../../../../CLAUDE.md)).

## Distilled from
| Source | Verdict |
|---|---|
| Alliance / Horde two-faction rivalry | **Keep** — two legible identities is great design; the classic silhouette-vs-silhouette read is worth preserving. |
| ~15 years of faction *segregation* | **Fix** — cross-faction grouping from day 1; the war is story, not a mechanical wall. |
| "Faction = your eternal enemy" framing | **Fix** — conflict is ideological; world-threats unite both. |
| Hardcoded 2-faction assumption | **Fix** — faction count is data, N-capable. |

## Rules
- **Two shipped, N-capable.** Faction count is a content decision, never a recompile.
- **Cross-faction grouping is legal from launch** — enforced allow, not a toggle bolted on later.
- **No faction is "the good one"** — both are pluralist coalitions with real virtues and real flaws.
- **The force lean is gravity, not law** — race rarely gates class ([classes](../classes/README.md)).
- **Original IP only** — no ripped faction, race, or capital name ([legal](../../../initial-idea/01-legal-and-licensing.md)).

## Links
[Concord](concord.md) · [Pact](pact.md) · [world & cosmology](../01-world-and-cosmology.md) · [design pillars](../00-design-pillars.md) · [classes](../classes/README.md) · [ages](../progression/ages.md) · [progression](../progression/README.md) · [itemization](../itemization.md) · [gameplay README](../README.md) · [modding](../../../initial-idea/06-modding-and-extensibility.md) · [game-server](../../game-server/README.md)
