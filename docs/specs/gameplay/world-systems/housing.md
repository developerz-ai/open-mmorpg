# Housing — a place you choose to live

> Player housing is the **redemption of the [Age VI](../progression/ages.md) garrison**. The garrison trapped you indoors and made a solo chore-box mandatory ([pillars](../00-design-pillars.md)); housing inverts every part of that — **opt-in, social, expressive, never required**. It matures at [Age XII *The Long Night*](../progression/ages.md) (the *Midnight*-tier feature bar we match, [feature-bar](../../../initial-idea/08-feature-bar.md)). Homes and decor are a [rail](../progression/rails.md), account-wide via **[warband](../progression/rails.md)**. → seamless & server-authoritative ([world-model](../../game-server/world-model/README.md)).

## What it is
| Piece | What you get |
|---|---|
| **Plot ownership** | A durable, owned plot in the world — bought & upgraded on the [housing rail](../progression/rails.md) |
| **3D placement tools** | Free-form placement: move/rotate/scale/snap decor and structure; original toolset, open formats ([CLAUDE.md](../../../../CLAUDE.md)) |
| **Customization** | Walls, roofs, gardens, lighting, themes — build the home, not just pick a preset |
| **Decor collection** | A **[warband](../progression/rails.md) wardrobe** of decor earned across the world — a collection rail, pure prestige ([pillars](../00-design-pillars.md) #11) |
| **Neighborhoods** | Shared, seamless clusters of plots — neighbors, communal spaces, standing you build with them |

## Opt-in, not the garrison (the fix)
| Garrison ([Age VI](../progression/ages.md)) | Housing ([Age XII](../progression/ages.md)) |
|---|---|
| **Mandatory** — the progression path ran through it | **Opt-in** — a place you *choose* to go, zero power tax |
| **Isolating** — a solo instance that pulled you *out* of the world | **Social** — shared **neighborhoods**, seamless in the world |
| **A chore-box** — daily missions, follower tables | **Expressive** — placement freedom, decor as a hobby |
| Prefab, identical for everyone | Yours — built with free 3D tools |
| Character-bound busywork | **[Warband](../progression/rails.md)** account-wide home & decor |

**Why it ships:** it answers *"does this make time feel well-spent, or long?"* ([pillars](../00-design-pillars.md)) with the first — a home is a reason to log in for joy, never an obligation.

## Rail & account-wide
- **Housing is a [rail](../progression/rails.md)** — plot upgrades, decor, and neighborhood standing are permanent, additive progress; **none of it is mandatory** for raid-readiness.
- **Warband-wide** — your home and decor **wardrobe** belong to the account, earned once, shared across characters ([pillars](../00-design-pillars.md) #9).
- **Effort-metered** — decor and upgrades come from doing the world's activities, not a weekly reset ([pillars](../00-design-pillars.md) #8).

## Server-authoritative & seamless
- Plots, neighborhoods, and placed decor are **server state** — authored, owned, and persisted server-side; the client renders intent ([world-model](../../game-server/world-model/README.md)).
- **Plot ownership is a [persistence](../../game-server/economy/README.md) transaction** — the anti-dupe rule applies to homes and decor as to gear ([CLAUDE.md](../../../../CLAUDE.md), [economy](../../game-server/economy/README.md)).
- Neighborhoods are **seamless** — no loading wall between the world and your street; shards merge/split under it ([world-model](../../game-server/world-model/README.md)).
- **Decor & plots are data** — new themes and neighborhood layouts ship with no recompile ([content-scripting](../../game-server/content-scripting/README.md)).

## Distilled from
| Source | System | Verdict |
|---|---|---|
| WoW WoD garrison ([Age VI](../progression/ages.md)) | Personal stronghold, mandatory & solo | **Fix** — make it opt-in, social, powerless |
| WoW *Midnight* housing ([Age XII](../progression/ages.md)) | Player homes, neighborhoods, decor | **Keep** placement freedom & neighborhoods |
| Player-housing genre (sandbox builders) | Free-form 3D placement, plot economies | **Keep** the expressive toolset; server-authoritative |
| FOMO cosmetic vaulting | Limited decor removed forever | **Fix** — decor stays *earnable*, warband-wide ([pillars](../00-design-pillars.md) #11) |

## Rules
- **Opt-in, never mandatory** — housing grants **zero** raid power; it's a home, not a garrison ([pillars](../00-design-pillars.md)).
- **Housing is a [rail](../progression/rails.md)** — permanent, additive, **warband** account-wide.
- **Social by default** — shared, seamless **neighborhoods**, not solo instances.
- **Server-authoritative** — plots/decor are persisted server state; ownership is one [persistence](../../game-server/economy/README.md) transaction ([CLAUDE.md](../../../../CLAUDE.md)).
- **Open formats only** — glTF decor, no proprietary tooling ([CLAUDE.md](../../../../CLAUDE.md)).
- **All decor & layouts are data** — ships with no recompile ([content-scripting](../../game-server/content-scripting/README.md)).

## Links
[world-systems](README.md) · [professions](professions.md) · [living-world](living-world.md) · [rails](../progression/rails.md) · [ages](../progression/ages.md) · [world-model](../../game-server/world-model/README.md) · [economy](../../game-server/economy/README.md) · [content-scripting](../../game-server/content-scripting/README.md) · [feature-bar](../../../initial-idea/08-feature-bar.md) · [pillars](../00-design-pillars.md)
