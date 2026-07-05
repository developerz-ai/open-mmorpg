# Operator Web Portal — build-out plan

Track: [50 web-client](../../../../../mvp/v1/50-web-client.md) · Specs: [web-client](../../../../../specs/web-client/README.md)

Build `apps/web` + `packages/{i18n,ui}` end-to-end: account/auth, armory, auction
browser, world feed — dark-only, i18n-first, Zod-validated, brandable by config,
talking only to gateway/worldsvc (mocked where the server isn't live yet).

## Deviation on record
The spec's app-shell calls for a **SolidStart SSR** half for marketing + a Vite
SPA for the app. The scaffold is a Vite SPA. To keep CI green and the change
coherent, v1 ships as a **single Vite SPA** with a marketing/app route split and
SSR-ready thin routes (parse → hook → render). SSR migration is a follow-up; no
functional DoD item depends on it. Every other spec rule holds verbatim.

## Slices (fat PRs, machine-reviewed)
1. **Foundation** — design system (`packages/ui`), i18n + `Intl` (`packages/i18n`),
   data layer (typed api client, Zod, typed errors, mock backend), app shell
   (router, layout, brand header/footer), operator brand + Zod-validated config,
   marketing home, realm status. Component-test harness (`@solidjs/testing-library`
   + happy-dom). → `feat/web-foundation`
2. **Portal surfaces** — account/auth, armory, auction house, world feed; each a
   thin route → query hook (Zod) → `packages/ui` component, feature-flag gated,
   with component tests + Playwright E2E + CI wiring. → `feat/web-surfaces`

## Invariants (enforced)
- TS strict, no `any`; Zod at every boundary; typed errors.
- Files ≤300 LOC, SRP; shared UI → `packages/ui`; data in query hooks.
- Dark-only tokens by role; components use `bg-*/text-*`, never hex.
- Every string via `t()`; missing keys render `⟦key⟧`. Dates/money via `Intl`.
- Server-authoritative: projections in, intents out. No game truth in the web.
