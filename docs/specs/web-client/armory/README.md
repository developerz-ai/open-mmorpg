# Armory

> Public character & guild lookup. **Read-only projections** from [`apps/worldsvc`](../../../../apps/worldsvc): character sheet, gear, talents, achievements, guild roster. Zod-validated, query-cached, i18n'd, SSR for SEO. It **mirrors** server projections — never a source of truth. WoW's Armory + the community sites, typed and operator-controlled.

## What it does
Renders a browsable, shareable view of what the world already knows about a character or guild. Public character pages are **SSR** ([SolidStart](../app-shell/README.md)) so they're crawlable and link-previewable; the logged-in deep views live in the SPA. All data is a projection served by `worldsvc` across shards — the web computes nothing about game state.

## Surfaces
| Surface | Projection source | Rendering |
|---|---|---|
| **Character sheet** | worldsvc character projection | SSR public page (SEO), SPA detail |
| **Gear / itemization** | worldsvc item projection | read-only, item tooltips via [design-system](../design-system/README.md) |
| **Talents / build** | worldsvc spec projection | read-only |
| **Achievements** | worldsvc achievement projection | paginated query |
| **Guild roster** | worldsvc guild projection | roster list + member links |

## Design
- **Read-only, always.** No armory endpoint mutates game state; the web submits no intents here. It is a projection viewer.
- **Zod at the boundary** — every projection parsed to a typed shape before render (per [`realm.ts`](../../../../apps/web/src/lib/realm.ts)); a shape drift fails loud, not silently.
- **TanStack Query** caches reads keyed by character/guild id; stale-while-revalidate keeps public pages fast without holding truth.
- **SSR for public pages** — server-render for SEO and social previews; the SPA hydrates for interactivity.
- **Operator toggle: armory public/private** — a [feature flag](../operator-brand/README.md). Private → pages 404/gate; the flag is enforced server-side, the web just respects it.
- **i18n** — stat names, tooltips, section headers via `t()`.

## Distilled from
| Source | Lesson | Our verdict |
|---|---|---|
| WoW Armory (official character site) | A first-party, crawlable character projection is the right shape — but it must be an authoritative-server read, not a scraped snapshot | **Keep** the concept, **fix** the plumbing — typed worldsvc projection, cached, SSR |
| Community armory/scraper sites | Third-party scrapes drift, break, and become a de-facto (wrong) source of truth | **Replace** with a first-party cached read API the operator owns and toggles |

## Rules
- **Read-only projection** — the armory never writes or mutates game state, never a source of truth.
- **Zod-validate every projection** before render; drift fails loud.
- **Server state via TanStack Query** — no hand-rolled cache.
- **SSR public character pages** for SEO; SPA for logged-in depth.
- **Armory public/private is an operator flag**, enforced server-side.
- **Every string via `t()`** — including stat and tooltip labels.

## Links
[app-shell](../app-shell/README.md) · [data-layer](../data-layer/README.md) · [design-system](../design-system/README.md) · [operator-brand](../operator-brand/README.md) · [i18n](../i18n/README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [`apps/worldsvc`](../../../../apps/worldsvc) · [`realm.ts`](../../../../apps/web/src/lib/realm.ts) · [CLAUDE.md](../../../../CLAUDE.md)
