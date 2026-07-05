# 00 — Design Pillars (gamers-first)

> Blizzard did not lack talent; it lacked the right incentive. A subscription + engagement business rewards *time spent*, so the systems evolved to consume time — even at the cost of fun. We have no such incentive (MIT, open-core, no sub, no cash shop mandate). So we can make the honest choice at every fork. This doc is the list of forks and the choice we make.

## The prime directive
**Respect the player's time, intelligence, and attachment to the world.** Every system is judged against those three. If a mechanic exists to inflate a metric rather than reward the player, it is cut.

## The reversals (their decision → ours)
| # | What Blizzard shipped | Why players hated it | Our decision |
|---|---|---|---|
| 1 | **Borrowed power** (Artifact / Azerite / Covenant / soulbinds) reset every expansion | You grind a system to full, then it's deleted and you start over | **Power is permanent & additive.** Everything you earn stays earned. No system is confiscated at an Age boundary ([classes](classes/README.md), [ages](progression/ages.md)) |
| 2 | **Gear reset**: raid epics → quest greens in the next zone | Your best effort obsoleted in an hour | **30-level relevance window** — raid gear is BiS-viable for 3 Ages ([itemization](itemization.md)) |
| 3 | **Forced onto the newest patch**; old content nerfed/removed | The version you loved is gone forever | **Era servers** freeze any milestone cap, forever playable ([era-servers](progression/era-servers.md)) |
| 4 | **Faction segregation** for ~15 years | Couldn't play with friends across the divide | **Cross-faction grouping from day 1.** Factions are an identity & story, not a prison ([factions](factions/README.md)) |
| 5 | **Daily/weekly lockout as the *only* path** to power | Miss a day, fall behind; play becomes a chore-list | **Parallel [rails](progression/rails.md)** — many routes to progress; no single mandatory treadmill |
| 6 | **Illusion-of-choice talents** (one correct build) | "Choice" that a sim solves for you | **Genuinely divergent trees** with situational, not strictly-dominant, nodes ([talents](classes/talents-and-hero-trees.md)) |
| 7 | **Pathfinder** — flying gated behind grind, patches late | The world's best traversal held hostage | **Skyriding learned early**, in the Age it's introduced; no meta-achievement gate ([world-systems](world-systems/README.md)) |
| 8 | **Timegating & artificial rep grinds** (revered-at-1-day-a-week) | Progress metered by the calendar, not effort | Effort-metered [renown](progression/rails.md); catch-up is generous and account-wide |
| 9 | **Alt-hostile** account design | Every alt re-grinds the same unlocks | **Warband account-wide** unlocks: collections, renown, currencies, knowledge shared ([rails](progression/rails.md)) |
| 10 | **Level squishes** (120→50→70…) | Your number, your milestones, retconned | **The number only ever goes up.** 1→300, no squish, ever ([progression](progression/README.md)) |
| 11 | **FOMO cosmetics** — limited mounts/titles removed forever | Anxiety-driven login, not joy | Prestige stays *earnable*; nothing desirable is time-vaulted out of reach |
| 12 | **Pay-to-skip** (level boosts, token→gold→carry) | Undermines the journey it sells | No boosts. The journey is the product; you can't buy past it |
| 13 | **RNG-on-RNG** loot (titanforging, tertiary lottery) | Best-in-slot from a slot machine, never "done" | **Deterministic targetable loot** + bad-luck protection; a set you can *finish* ([itemization](itemization.md)) |
| 14 | **Class/race lock** matrix | Arbitrary "you can't be that" walls | Minimal restriction — nearly any race plays nearly any class; lore-flavored, not hard-walled ([classes](classes/README.md)) |
| 15 | **Reused/instanced "world"** that ignores you | The world resets; you never mattered | **Reactive living world** — factions, guards, weather, events remember and respond ([living-world](world-systems/living-world.md)) |

## What we deliberately keep (their decisions worth honoring)
- **Wrath-era combat weight** — a global cooldown you feel, rotations that breathe, healing that's about triage not whack-a-mole.
- **The holy trinity** (tank/heal/dps) — legible roles beat soup.
- **Bring-the-player-not-the-class** — no roster held hostage to one buff.
- **Modern QoL that *is* pro-player**: dungeon/raid finder as an *option*, transmog wardrobe, account-wide collections, delves for the solo player, hero talents as permanent flavor.
- **Mythic+ style scalable dungeons** and **cross-realm tech** — good engineering, kept ([dungeons](endgame/dungeons.md)).

## The test we apply to every new system
> *"Does this exist to make the player's time feel well-spent, or to make it feel long?"* If the honest answer is the second, it doesn't ship. Write the answer down in the doc that proposes the system.

## Links
[README](README.md) · [progression](progression/README.md) · [itemization](itemization.md) · [era-servers](progression/era-servers.md) · [rails](progression/rails.md) · [feature-bar](../../initial-idea/08-feature-bar.md)
