# 🌐 Web-Client Specs

> **Scope:** the **operator web** — the brandable website every server operator hosts for their realm: marketing/landing, account & registration, realm status, armory, auction browser, world feed, downloads. This is [`apps/web`](../../../apps/web) — the **only non-Rust part of the stack** (Bun + SolidJS), a deliberate deviation because it's a standard web product, not a hot path. The game the player *runs* is the [client](../client/README.md) (Bevy); this is the site they *sign up and check their character on*. Architecture-level rationale: [architecture/09](../../architecture/09-operator-web.md).

## Stack (gold-standard default — do not re-litigate)
| Layer | Choice |
|---|---|
| Runtime · Language | **Bun** · **TypeScript** (strict, no `any`) |
| UI | **SolidJS** |
| Public/SEO pages | **SolidStart** (SSR) — SEO only |
| Logged-in app | **SPA** (Solid + Vite + `@solidjs/router`) |
| Server state | **TanStack Solid Query** — never a hand-rolled fetch cache |
| Styling | **Tailwind + semantic CSS-variable tokens**, dark theme only |
| Validation | **Zod** at every boundary |
| i18n | **`t()` from day one**, missing keys render loud (`⟦key⟧`) |
| Lint/format · Test | **Biome** · `bun test` + `@solidjs/testing-library` + Playwright |

It talks to [`apps/gateway`](../../../apps/gateway) (HTTP control plane: realm status, auth) and [`apps/worldsvc`](../../../apps/worldsvc) (cross-shard data: armory, auction house, world feed) over HTTP — read-only public data, authenticated account actions. It never touches shards or the DB directly.

## Framework vs. brand — every operator forks this
Like the [server](../game-server/README.md#framework-vs-game--build-other-games-on-this), the web-client is a **reusable product**, not a bespoke site. The split:
- **Reusable app framework** (fork verbatim): the SPA/SSR shell, [design-system](design-system/README.md), [i18n](i18n/README.md), [data-layer](data-layer/README.md), [account-auth](account-auth/README.md), [testing/DX](testing-dx/README.md).
- **This game's portal surfaces** (data + config): [armory](armory/README.md), [auction-house](auction-house/README.md), [world-feed](world-feed/README.md) — MMORPG-shaped, but driven by the gateway/worldsvc API contracts, not hard-coded copy.
- **Per-operator brand** (config, never code edits): realm name, logo, palette (within the dark tokens), copy, and enabled features come from [operator config](operator-brand/README.md) — mirrors the data-driven core. A different game reuses the framework column and swaps portal surfaces to its own API.

## Index
| Spec | Subsystem | Covers |
|---|---|---|
| [app-shell/](app-shell/README.md) | SSR-marketing + SPA-app split, thin routes, routing, layout | SolidStart SEO pages + Solid/Vite SPA |
| [design-system/](design-system/README.md) | Dark-only theming, semantic tokens, `packages/ui`, a11y | Token roles, component library |
| [i18n/](i18n/README.md) | `t()`, catalogs, `packages/i18n`, `Intl` for dates/money | Loud missing keys, operator locales |
| [data-layer/](data-layer/README.md) | TanStack Query hooks, Zod boundaries, typed errors, API client | Gateway/worldsvc contracts, realm status |
| [account-auth/](account-auth/README.md) | Registration, login, sessions, account management | Server-authoritative auth via gateway |
| [armory/](armory/README.md) | Character & guild lookup, public/private toggle | Read-only worldsvc projections |
| [auction-house/](auction-house/README.md) | AH browse/search, price history | Cached reads only — never authoritative |
| [world-feed/](world-feed/README.md) | Dynamic world-event / social news feed | Surfaces [living-world](../gameplay/world-systems/living-world.md) events |
| [operator-brand/](operator-brand/README.md) | Brandability, feature flags, multi-tenant config | Fork-and-reskin without code edits |
| [testing-dx/](testing-dx/README.md) | Testing-library + Playwright, CI, AI-testability | Deterministic dark theme → stable diffs |

## Non-negotiable principles (inherited → [CLAUDE.md](../../../CLAUDE.md))
1. **SRP / thin routes.** One component, one job; files ≤300 LOC; shared UI → `packages/ui`. A route parses params, calls a query, renders — data logic lives in hooks.
2. **Zod at every boundary.** Never trust a network shape; validate, then the inferred type flows into the UI. Custom typed errors, not bare throws.
3. **Server state only via TanStack Query.** No hand-rolled caches.
4. **Dark theme ONLY** — no toggle, semantic tokens by role (`--color-bg`, never raw hex).
5. **i18n from day one** — every user-facing string via `t()`; missing keys render loud.
6. **Server-authoritative.** The web never holds game truth; it renders projections and submits intents to gateway/worldsvc ([game-server](../game-server/README.md)).

## Distilled from
Standard operator/community-site patterns done right: WoW's Armory & community site, private-server control panels (the brittle PHP kind we replace), and the [gold-standard SolidJS frontend](../../architecture/09-operator-web.md) defaults — typed, i18n-first, dark, AI-testable, brandable by config.

> Each subsystem doc is ≤ ~1 screen: what it does, the design, what it reimagines, the rules, links. Grows past that → split it. Same SRP rule as the code.
