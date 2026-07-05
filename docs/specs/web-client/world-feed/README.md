# World Feed

> The in-browser world news / social feed — surfaces **dynamic world events** from the [living world](../../gameplay/world-systems/living-world.md): boss kills, faction shifts, bounties posted, world-boss spawns, server milestones. A first-party live feed that replaces alt-tabbing to external fansite trackers. **Read-only projection** from [`worldsvc`](../../../../apps/worldsvc) — the web mirror of the in-game feed.

## What it does
Renders the reactive world as a scrollable, shareable news stream. It's the same event stream the [in-game world-news feed](../../gameplay/world-systems/living-world.md#world-news--social-feed) shows, projected to the operator site so players (and prospects) can read what's happening without logging in. Each item is an SSR-shareable permalink for social previews.

## Event sources ([living world](../../gameplay/world-systems/living-world.md))
| Event | Origin | Feed item |
|---|---|---|
| **Boss / world-boss kill** | shard combat + world state | "⟨guild⟩ felled ⟨boss⟩" |
| **World-boss spawn** | dynamic world event | spawn alert, zone link |
| **Faction-front shift** | faction/living-world state | control-change banner |
| **Bounty posted** | witness-based bounty ladder | wanted notice, target link |
| **Server milestone** | worldsvc aggregate | first-clear, population, event-season marks |

## Design
- **Read-only projection** — the feed submits no intents and mutates nothing; it visualizes worldsvc's event stream.
- **Zod at the boundary** — each event parsed to a typed, discriminated `FeedItem` union (per [`realm.ts`](../../../../apps/web/src/lib/realm.ts)) before render; an unknown variant degrades gracefully, never crashes the stream.
- **TanStack Query, polled** — the feed refetches on an interval (query polling), stale-while-revalidate; no bespoke socket layer for v1.
- **SSR-shareable permalinks** — public feed and per-event pages server-render for SEO and link previews; SPA hydrates for live polling.
- **i18n + `Intl`** — event templates via `t()` with interpolation; timestamps/counts via `Intl`.
- **Operator-toggleable** — feed on/off and which event classes surface are [flags](../operator-brand/README.md).

## Distilled from
| Source | Lesson | Our verdict |
|---|---|---|
| GTA-style in-world social feed ([living world](../../gameplay/world-systems/living-world.md)) | An in-world event stream is how a player *reads* the reactive world — surface it on the web too | **Keep** — first-party feed mirroring the in-game one |
| External WoW event/rare-spawn tracker sites | Players alt-tab to third-party trackers for world state; scraped, laggy, unofficial | **Replace** with a first-party live feed off the authoritative worldsvc stream |

## Rules
- **Read-only projection** — mirrors the in-game feed; submits nothing, mutates nothing.
- **Zod-validate every event**; unknown variants degrade gracefully.
- **Server state via TanStack Query** (polled) — no hand-rolled cache or socket for v1.
- **SSR-shareable** per-event permalinks for SEO and social previews.
- **Every string via `t()`**; times/counts via `Intl`.
- **Feed + event classes are operator flags** ([operator-brand](../operator-brand/README.md)).

## Links
[living-world](../../gameplay/world-systems/living-world.md) · [data-layer](../data-layer/README.md) · [app-shell](../app-shell/README.md) · [operator-brand](../operator-brand/README.md) · [i18n](../i18n/README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [`apps/worldsvc`](../../../../apps/worldsvc) · [`realm.ts`](../../../../apps/web/src/lib/realm.ts) · [CLAUDE.md](../../../../CLAUDE.md)
