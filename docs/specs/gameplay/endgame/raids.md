# Raids — the Emberheart Saga's spine

> One **raid tier per [Age](../progression/ages.md)**, each advancing the [Emberheart Saga](../01-world-and-cosmology.md) — the world-soul waking, one confrontation at a time. Raids are the coordinated pinnacle *and* the anchor of the whole [gear ladder](../itemization.md): a tier's loot is the **30-level relevance window**'s origin point. Blizzard nailed the spectacle and the difficulty tiers, then wrecked both by obsoleting the reward in an hour and making the weekly lockout the *only* path ([pillars](../00-design-pillars.md) #2, #5). We keep the mountain; we fix the erosion.

## Difficulties (one tier, four ways in)
| Difficulty | Roster | Sizing | For | Loot band |
|---|---|---|---|---|
| **Story** | Solo or any size | Scales to you | Experiencing the [Saga](../01-world-and-cosmology.md) beat, no group barrier | Entry / catch-up ([itemization](../itemization.md)) |
| **Normal** | 10–30 | **Flexible** | The broad guild/pug audience | Tier baseline |
| **Heroic** | 10–30 | **Flexible** | Organized groups wanting real challenge | Tier standard |
| **Mythic** | **Fixed 20** | Locked roster | The pinnacle — tuned to the edge | Tier apex — the relevance anchor |

- **Cross-faction throughout** — [Concord](../factions/README.md) and [Pact](../factions/README.md) in one raid, day 1 ([pillars](../00-design-pillars.md) #4).
- **Flex on Normal/Heroic** — bring who shows up; tuning scales to headcount. Mythic stays fixed-20 so the top rung has a stable, tournament-legible standard.
- **Story is real endgame-entry**, not a cutscene — a solo player sees the Age's climax and earns catch-up gear.

## The 30-level relevance anchor (lives here)
The **tier-anchor gear** drops from raids ([itemization](../itemization.md)). An Age-N tier's loot is **BiS-viable for three Ages** — Age N plus the two after — only clearly surpassed by the Age-(N+3) tier. Concretely: your Mythic clear isn't quest-greened in the next zone's first hour; it *matters for a season* ([pillars](../00-design-pillars.md) #2). This is the ladder's top rung, so it's where the window is set — dungeons, delves, and PvP band *relative* to it.

## Deterministic loot + the weekly cache (not the only path)
| Their model | Ours |
|---|---|
| Personal-loot RNG + titanforging lottery | **Fixed iLvl per boss**, group or **targeted** loot ([itemization](../itemization.md) #13) |
| No pity for the drop that never comes | **Bad-luck protection** — a currency accrues to a **guaranteed pick** |
| Weekly vault as *the* progression gate | A **weekly cache** (great-vault-equivalent) is *a* bonus, **not the only path** ([pillars](../00-design-pillars.md) #5) — [dungeons](dungeons.md)/[delves](delves.md)/[crafting](../world-systems/professions.md) fill the same slots |
| Lockout-or-nothing | Lockout is one route; parallel [rails](../progression/rails.md) reach raid-adjacent gear too |

A raid set is something you can **plan and finish** — essential so an [era server](../progression/era-servers.md) tier is genuinely toppable, not a moving target.

## Distilled from
| WoW system | Reimagined as | Keep / fix |
|---|---|---|
| Vanilla 40-man → 25/10 → **flex** → Mythic-20 | Story/Normal/Heroic-flex + fixed-20 Mythic | **Keep** the difficulty spectrum & flex sizing; **keep** spectacle & Mythic prestige |
| Raid epics obsoleted next zone | 30-level relevance anchor | **Fix** ([pillars](../00-design-pillars.md) #2) |
| Titanforging / personal-loot RNG | Fixed iLvl, targetable, bad-luck-protected | **Fix** ([pillars](../00-design-pillars.md) #13) |
| Weekly lockout as sole power path | Weekly cache as a bonus among parallel paths | **Fix** ([pillars](../00-design-pillars.md) #5) |
| Faction-locked raids | Cross-faction rosters | **Fix** ([pillars](../00-design-pillars.md) #4) |

## Rules
- **One raid tier per Age**, each a [Saga](../01-world-and-cosmology.md) chapter — shipped as [data](../../game-server/content-scripting/README.md), no recompile.
- **Four difficulties**: Story (solo-able) → Normal → Heroic (flex) → Mythic (fixed 20, the pinnacle).
- **Raids set the relevance anchor** — tier-anchor gear, BiS-viable three Ages ([itemization](../itemization.md)).
- **Deterministic, finishable loot** + bad-luck protection; the **weekly cache is a bonus, never the only path**.
- **Cross-faction always; era-preserved always** ([era-servers](../progression/era-servers.md)).
- **Loot ownership = one [persistence](../../game-server/persistence/README.md) transaction** — never via cache ([economy](../../game-server/economy/README.md)).

## Links
[README](README.md) · [dungeons](dungeons.md) · [delves](delves.md) · [pvp](pvp.md) · [itemization](../itemization.md) · [ages](../progression/ages.md) · [rails](../progression/rails.md) · [era-servers](../progression/era-servers.md) · [factions](../factions/README.md) · [economy](../../game-server/economy/README.md) · [pillars](../00-design-pillars.md)
