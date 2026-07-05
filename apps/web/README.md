# `@omm/web` — Operator Web Portal

The brandable website every operator hosts for their realm: marketing/landing,
account & registration, realm status, armory, auction browser, world feed,
downloads. The only non-Rust app in the stack — **Bun + SolidJS**. It talks to
[`apps/gateway`](../gateway) (auth, realm status) and [`apps/worldsvc`](../worldsvc)
(armory, auction house, world feed) over HTTP, and holds no game truth.

→ Spec: [`docs/specs/web-client`](../../docs/specs/web-client/README.md) ·
Architecture: [`docs/architecture/09-operator-web.md`](../../docs/architecture/09-operator-web.md)

## Run it

```bash
bun install
bun run --filter @omm/web dev      # http://localhost:5173 — mocks on by default
```

With `VITE_USE_MOCKS=true` (the dev default) the site is fully usable with **no
backend** — every gateway/worldsvc call is served by an in-memory mock. Point it
at real servers by setting the endpoints and `VITE_USE_MOCKS=false`.

## Configure — brand by config, never code

An operator ships a realm by setting env, never by editing code. Everything is
Zod-validated at boot; a bad value fails loud.

| Variable | Default | Purpose |
|---|---|---|
| `VITE_REALM_NAME` | `Open-MMORPG` | Realm/brand name (title, header, footer) |
| `VITE_REALM_TAGLINE` | — | Hero tagline copy |
| `VITE_LOGO_URL` | — | Header logo (falls back to the realm name) |
| `VITE_BRAND_ACCENT` | — | Accent token override, `"R G B"` channels (within the dark palette) |
| `VITE_BRAND_ACCENT_STRONG` | — | Stronger accent token override, `"R G B"` |
| `VITE_GATEWAY_URL` | `http://localhost:8080` | Gateway base URL (auth, realm status) |
| `VITE_WORLDSVC_URL` | `http://localhost:8081` | Worldsvc base URL (armory, AH, feed) |
| `VITE_USE_MOCKS` | `false` | Serve all API calls from the in-memory mock backend |
| `VITE_REGISTRATION_OPEN` | `true` | Show the registration form (server re-enforces) |
| `VITE_CASH_SHOP` | `false` | Enable the cash shop surface |
| `VITE_ARMORY_PUBLIC` | `true` | Expose the armory |
| `VITE_AUCTION_HOUSE` | `true` | Expose the auction browser |
| `VITE_WORLD_FEED` | `true` | Expose the world feed |

The palette stays **within the dark tokens** — you retint the accent role; there
is no light theme and no toggle. A different game reuses this shell and swaps the
portal surfaces to its own API contracts. → [operator-brand](../../docs/specs/web-client/operator-brand/README.md)

## Add a language

Drop a catalog for the locale beside [`src/lib/catalog.ts`](src/lib/catalog.ts) —
no code change. Every string flows through `t()`; a missing key renders `⟦key⟧`
(loud, never a silent English fallback). Dates/numbers/money format via `Intl`.

## Test

```bash
bun test                              # unit + integration (mock backend, Zod, i18n)
bun run --filter @omm/web test:e2e    # Playwright E2E (deterministic, dark, frozen clock)
```

## Deploy

`docker build -f docker/web.Dockerfile -t omm-web .` builds the static bundle and
serves it via nginx with SPA fallback. Set the `VITE_*` env at build time to bake
in the operator's brand and endpoints.
