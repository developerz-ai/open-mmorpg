# Operator Brand & Config

> Brandability + multi-tenant config. Realm name, logo, palette (**within the dark tokens**), copy, and enabled features come from **operator config**, not code edits — mirrors the data-driven core. One codebase, many branded deployments; a *different game* reuses the shell and swaps portal surfaces. Fork-and-hardcode private-server sites, done as a config-driven product.

## What it does
Turns [`apps/web`](../../../../apps/web) from a bespoke site into a **configurable product**. An operator ships their realm by editing config — never the code. Two axes: **brand** (identity: name, logo, palette-within-tokens, copy, locales) and **features** (which surfaces are on). A different game keeps the framework shell and swaps the [portal surfaces](../README.md#framework-vs-brand--every-operator-forks-this) to its own API contracts.

## Config surface
| Config | Examples | Consumed by |
|---|---|---|
| **Brand** | realm name, logo, favicon, social image, tagline copy | [app-shell](../app-shell/README.md), SSR meta |
| **Palette** | accent + surface values **within the dark token roles** — never raw hex, never a light theme | [design-system](../design-system/README.md) |
| **Locales** | which catalogs ship, default locale | [i18n](../i18n/README.md) |
| **Feature flags** | registration open/closed, cash-shop on/off, armory public/private, AH view on/off, world-feed on/off | each portal surface |
| **Endpoints** | gateway + worldsvc base URLs | [data-layer](../data-layer/README.md) |

## Design
- **Config, not code.** Brand + features load from an operator config (Zod-validated at boot — fail loud on a bad config, never a silent half-brand). Mirrors content-is-data on the core.
- **Palette stays inside the dark tokens.** Operators retint accent/surface **roles**; there is no light theme and no toggle — the dark-only invariant holds across every tenant, keeping [screenshot diffs stable](../testing-dx/README.md).
- **Feature flags gate surfaces client-side and are re-enforced server-side.** A hidden AH is also an off endpoint; the flag is UX, the server is the authority.
- **Cash-shop on/off is the operator's choice** — the framework ships the capability; enabling it is a per-deployment decision.
- **i18n copy is operator-overridable** — brand copy flows through `t()` catalogs, so rebranding needs no code edit.
- **One codebase, many deployments** — no per-operator fork; a new realm is a new config.

## Distilled from
| Source | Lesson | Our verdict |
|---|---|---|
| Private-server sites (fork the PHP, hardcode the realm, re-edit per operator) | Every operator maintaining a code fork = drift, unpatched holes, no shared upgrades | **Replace** with one config-driven product; brand + features are data |
| Bespoke per-game portal rebuilds | Rebuilding the whole site for a different game throws away the reusable shell | **Fix** — framework shell stays; only portal surfaces + config swap ([index](../README.md)) |

## Rules
- **Brand + features are config, not code edits** — one codebase, many branded deployments.
- **Palette lives within the dark tokens** — retint roles, never raw hex, never a light theme/toggle.
- **Feature flags are UX; the server re-enforces** — hidden surface = off endpoint.
- **Config is Zod-validated at boot** — a bad config fails loud, never a silent half-brand.
- **Operator copy via `t()` catalogs** — rebrand without touching code.
- **A different game reuses the shell**, swapping portal surfaces to its own API contracts.

## Links
[README (index)](../README.md) · [design-system](../design-system/README.md) · [app-shell](../app-shell/README.md) · [i18n](../i18n/README.md) · [data-layer](../data-layer/README.md) · [account-auth](../account-auth/README.md) · [armory](../armory/README.md) · [auction-house](../auction-house/README.md) · [world-feed](../world-feed/README.md) · [testing-dx](../testing-dx/README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [`apps/web`](../../../../apps/web) · [CLAUDE.md](../../../../CLAUDE.md)
