# 09 — Operator Web Server

> A **base website every server operator can host** for their realm: landing page, account/registration, realm status, character/armory, auction browser, world feed, downloads. Ships in the box, MIT, fully brandable. The **only non-Rust part of the stack** — a deliberate deviation because it's a standard web product, not a hot path.

## Stack (gold-standard default)
| Layer | Choice |
|---|---|
| Runtime | **Bun** |
| Language | **TypeScript** (strict, no `any`) |
| UI | **SolidJS** |
| Public/SEO pages (landing, blog) | **SolidStart** (SSR) — SEO only |
| Logged-in app (account, armory, AH) | **SPA** (Solid + Vite + `@solidjs/router`) |
| Server state | **TanStack Solid Query** |
| Styling | **Tailwind + semantic CSS variables** |
| Validation | **Zod** at every boundary |
| Lint/format | **Biome** |

→ `../../gold-standards-in-ai/docs/stack/frontend-solidjs.md`

## Where it lives
`apps/web/` (SPA app shell) + `apps/web-site/` (SSR marketing) if both are needed; shared UI in `crates/`-equivalent `packages/ui`, i18n in `packages/i18n`. It talks to `apps/gateway` / `apps/worldsvc` over HTTP (read-only public data; auth for account actions).

## SOLID / SRP (hard rule)
- **One component, one job.** Files ≤300 LOC. Lift shared UI to `packages/ui`.
- **Thin routes** — a route parses params, calls a service/query, renders. Data-fetch logic lives in query hooks, not components.
- **No hardcoded strings** — everything through `t()` ([i18n](#i18n) below).
- **Server state only via TanStack Query** — never hand-rolled fetch caches.
- Custom typed errors, not bare throws. Zod-validate every API response.

## Dark theme ONLY
- **No light theme, no toggle.** One dark palette, semantic tokens defined once in `:root`.
- Tokens by **role, not value** (`--color-bg`, not `--zinc-900`); stored as space-separated RGB channels so `rgb(var(--color-bg) / 0.8)` works.
- Components reference tokens/`bg-bg text-fg` — **never** raw hex.
```css
:root {
  --color-bg:          18  18  20;
  --color-bg-soft:     28  28  32;
  --color-surface:     34  34  39;
  --color-fg:         228 226 222;
  --color-fg-strong:  248 247 245;
  --color-fg-muted:   150 146 140;
  --color-line:        54  54  60;
  --color-accent:      96 170 240;
  --color-accent-strong:130 190 248;
}
```
→ Token roles + Tailwind mapping: `../../gold-standards-in-ai/docs/frontend-craft/theming-dark-mode.md` (we use the dark values only, in `:root`, no media query / no `data-theme`).

## i18n (from day one)
- Every user-facing string through a translator `t()`. **Missing keys render loudly** (`⟦key⟧`), never silent.
- Nested authoring → flat dot-key lookup. One catalog per locale, namespaced by feature.
- SRP modules in `packages/i18n`: `translator` · `catalog` · `locales` · `interpolate`.
- Dates/numbers/money via `Intl`, **not** i18n strings.
- Operators add a locale by dropping a catalog file — no code change.
→ `../../gold-standards-in-ai/docs/frontend-craft/i18n.md`

## Operator brandability
- Realm name, logo, colors (within the dark palette), copy, and enabled features come from **operator config**, not code edits — mirrors the data-driven core philosophy ([05](05-ecs-and-scripting.md)).
- Feature flags: registration open/closed, cash-shop on/off (operator's choice), armory public/private.

## Testing
- `@solidjs/testing-library` + `bun test`; integration against a running gateway in CI.
- Keep it **AI-testable** (Playwright) — deterministic dark theme (no toggle) makes screenshot diffs stable.
