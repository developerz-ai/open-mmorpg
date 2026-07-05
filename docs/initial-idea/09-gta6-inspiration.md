# 09 — GTA-Inspired Living-World Systems

> Nothing stops a WoW-like from adopting GTA's best world systems. Original expression only — mechanics aren't copyrightable ([01](01-legal-and-licensing.md)). Do **not** use any patented mechanism verbatim; study the *concept*.

## Transferable features
| GTA system | Our mapping | Priority |
|---|---|---|
| Reactive NPC/police AI: crime severity → response tiers, **witness-based** detection (not omniscient), surrender option | Guard/faction-reputation AI reacting to PvP crimes in cities; witnesses, bounties, escalation | High |
| Dynamic weather + day/night affecting **gameplay**, not just visuals | Ties into world events, spawns, gathering yields, buffs | Medium |
| In-world social layer (in-game phone/feed of world events) | In-game **"world news / social" feed** surfacing dynamic events — replaces external sites | Medium |
| Session merge/split into one persistent world without loading | Directly relevant to **shard autoscaling** — study the concept for seamless merge/split ([02](02-architecture-and-scaling.md)) | High |
| Interactive world businesses; fishing/hunting/scuba as activities | Non-combat MMO content: professions/gathering, ownable world businesses | Medium |
| Physical stat/condition (workout → build) | Optional profession/exercise-style stat systems | Low |
| Dual-protagonist character-switch | Skip — MMO is single-character-per-slot | — |

## Most valuable for a bulletproof core
The **seamless session merge/split** concept. It is the same problem our shard-autoscaling solves; worth a deep design pass before locking the netcode/sharding model.
