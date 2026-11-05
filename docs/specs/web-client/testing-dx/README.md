# Testing & DX — deterministic, AI-testable

> **`bun test`** for unit/component source-contract checks, **Playwright** E2E against a running [gateway](../../../../apps/gateway) in CI, **Biome** as the format/lint gate. The [dark-only theme](../design-system/README.md) (no toggle) makes screenshots deterministic → stable visual diffs → an **AI agent can author and verify tests**, which is how this repo is built. → [architecture/09 §Testing](../../../architecture/09-operator-web.md).

## The pyramid
| Layer | Tooling | Scope |
|---|---|---|
| **Unit** | `bun test` | Pure logic: [Zod schemas](../data-layer/README.md), [`interpolate`/`t()`](../i18n/README.md), utils |
| **Component** | `bun test` | A component's states — assert loading/error/data wire the right [`t()`](../i18n/README.md) keys |
| **E2E** | Playwright | Real flows against a **running gateway** in CI (register → log in → view armory) |
| **Visual** | Playwright screenshots | Deterministic diffs — see below |

Component tests target the [three-state pattern](../app-shell/README.md): mock the [query hook](../data-layer/README.md), assert each of pending / error / data renders the right `t()` key. A missing string shows `⟦key⟧` — a **failing** assertion, so [i18n gaps fail tests](../i18n/README.md), loudly.

## Deterministic → AI-testable
The [dark-only theme](../design-system/README.md) is the enabler. **No toggle, no `prefers-color-scheme`, no per-user theme** means one canonical render → **screenshot diffs are stable**, not flaky. That determinism is what lets an **AI agent author, run, and verify** tests without a human eyeballing pixels — the point of an AI-built repo.

Pin the other non-determinism sources too: freeze time (`Intl` dates via a fixed clock in tests), stub live data at the [query boundary](../data-layer/README.md), fixed viewport. Deterministic input + deterministic theme = a diff that only moves when the UI actually changed.

## Biome — the gate
**Biome** is format + lint, non-negotiable in CI. No `any` (TS strict), no bare literals where [`t()`](../i18n/README.md) belongs, no raw hex where [tokens](../design-system/README.md) belong — caught mechanically, not in review.

## What CI runs
```
biome ci            # format + lint, TS strict — the gate
bun test            # unit + component (source-contract checks)
playwright test     # E2E + visual, against a running gateway
```
Green CI is the merge bar — same commands local, hook, and CI ([CLAUDE.md](../../../../CLAUDE.md)).

## AI-agent authorable
Everything above compounds for agents: typed boundaries mean a wrong shape **won't compile**; loud `⟦key⟧` and stable screenshots mean a regression is **visible in a diff**; deterministic renders mean a test either passes or points at a real change. An agent writes the [thin route](../app-shell/README.md) + hook + component + test in one pass and verifies itself.

## Distilled from
| Reimagines | Keep | Fix |
|---|---|---|
| Manual QA click-throughs | Catching regressions | **Automated `bun test` + Playwright** in CI |
| Flaky snapshot suites (theme/time drift) | Visual regression coverage | **Deterministic dark theme + frozen clock** → stable diffs |
| Lint/format bikeshed in review | Consistent code | **Biome as the gate** — mechanical, not opinion |
| Human-eyeballed verification | Confidence a change works | **AI-testable** determinism — agent verifies itself |

## Rules
- **`bun test`** for unit/component source-contract checks; **Playwright** E2E against a running gateway.
- **Assert on [`t()`](../i18n/README.md) output** — `⟦key⟧` in a test is a failure.
- **Determinism is a test requirement** — dark-only theme, frozen time, stubbed queries, fixed viewport.
- **Biome is the gate** — TS strict, no `any`, no bare literals, no raw hex.
- **Same commands local / hook / CI.** Green is the merge bar.

## Links
[app-shell](../app-shell/README.md) · [design-system](../design-system/README.md) · [i18n](../i18n/README.md) · [data-layer](../data-layer/README.md) · [index](../README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [`apps/gateway`](../../../../apps/gateway) · [CLAUDE.md](../../../../CLAUDE.md)
