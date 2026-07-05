# App Shell — SSR/SPA split, thin routes

> The **skeleton every operator site forks**: a **SolidStart SSR** half for public/SEO surfaces (landing, blog, downloads) and a **Solid + Vite SPA** for the logged-in app (account, armory, AH). Two modes, one design system, one API contract. Routes stay thin — parse params → call a query hook → render. → [architecture/09](../../../architecture/09-operator-web.md).

## Two modes, one boundary
The split is about **who needs to see the HTML**: a crawler / first-paint marketing visitor, or an authenticated player already in-app.

| | SSR half (SolidStart) | SPA half (Solid + Vite + `@solidjs/router`) |
|---|---|---|
| Surfaces | Landing, blog, downloads, patch notes | Account, [armory](../armory/README.md), [auction house](../auction-house/README.md), [world feed](../world-feed/README.md) |
| Why | **SEO** — server-rendered HTML, meta tags, no JS-to-content | **Interactivity** — authed, stateful, no crawl value |
| Rendered | Per-request on the server, hydrated | Client-only, static shell + JS bundle |
| Data | Public projections, cached hard | [TanStack Query](../data-layer/README.md), authed + live |
| Auth | None (or a "log in" CTA) | Session-gated ([account-auth](../account-auth/README.md)) |

**Rule of thumb:** does Google need to index it, or does it change per logged-in user? First → SSR. Second → SPA. Never SSR an authed, per-user view; never make a marketing page depend on client JS to show its copy.

## Thin routes (SRP)
A route is a **wiring file**, not a place for logic. It parses params, calls a query hook, renders a component — nothing else. Fetch + validation live in the [data-layer](../data-layer/README.md) hook; strings via [`t()`](../i18n/README.md); markup from [`packages/ui`](../design-system/README.md).

```
route (parse params) → query hook (fetch + Zod) → component (render 3 states)
```

If a route grows branching data logic, it's wrong — lift it into a hook. Files ≤300 LOC, one job each. See [`RealmStatusCard`](../../../../apps/web/src/components/RealmStatusCard.tsx) for the shape: component renders loading/error/data, hook owns the fetch.

## Layout & code-splitting
- **Nested layouts** — shared chrome (nav, footer, brand header) at the layout level, page content in the `<Outlet>`. One nav, not N copies.
- **Route-level code-splitting** — each SPA route is a lazy chunk; the armory bundle never ships to a visitor reading the blog.
- **Brand slot** — realm name / logo / palette resolve once at the shell from [operator config](../operator-brand/README.md), flow down as tokens. No page hard-codes a realm name.

## Distilled from
| Reimagines | Keep | Fix |
|---|---|---|
| Monolithic PHP fansite + control panel (one process renders everything) | The **surface set** operators expect (landing + account + armory) | Split by SEO-vs-authed, not one blob; **typed** routes, not string-templated pages |
| Server-rendered-everything (slow authed pages) | SSR **where crawlers need it** | SPA for the authed app — no SSR cost on per-user views |
| SPA-everything (blank-page SEO) | Rich client interactivity | SSR the marketing pages so content is in the HTML |

## Rules
- **SEO/public → SSR (SolidStart). Authed/interactive → SPA (Vite + `@solidjs/router`).** Don't cross the streams.
- **Routes are thin** — parse → hook → render. Data logic lives in [query hooks](../data-layer/README.md), never in the route.
- **Shared chrome in a layout**, page content in the outlet. One nav/footer definition.
- **Lazy-load routes** — authed bundles never ship to public visitors.
- Files ≤300 LOC, one component one job. Strings via [`t()`](../i18n/README.md), colors via [tokens](../design-system/README.md), never hex.
- The shell holds **no game truth** — it renders projections and submits intents to [gateway](../../../../apps/gateway)/[worldsvc](../../../../apps/worldsvc).

## Links
[design-system](../design-system/README.md) · [i18n](../i18n/README.md) · [data-layer](../data-layer/README.md) · [account-auth](../account-auth/README.md) · [operator-brand](../operator-brand/README.md) · [testing-dx](../testing-dx/README.md) · [index](../README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [CLAUDE.md](../../../../CLAUDE.md)
