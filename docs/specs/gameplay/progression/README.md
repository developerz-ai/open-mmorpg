# Progression — 1 → 300

> The spine of the whole game: a **one-year journey** from level 1 to 300, structured as **25 Ages** of ten levels each, playable as one long climb *or* frozen at any milestone ([era-servers](era-servers.md)), and never a single grind because [rails](rails.md) run in parallel the whole way. → [pillars](../00-design-pillars.md) #1, #5, #10

## The shape of the climb
| | |
|---|---|
| **Level span** | 1 → 300. Never squished. The number only goes up ([pillars](../00-design-pillars.md) #10). |
| **Structure** | **25 Ages**, ten levels each. Age I is 1→60; Age II 60→70; … Age XXV 290→300 ([ages](ages.md)). |
| **Milestone caps** | 60, 70, 80, 90 … 300 — the boundary of each Age, and the freeze points for [era servers](era-servers.md). |
| **Time to cap** | **~1 year** of unhurried mainline play for a mainline player; far less if you sprint, far more if you wander the [rails](rails.md). Deliberate ([pillars](../00-design-pillars.md) #1). |
| **The feel** | Wrath-era pacing — leveling *is* the game, not a tutorial you rush to delete. |

## XP curve & pacing philosophy
- **Front-loaded onboarding, long tail.** Age I (1→60) is brisk and teaches the game — the "Vanilla" journey, tuned to hook, ~2–3 weeks casual. Each later Age lengthens: the world is bigger, the story deeper, the [renown](rails.md) richer.
- **No dead levels.** Every level grants *something* — an ability, a talent point, a rail unlock, a stat breakpoint. WoW's "levels 70–79 are filler" problem is a content-density failure we refuse.
- **Rested & catch-up are generous, account-wide.** Alts and returners ride [warband](rails.md) bonuses; the point is to keep *playing*, not to punish a gap ([pillars](../00-design-pillars.md) #8, #9).
- **Bring-a-friend mentoring** — a high-level player can sync down to a friend's Age and earn full rewards, so the year-long climb is never lonely.

## Why a year, and why that's the good version
Blizzard's cap is reachable in a long weekend, so leveling became a vestigial hallway to the "real" game at max. That is the insult in [pillars](../00-design-pillars.md) #1. We invert it: **the journey is the game.** A year sounds long only if it's one treadmill — so it isn't. At any moment you can:
- push the **main rail** (level, zone story, dungeons),
- or ride a **side rail** ([rails](rails.md)) — professions, renown, PvP ranks, delve mastery, housing, collections, exploration —
each granting real, permanent progress. The year is a *life in the world*, self-paced across many tracks.

## Power growth (permanent, additive)
Every Age adds character power that is **never confiscated** ([pillars](../00-design-pillars.md) #1):
- **Levels** → base stats, ability unlocks.
- **Talent points** → [trees + hero trees](../classes/talents-and-hero-trees.md), re-specced freely, never reset by an Age boundary.
- **Gear** → [itemization](../itemization.md), with the 30-level relevance window so it *matters* across Ages.
- **Renown / mastery rails** → small permanent account-wide bonuses ([rails](rails.md)).

No borrowed-power system. Nothing here is a rented mechanic you'll grind and lose.

## How this maps to servers
- A **mainline server** runs the full 1→300 (the official flagship service and any operator who wants the whole climb).
- An **era server** freezes at a chosen milestone — a 60, an 80 ("peak Wrath"), a 150 — and *is* that era's complete game forever ([era-servers](era-servers.md)).
- Level cap and unlocked Ages are **content-manifest values** ([content-scripting](../../game-server/content-scripting/README.md)) — flipping a server's era is data, not a rebuild.

## Rules
- **The number never decreases.** No squishes, no retro-renumbering.
- **No dead levels** — every level grants a mechanically or narratively meaningful reward.
- **No borrowed power** — all progression is permanent and additive.
- **Cap & Ages are data** — a server's era is a manifest value, editable without recompiling core.

## Links
[ages](ages.md) · [era-servers](era-servers.md) · [rails](rails.md) · [itemization](../itemization.md) · [talents](../classes/talents-and-hero-trees.md) · [pillars](../00-design-pillars.md)
