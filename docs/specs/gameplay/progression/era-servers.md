# Era Servers — time can stop

> The signature feature: a world can **freeze at any milestone cap** and be that era's complete game, forever. A **60**. An **80** (peak [Wrath](ages.md)). A **150**. Up to **300**. You choose the era you want to *live in*, and no patch ever marches you out of it. → [pillars](../00-design-pillars.md) #3, #11

## The idea
WoW Classic proved players will pay to re-inhabit a fixed era — Vanilla (60), TBC (70), Wrath (80) — because a frozen world has a *completeness* the retail treadmill destroys: a finite gear ladder you can top out, a meta that stabilizes, a community that isn't scattered across catch-up mechanics. Blizzard offers exactly three frozen points and rotates even those. **We generalize it to every ten-level milestone and make it permanent.**

An **era server** is a mainline world with its clock stopped: level cap and unlocked [Ages](ages.md) pinned to a milestone, and *kept there*. It is not a lesser mode — it is the whole game as it stood at that summit.

## Milestone caps (freeze points)
Every Age boundary is a legal freeze: **60, 70, 80, 90, 100 … 300** ([ages](ages.md)). Named eras for the reimagined Ages:

| Cap | Era ("live in the world of…") | Unlocked Ages |
|---|---|---|
| 60 | *The Founding* | I |
| 70 | *The Shattered Gate* | I–II |
| **80** | *The Hollow King* — **the flagship era** ([pillars](../00-design-pillars.md) tonal north-star) | I–III |
| 90 | *The Sundering* | I–IV |
| 100 | *The Veiled Continent* | I–V |
| … | … | … |
| 170 | *The Long Night* (current-tier feature bar) | I–XII |
| … | … | … |
| 300 | *The Emberheart's Birth* (living mainline cap) | I–XXV |

## What "frozen" guarantees
- **The cap holds.** Characters top out at the era's level; no surprise increase drags the world forward.
- **The gear ladder is finite and toppable.** Its raid tiers are the whole ladder — you can *finish* your character ([itemization](../itemization.md), [pillars](../00-design-pillars.md) #13).
- **The world is preserved, not overwritten.** A `cap-60` server keeps the pre-[Sundering](ages.md) old world that Age IV would revamp ([pillars](../00-design-pillars.md) #3, #4 — the "old zones gone forever" loss, refused).
- **Balance is pinned** to that era's tuning pass; it does not drift under later Ages' rebalancing.
- **It's forever.** No sunset, no forced merge to the next era. Era servers don't get "seasonal" and deleted.

## How it works (data, not a fork)
A server's era is **content-manifest configuration** ([content-scripting](../../game-server/content-scripting/README.md)), never a code branch:
- `era.cap` — the pinned level cap (e.g. `80`).
- `era.ages` — unlocked Age range (e.g. `I..III`).
- `era.tuning` — the balance snapshot id for that milestone.
- Optional `era.progression: "seasonal"` — an operator *may* run a fresh-start era that unlocks Ages on a slow cadence toward its cap, but the default is a stable freeze.

Because [content is data](../../game-server/content-scripting/README.md) and the core is version-stable, one binary serves every era; operators pick a manifest ([modding](../../../initial-idea/06-modding-and-extensibility.md)). No `cargo build` to stand up a `cap-80` world next to a `cap-300` one.

## Characters & the warband
- A character is **bound to its era** — an 80-era character lives on 80-era worlds. This keeps each era's ladder honest (no 300 gear walking into an 80 world).
- The **[warband](rails.md)** (account-wide collections/cosmetics) spans eras where it can't unbalance them — appearances and titles travel; power does not.
- **Mainline → era is one-directional in power**: you don't import a maxed character to trivialize a frozen world.

## Why this is pro-player (the honest version)
Every reason a frozen era is good for the *player* is a reason it's bad for an *engagement-metrics* business — which is exactly why Blizzard rations it. A finite, completable, permanent world respects the player who wants to *master* something and then *keep* it, instead of being herded onto the next grind. That player is who we build for ([pillars](../00-design-pillars.md)).

## Rules
- **Era = manifest data.** Cap, Age range, tuning snapshot — never a code branch. One core binary, many eras.
- **Frozen means frozen.** No silent cap creep; no forced merge; no sunset.
- **Power never crosses eras**; cosmetics may. The ladder in each era stays honest.
- **Preserve, don't overwrite** — an early-cap era keeps the pre-revamp world that a later Age would replace.

## Links
[README](README.md) · [ages](ages.md) · [itemization](../itemization.md) · [rails](rails.md) · [content-scripting](../../game-server/content-scripting/README.md) · [pillars](../00-design-pillars.md)
