# Living World — a world that remembers you

> The instanced MMO world resets and never noticed you were there ([pillars](../00-design-pillars.md) #15). We fix that with **GTA's** reactive-world systems, adapted to MMO scale ([09-gta6](../../../initial-idea/09-gta6-inspiration.md)). **Auralon** watches: guards see *witnesses*, not omniscient truth; crimes earn **bounties**; weather and the day/night cycle bend gameplay; a **world-news feed** surfaces what's happening; faction AI **remembers** you. This is the centerpiece of pillar #15. Original expression only — mechanics aren't copyrightable ([CLAUDE.md](../../../../CLAUDE.md)).

## GTA systems → our mapping (from [09](../../../initial-idea/09-gta6-inspiration.md))
| GTA system | Our mapping | Priority |
|---|---|---|
| Reactive police AI: severity → response tiers, **witness-based** (not omniscient), surrender | **Guard/faction AI** reacting to PvP crimes in cities; witnesses, bounties, escalation | High |
| Dynamic weather + day/night affecting **gameplay**, not just visuals | Weather/time drive **spawns, gathering yields, buffs, events** | Medium |
| In-world social layer (in-game phone/feed) | In-game **world-news / social feed** surfacing dynamic events | Medium |
| Seamless session merge/split, one persistent world | **Shard autoscaling** carries the reactive world seamlessly ([world-model](../../game-server/world-model/README.md)) | High |
| Dynamic ambient events reacting to the player | **Dynamic world events** + faction memory | High |

## Witness-based guard & faction AI
- **Not omniscient.** A city crime (attacking a flagged rival, ganking a questgiver's ward) is only *known* if a **witness** — NPC or player — perceives it and reports. No witness, no heat.
- **Response tiers by severity** — a scuffle draws a warning; a massacre draws elite guards. Guards escalate, they don't teleport-snipe.
- **Surrender / de-escalation** — pay the bounty, flee the zone, or stand down; heat decays over time, faster if you make amends.
- Ties world-PvP crime to the **crime/bounty ladder** in [pvp](../endgame/pvp.md) — city aggression is a PvP act with reputational consequences, not a consequence-free gank.

## Bounty & reputation escalation
| Tier | Trigger | Response |
|---|---|---|
| **Noticed** | Witnessed minor aggression | Guard warning; small [renown](../progression/rails.md) ding with the offended faction |
| **Wanted** | Repeat/serious crime, witnessed | Bounty posted to the **world feed**; guards hunt; players can collect |
| **Marked** | Sustained spree | Elite response, bounty scales, faction cities hostile until it decays |
Reputation is **remembered** on the [renown](../progression/rails.md) rail — a marked player is treated differently by that faction's NPCs until they earn it back. Server-authoritative, persisted ([world-model](../../game-server/world-model/README.md)).

## Dynamic weather & day/night — gameplay, not wallpaper
- **Spawns** shift by time — nocturnal creatures, night-only rares, dawn-only events tied to the [Dawn](../01-world-and-cosmology.md) force.
- **Gathering yields** vary — certain [Foraging](professions.md) herbs bloom by weather/season; storms change [Delving](professions.md) node access.
- **Buffs & conditions** — weather grants or taxes (rain feeds [Bloom](../01-world-and-cosmology.md), fog aids stealth); a legible, sim-able layer, not RNG punishment.
- **Events** — storms, [Rift](../01-world-and-cosmology.md) tears, and force-tides fire dynamic world events by weather/time.

## World-news / social feed
An in-game feed (no external site needed) surfaces the living world: **bounties posted**, world events firing, neighborhood happenings ([housing](housing.md)), rare spawns, faction-front shifts. It's how a player *reads* the reactive world and decides where to go — replaces alt-tabbing to a tracker site ([09](../../../initial-idea/09-gta6-inspiration.md)).

## Dynamic world events & faction memory
- **Dynamic events** — invasions, force-tides, escort chains, and emergent objectives fire from world state, not a fixed timer; they're an [exploration](../progression/rails.md) draw.
- **Faction memory** — factions remember your bounties, your renown, your event participation. The world's stance toward you is *earned*, persisted, and consistent across sessions ([world-model](../../game-server/world-model/README.md)) — the direct answer to pillar #15.
- **All events & AI reactions are data** — new events and response rules ship with no recompile ([content-scripting](../../game-server/content-scripting/README.md)).

## Distilled from
| GTA system ([09](../../../initial-idea/09-gta6-inspiration.md)) | Our reimagining | Verdict |
|---|---|---|
| Witness-based police, response tiers, surrender | Guard/faction AI, bounty escalation, de-escalation | **Keep** — study the concept, original expression ([CLAUDE.md](../../../../CLAUDE.md)) |
| Weather/day-night affecting gameplay | Spawns, gathering, buffs, events by weather/time | **Keep** — legible & sim-able, never RNG punishment |
| In-game phone/feed | World-news / social feed | **Keep** — surfaces dynamic events in-client |
| Seamless session merge/split | Shard autoscaling under the reactive world | **Keep** — deep design pass before locking netcode ([world-model](../../game-server/world-model/README.md)) |
| Reactive ambient world | Dynamic events + faction memory | **Keep** — the core of pillar #15 |

## Rules
- **Guards are not omniscient** — witness-based detection, tiered response, a surrender path.
- **Crime has memory** — bounties & [renown](../progression/rails.md) persist; the world's stance is earned ([pvp](../endgame/pvp.md)).
- **Weather & time change gameplay** — spawns, yields, buffs, events; legible, never punitive RNG.
- **The world reads back** — a world-news feed surfaces dynamic events in-game.
- **Server-authoritative & persisted** — reactive state lives on the shard, seamless under autoscaling ([world-model](../../game-server/world-model/README.md)).
- **Original expression only** — study GTA's concepts, copy nothing verbatim ([CLAUDE.md](../../../../CLAUDE.md), [09](../../../initial-idea/09-gta6-inspiration.md)).
- **Events & reactions are data** — ship with no recompile ([content-scripting](../../game-server/content-scripting/README.md)).

## Links
[world-systems](README.md) · [professions](professions.md) · [housing](housing.md) · [09-gta6](../../../initial-idea/09-gta6-inspiration.md) · [pvp](../endgame/pvp.md) · [rails](../progression/rails.md) · [world-model](../../game-server/world-model/README.md) · [content-scripting](../../game-server/content-scripting/README.md) · [pillars](../00-design-pillars.md)
