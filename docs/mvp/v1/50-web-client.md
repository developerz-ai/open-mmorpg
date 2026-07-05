# 50 — Web Client · Wave 3

> **Big idea.** The **operator web portal** every server operator hosts: marketing/landing, account & registration, realm status, armory, auction browser, world feed, downloads. The **only non-Rust part of the stack** — Bun + SolidJS, a deliberate deviation because it's a standard web product, not a hot path. Reusable app framework + this game's portal surfaces + per-operator brand by config. **Owner: [`webclient`](../../../.claude/agents/webclient.md).** → [web-client specs](../../specs/web-client/README.md) · [09 operator-web](../../architecture/09-operator-web.md)

## Reads
[web-client README](../../specs/web-client/README.md) and each subsystem: [app-shell](../../specs/web-client/app-shell/README.md) · [design-system](../../specs/web-client/design-system/README.md) · [i18n](../../specs/web-client/i18n/README.md) · [data-layer](../../specs/web-client/data-layer/README.md) · [account-auth](../../specs/web-client/account-auth/README.md) · [armory](../../specs/web-client/armory/README.md) · [auction-house](../../specs/web-client/auction-house/README.md) · [world-feed](../../specs/web-client/world-feed/README.md) · [operator-brand](../../specs/web-client/operator-brand/README.md) · [testing-dx](../../specs/web-client/testing-dx/README.md) · [09 operator-web](../../architecture/09-operator-web.md).

## Depends on
[Foundation](00-foundation.md) (repo, CI); [Server](30-game-server.md) gateway (auth, realm status) + worldsvc (armory, AH, feed) **HTTP contracts** — build against the typed contract in parallel, mock until live. Builds `apps/web` + `packages/i18n` + `packages/ui`.

## PR batches (~100 PRs)
| Batch | Scope | Spec | Definition of Done |
|---|---|---|---|
| **W1 · App shell** | SolidStart SSR marketing pages (SEO) + Solid/Vite SPA (logged-in app) split; thin routes, routing, layout. | [app-shell](../../specs/web-client/app-shell/README.md) | Public pages SSR; app is an SPA; routes parse→query→render only. |
| **W2 · Design system** | `packages/ui`: **dark-only** semantic CSS-variable tokens by role, component library, a11y. | [design-system](../../specs/web-client/design-system/README.md) | Components use `bg-bg text-fg`, never raw hex; no theme toggle. |
| **W3 · i18n** | `packages/i18n`: `t()` from day one, catalogs, `Intl` for dates/money, **missing keys render loud** (`⟦key⟧`). | [i18n](../../specs/web-client/i18n/README.md) | Every string via `t()`; operator locales swappable. |
| **W4 · Data layer** | TanStack Solid Query hooks, **Zod at every boundary**, typed errors, API client to gateway/worldsvc. | [data-layer](../../specs/web-client/data-layer/README.md) | No hand-rolled fetch cache; every response Zod-validated → typed. |
| **W5 · Account & auth** | Registration, login, sessions, account management — **server-authoritative via gateway**. | [account-auth](../../specs/web-client/account-auth/README.md) | Sign up → log in → session; the web holds no game truth. |
| **W6 · Armory** | Character & guild lookup, public/private toggle — read-only worldsvc projections. | [armory](../../specs/web-client/armory/README.md) | A character created in the slice shows on the armory. |
| **W7 · Auction house** | AH browse/search, price history — cached reads only, never authoritative. | [auction-house](../../specs/web-client/auction-house/README.md) | Browse live AH listings from worldsvc. |
| **W8 · World feed** | Dynamic world-event/social news feed surfacing [living-world](../../specs/gameplay/world-systems/living-world.md) events. | [world-feed](../../specs/web-client/world-feed/README.md) | Slice events appear in the feed. |
| **W9 · Operator brand** | Realm name, logo, palette (within dark tokens), copy, feature flags — **config, never code edits**; multi-tenant. | [operator-brand](../../specs/web-client/operator-brand/README.md) | A new operator reskins by config; a different game swaps portal surfaces. |
| **W10 · Testing & DX** | testing-library + Playwright, CI, deterministic dark theme → stable visual diffs; operator setup docs. | [testing-dx](../../specs/web-client/testing-dx/README.md) | `bun run --filter @omm/web test` green; operator can deploy from docs. |

## Interfaces this track owns / consumes
- **Owns** `apps/web`, `packages/ui`, `packages/i18n` and the **Zod mirror** of the gateway/worldsvc contracts.
- **Consumes** gateway (auth, realm status) + worldsvc (armory, AH, feed) over HTTP — read-only public data + authenticated account actions. **Never touches shards or the DB directly.**

## Rules (hard)
- **TypeScript strict, no `any`; Zod at every boundary**; custom typed errors.
- **SRP / thin routes** — one component one job, files ≤300 LOC; shared UI → `packages/ui`; data in TanStack Query hooks.
- **Server state only via TanStack Query** — no hand-rolled caches.
- **Dark theme ONLY** — no toggle; semantic tokens by role, defined once in `:root`.
- **i18n from day one** — every user-facing string via `t()`; missing keys render loud.
- **Server-authoritative** — the web renders projections and submits intents to gateway/worldsvc; it never holds game truth.

## Definition of Done (track)
A player registers, logs in, downloads the client, and sees their slice character's armory, live AH listings, and world feed — dark, i18n'd, Zod-validated, brandable by config, `bun test` + Playwright green — talking only to gateway/worldsvc.

## Links
[web-client specs](../../specs/web-client/README.md) · [09 operator-web](../../architecture/09-operator-web.md) · [server](30-game-server.md) · [`webclient` subagent](../../../.claude/agents/webclient.md)
