# Itemization

> Gear is the currency of accomplishment. Blizzard debased it two ways — obsoleting raid epics in an hour ([pillars](00-design-pillars.md) #2) and turning best-in-slot into a slot-machine you can never finish ([pillars](00-design-pillars.md) #13). We restore both: a **30-level relevance window** and **deterministic, finishable loot**. → [economy](../game-server/economy/README.md) for the transactional plumbing.

## The item model
- **Item level (iLvl)** is the single power scalar; slot budgets distribute it across [stats](#stats). Deterministic, legible, sim-able.
- **Newtyped** end to end (`ItemId`, `ItemLevel`) — no raw primitives ([CLAUDE.md](../../../CLAUDE.md)).
- **Definitions are [data](../game-server/content-scripting/README.md)** — new items ship without a recompile.
- **Ownership lives only in a [persistence](../game-server/persistence/README.md) transaction** — the anti-dupe rule; loot/craft/trade are transactional & idempotent ([economy](../game-server/economy/README.md)).

## The 30-level relevance window (the headline)
A raid tier's gear is **best-in-slot-viable for three Ages** — the Age it drops in, plus the two after. Concretely: gear from the Age-N raid ([raids](endgame/raids.md)) out-performs Age-(N+1) and Age-(N+2) *leveling* drops and holds entry-raid viability, only clearly surpassed by the Age-(N+3) raid tier.

**Why:** your raid effort should matter for a *season*, not a fortnight. The old model — purple drops replaced by the next zone's quest greens in the first hour — is the archetypal disrespect of player time ([pillars](00-design-pillars.md) #2). Thirty levels of relevance makes a raid clear feel *earned and durable*.

**How the math holds (no runaway power):**
- **Flattened iLvl curve.** Per-level stat growth is gentle enough that a 30-level lead in *quality* (raid vs. quest) outweighs a 30-level deficit in *level*. Leveling greens close the gap slowly, not instantly.
- **Sidegrade, not obsolete.** The next Age's leveling gear offers *different* stat profiles and set flavors — reasons to swap pieces situationally — without strictly dominating last Age's raid set.
- **Catch-up floors, doesn't leapfrog.** A returning/alt player gets gear that makes them *raid-ready for the current tier*, but not gear that trivializes the three-Age window others earned.

## No green reset, no gear treadmill whiplash
- Entering a new Age does **not** vaporize your gear ([pillars](00-design-pillars.md) #2). You upgrade *pieces* as you go, keeping strong slots across the window.
- Because [era servers](progression/era-servers.md) exist, a tier's ladder is genuinely *finishable* — you can top out and *stay* topped out, not chase a moving titanforge target.

## Deterministic, finishable loot (reverse the slot machine)
| Their RNG-on-RNG | Ours |
|---|---|
| Titanforging / warforging — random iLvl bumps | Fixed iLvl per source. What drops is what it is. |
| Tertiary/socket lottery on every drop | Sockets/tertiaries are **crafted or targeted**, not gambled |
| Personal-loot black box | Choice: group loot or **targeted vendor/currency** paths |
| No pity | **Bad-luck protection** — a currency accrues toward a guaranteed pick |

The result: a set you can **plan and complete** ([pillars](00-design-pillars.md) #13). Crafting ([professions](world-systems/professions.md)) bridges gaps deterministically — the [rail](progression/rails.md) that guarantees you're never wall-blocked by a drop that won't come.

## Stats (legible, Wrath-clean)
- **Primary:** one per role archetype (e.g. Might / Finesse / Focus / Spirit — final names in [classes](classes/README.md)) + Stamina.
- **Secondary:** a small, tuned set (crit, haste, mastery-equivalent, versatility-equivalent) — enough for build texture, few enough to reason about. No reforging needed; if a stat's bad, we fix the stat, not add a reforge tax ([pillars](00-design-pillars.md)).
- **No stat squish churn** — the [no-squish rule](progression/README.md) applies to items too; numbers grow, they don't get retro-renumbered.

## Sources & the gear ladder
Every [rail](progression/rails.md) feeds gear at a coherent iLvl band, so no single activity is the only path:
| Source | Band | Notes |
|---|---|---|
| Leveling / world | baseline | Fills slots on the way up |
| [Dungeons](endgame/dungeons.md) (incl. scaling/Mythic+) | above world | Repeatable, scales with key level |
| [Raids](endgame/raids.md) | top of the tier | The 30-level relevance anchor |
| [Delves](endgame/delves.md) | dungeon-adjacent | Solo/small-group path to raid-adjacent gear |
| [PvP](endgame/pvp.md) | parallel ladder | Own currency; scaled in instanced PvP |
| [Professions](world-systems/professions.md) | fills & bridges | Deterministic crafted BiS-adjacent pieces |

## Rules
- **30-level relevance window** — raid gear is BiS-viable across three Ages; no quest-green obsoletion.
- **Deterministic loot** — fixed iLvl, targetable, bad-luck-protected. No titanforging, no tertiary lottery.
- **A tier's ladder is finishable** — you can top out and stay there (essential for [era servers](progression/era-servers.md)).
- **Ownership = one [persistence](../game-server/persistence/README.md) transaction.** Loot/craft/trade transactional & idempotent — never through cache ([economy](../game-server/economy/README.md)).
- **Items are data**; stats are few and legible; the number only goes up.

## Links
[progression](progression/README.md) · [raids](endgame/raids.md) · [professions](world-systems/professions.md) · [economy](../game-server/economy/README.md) · [persistence](../game-server/persistence/README.md) · [pillars](00-design-pillars.md)
