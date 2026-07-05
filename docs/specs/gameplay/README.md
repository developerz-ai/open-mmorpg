# 🎲 Gameplay Specs

> **Scope:** the *game* — what a player does, from character creation to level 300 and the endgame that outlives it. The [game-server](../game-server/README.md) folder says *how the runtime behaves*; this folder says *what the runtime is running*: factions, races, classes, progression, itemization, endgame, world systems. All content here is **data** ([content-scripting](../game-server/content-scripting/README.md)) — none of it recompiles the core ([CLAUDE.md](../../../CLAUDE.md)).

## The thesis
We take twenty years of MMO design decisions — Vanilla through the current WoW *Midnight* tier — and re-decide every one of them **for the player instead of the metrics dashboard**. Blizzard optimized for engagement-time and subscription churn; we optimize for the *feeling of a life well-lived in a world*. Same genre, opposite incentive. Every reversal is logged in [00-design-pillars](00-design-pillars.md). Original IP only — mechanics aren't copyrightable, expression is ([legal](../../initial-idea/01-legal-and-licensing.md)).

**The feel is Wrath-of-the-Lich-King** — deliberate combat, a world with weight, a story that earns its ending — **built on the newest systems** (hero talents, delves, account-wide warbands, skyriding, modern group finder) with none of the borrowed-power treadmills that came after.

## The three headline decisions
| Decision | What it means | Reverses |
|---|---|---|
| **Level cap 300, ~1 year to earn it** | 25 *Ages* of ten levels each. A real journey, not a weekend to catch up. Parallel [rails](progression/rails.md) mean the year is never one grind. | The modern "dinged max in 4 days, now the *real* game starts" insult. |
| **Era servers — time can stop** | Any operator (or the official service) can freeze a world at a milestone cap: a **60**, an **80** (peak Wrath), a **150**, up to **300**. Pick the era you want to live in. | Being force-marched onto every new patch, with old worlds deleted. |
| **Raid gear stays great for 30 levels** | A tier's loot is best-in-slot-viable across **three Ages** — your raid effort matters for a season, not a fortnight. | Purple raid drops replaced by quest greens in the next zone's first hour. |

## Canon (single source of truth — every doc below inherits these names)
| Concept | Canon | WoW-archetype it reimagines |
|---|---|---|
| **World** | **Auralon**, a living world with a dormant world-soul, the **Emberheart**, at its core | Azeroth / the Worldsoul |
| **Endgame meta-saga** | **the Emberheart Saga** — the slow waking of the world-soul across the Ages | Worldsoul Saga |
| **Faction I** | **the Aurelian Concord** ("the Concord") — order, cities, dawn & arcane, a pluralist coalition | Alliance |
| **Faction II** | **the Wildreach Pact** ("the Pact") — freedom, frontier, spirits & the primal, a confederation of the untamed | Horde |
| **Cosmic forces (6)** | **Lattice** (order/arcane) · **Rift** (chaos/ruin) · **Bloom** (life) · **Hollow** (death) · **Dawn** (radiance) · **Deep** (void) | Order · Fel · Life · Death · Light · Void |
| **Level span** | **1 → 300**, in **25 Ages** of 10 levels (Age I ends at 60) | 11 expansions, cap-per-expansion |
| **Milestone caps** | 60, 70, 80, 90 … 300 — each an era-server freeze point | Classic-era servers, generalized |

## Index
| Doc | Covers |
|---|---|
| [00-design-pillars](00-design-pillars.md) | The gamers-first manifesto — every Blizzard decision we reverse, and why |
| [01-world-and-cosmology](01-world-and-cosmology.md) | Auralon, the Emberheart, the six cosmic forces, the Saga |
| [factions/](factions/README.md) | The two-faction model · [Concord](factions/concord.md) · [Pact](factions/pact.md) + their races |
| [classes/](classes/README.md) | The class roster, roles, resources · [talents & hero trees](classes/talents-and-hero-trees.md) |
| [progression/](progression/README.md) | 1→300 arc · [the 25 Ages](progression/ages.md) · [era servers](progression/era-servers.md) · [rails](progression/rails.md) |
| [itemization](itemization.md) | Item model, the 30-level relevance window, stats, no-green-reset |
| [endgame/](endgame/README.md) | [raids](endgame/raids.md) · [dungeons](endgame/dungeons.md) · [delves](endgame/delves.md) · [pvp](endgame/pvp.md) |
| [world-systems/](world-systems/README.md) | [professions](world-systems/professions.md) · [housing](world-systems/housing.md) · [living world](world-systems/living-world.md) |

## Non-negotiables (inherited → [CLAUDE.md](../../../CLAUDE.md))
1. **Content is data.** A new race/class/Age/raid ships with no `cargo build`. If a gameplay change needs a recompile, it belongs in a crate, not here.
2. **Server-authoritative.** Every rule in these docs is enforced on the shard; the client renders intent ([game-server](../game-server/README.md)).
3. **Two-faction reference, N-faction capable.** We ship 2; the count is data ([modding](../../initial-idea/06-modding-and-extensibility.md)).
4. **Original IP only.** No ripped name, table, zone, or asset — ever ([legal](../../initial-idea/01-legal-and-licensing.md)).

## Distilled from
Vanilla → *Midnight* WoW (every expansion's systems, kept or fixed — see [ages](progression/ages.md)) · WoW **Classic/era** progression servers (the freeze-in-time idea, generalized to every cap) · **GTA** living-world systems ([living world](world-systems/living-world.md), [09](../../initial-idea/09-gta6-inspiration.md)) · two decades of player-side post-mortems on what respected their time and what didn't.

> Each doc is ≤ ~1 screen: what it is, the design, what it reimagines, the rules, links. Grows past that → split it. Same SRP rule as code.
