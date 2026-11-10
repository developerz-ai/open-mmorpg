# Testing & DX — deterministic, AI-testable

> **`bun test`** for unit/component source-contract checks, **Playwright** E2E against the in-memory mock backend (`VITE_USE_MOCKS=true`) — local via [`bin/test-e2e`](../../../../bin/test-e2e), CI-ready but not CI-enforced — **Biome** as the format/lint gate. The [dark-only theme](../design-system/README.md) (no toggle) makes screenshots deterministic → stable visual diffs → an **AI agent can author and verify tests**, which is how this repo is built. → [architecture/09 §Testing](../../../architecture/09-operator-web.md).

## The pyramid
| Layer | Tooling | Scope |
|---|---|---|
| **Unit** | `bun test` | Pure logic: [Zod schemas](../data-layer/README.md), [`interpolate`/`t()`](../i18n/README.md), utils |
| **Component** | `bun test` | A component's states — assert loading/error/data wire the right [`t()`](../i18n/README.md) keys |
| **E2E** | Playwright | Browser flows against the **in-memory mock backend** (register → log in → view armory), not proof of live gateway/worldsvc integration — no current CI execution |
| **Visual** | Playwright screenshots | Deterministic diffs — see below |

Component tests target the [three-state pattern](../app-shell/README.md): mock the [query hook](../data-layer/README.md), assert each of pending / error / data renders the right `t()` key. A missing string shows `⟦key⟧` — a **failing** assertion, so [i18n gaps fail tests](../i18n/README.md), loudly.

## Deterministic → AI-testable
The [dark-only theme](../design-system/README.md) is the enabler. **No toggle, no `prefers-color-scheme`, no per-user theme** means one canonical render → **screenshot diffs are stable**, not flaky. That determinism is what lets an **AI agent author, run, and verify** tests without a human eyeballing pixels — the point of an AI-built repo.

Pin the other non-determinism sources too: freeze time (`Intl` dates via a fixed clock in tests), stub live data at the [query boundary](../data-layer/README.md), fixed viewport. Deterministic input + deterministic theme = a diff that only moves when the UI actually changed.

## Biome — the gate
**Biome** is format + lint, non-negotiable in CI. No `any` (TS strict), no bare literals where [`t()`](../i18n/README.md) belongs, no raw hex where [tokens](../design-system/README.md) belong — caught mechanically, not in review.

## What CI runs
After dependency installation, the current [web job](../../../../.github/workflows/ci.yml) runs these commands from the repository root:

```
bun run lint                                 # Biome — format + lint
bun run typecheck                            # tsc --noEmit
bun test                                     # unit + component (source-contract checks)
bun run --filter @omm/web build               # production build
```
The [web job](../../../../.github/workflows/ci.yml) runs `ubuntu-latest` with a **10-minute** job timeout, no step timeouts, no `if`, and fail-fast with the default shell. [The guard](../../../../scripts/lib/e2e-ci.test.ts) pins what a silent mutation could break — the runner, an unconditional job that propagates failure, the 10-minute job timeout held above the sum of any step timeouts, each of the four commands exactly once in that order (unconditional, fail-fast, default shell), and no `--update-snapshots` flag — but it does **not** pin the absence of step timeouts: only their total is bounded by the job timeout. **Playwright is not in CI**: the E2E and visual suites are CI-ready — config, scripts, nine specs, eight baselines and the guard all exist — but no CI job executes a browser.

CI browser execution remains a **pending workflow change**, not a shipped capability. The workflow is the source of truth; the guard's comments describe a prior external-policy refusal, not a repository-enforced restriction or evidence of a browser run.

**Proposed authorized-PR recipe — not shipped:** add browser steps to the web job, in order, with the guard's job contract re-extended in the same PR (including re-pinning the guard's expected job timeout). Raise the job's `timeout-minutes` from 10 to **25** — the proposed step timeouts sum to 20, exceeding the current job budget and failing the guard's job-timeout-above-step-sum check — then add `bunx playwright install --with-deps chromium` (`timeout-minutes: 5`), `bun run --filter @omm/web test:e2e` (`timeout-minutes: 10`), `bun run --filter @omm/web test:e2e:shop` (`timeout-minutes: 5`), and `actions/upload-artifact@v4` with `name: playwright-artifacts`, `if: failure()`, `path: apps/web/test-results`, `if-no-files-found: ignore`. This would upload failure screenshots and first-retry traces under that stable artifact name; an install failure may leave none, and ignoring missing files must not hide the original failure. Neither these steps nor the artifact upload exist in the current web job; validate them in Actions before claiming post-landing coverage.

**CI baselines are read-only.** `updateSnapshots: 'none'` under `CI` rejects missing or mismatched baselines instead of regenerating them. The workflow never passes an update flag. Local Playwright snapshot-update defaults remain unchanged; intentional baseline changes must be reviewed, not generated by CI.

Green CI is required for merging. The local [`bin/check`](../../../../bin/check) web lane in [`scripts/lib/gate.ts`](../../../../scripts/lib/gate.ts) runs the same four commands shown above; the full local gate also includes Rust and content checks. **No committed Git hook setup invokes the gate.** The local gate excludes browser installation and execution — and so does CI today. The [source-contract guard](../../../../scripts/lib/e2e-ci.test.ts) checks wiring, not browser behavior: local green is neither browser proof nor evidence of a GitHub Actions run.

The [web package scripts](../../../../apps/web/package.json) keep the runners separate: `bun run --filter @omm/web test` invokes only `bun test`, not lint, typecheck, build or Playwright. `test:e2e` invokes `playwright test`; `test:e2e:shop` enables and selects the shop suite. The [Playwright config](../../../../apps/web/playwright.config.ts) uses `.e2e.ts` filenames so Bun does not discover browser specs, caps Chromium at two workers, and starts Vite with `VITE_USE_MOCKS=true`.

Locally, [`bin/setup`](../../../../bin/setup) installs Chromium. [`bin/test-e2e`](../../../../bin/test-e2e) runs the default script only; it does **not** enable cash-shop tests. Run both modes explicitly from the repository root for full browser coverage (with Chromium and OS dependencies installed):

```sh
CI=true bun run --filter @omm/web test:e2e
CI=true bun run --filter @omm/web test:e2e:shop
```

The second script sets `CASH_SHOP_ENABLED=true` and selects `e2e/shop-enabled.e2e.ts`; those tests skip in the default run. Both use the deterministic mock server, not live services. The timeouts in the recipe above are GitHub Actions limits, not limits imposed by these local commands.

### Authorized-run procedure

This is a procedure to execute, **not evidence of an observed browser or GitHub Actions run** — nothing in CI executes a browser today, so browser proof comes only from running it where Chromium is installed, or from an authorized workflow change under validation (recipe above). The scoped wiring tests check commands, Linux runner selection and eight baseline filenames; they neither decode PNGs nor compare browser renders.

On a Linux Chromium verification run, with exclusive ownership of the temporary test edits:

1. Run both `CI=true` commands above without mutations; record exits and confirm all eight visual comparisons pass with baseline files unchanged.
2. Temporarily add `expect(1).toBe(2)` after navigation in one default browser test. Run the default command: require a nonzero exit, a failure screenshot and a first-retry trace in `apps/web/test-results`. In a disposable CI validation change, require the web job to be red and inspect `playwright-artifacts`; a local exit alone does not prove upload behavior. Remove the assertion afterward.
3. Temporarily change the `home.png` expectation in `visual.e2e.ts` to an unused filename. Run `CI=true bun run --filter @omm/web test:e2e e2e/visual.e2e.ts`: require a missing-baseline failure and no new baseline file. Restore the expectation.
4. For a mismatch without changing any baseline, temporarily insert `await page.evaluate(() => { document.body.style.opacity = '0'; });` immediately before the home screenshot assertion. Repeat the visual command: require a comparison failure. Remove the temporary statement.
5. Restore every temporary test edit, rerun both modes, and confirm baseline bytes remain unchanged. Never pass snapshot-update flags, regenerate PNGs or weaken tolerances to make these probes green. Record actual results separately from this procedure.

If workflow submission is refused by an external authorization policy, report the exact refusal to the harness/operator; do not remove or disguise browser steps to bypass it.

## AI-agent authorable
Everything above compounds for agents: typed boundaries mean a wrong shape **won't compile**; loud `⟦key⟧` and stable screenshots mean a regression is **visible in a diff**; deterministic renders mean a test either passes or points at a real change. An agent writes the [thin route](../app-shell/README.md) + hook + component + test in one pass and verifies itself.

## Distilled from
| Reimagines | Keep | Fix |
|---|---|---|
| Manual QA click-throughs | Catching regressions | **Automated `bun test` + Playwright** |
| Flaky snapshot suites (theme/time drift) | Visual regression coverage | **Deterministic dark theme + frozen clock** → stable diffs |
| Lint/format bikeshed in review | Consistent code | **Biome as the gate** — mechanical, not opinion |
| Human-eyeballed verification | Confidence a change works | **AI-testable** determinism — agent verifies itself |

## Rules
- **`bun test`** for unit/component source-contract checks; **Playwright** E2E against the in-memory mock backend (`VITE_USE_MOCKS=true`).
- **Assert on [`t()`](../i18n/README.md) output** — `⟦key⟧` in a test is a failure.
- **Determinism is a test requirement** — dark-only theme, frozen time, stubbed queries, fixed viewport.
- **Biome is the gate** — TS strict, no `any`, no bare literals, no raw hex.
- **Keep gate and browser proof separate** — the local web lane and current CI web job run lint, typecheck, Bun tests and build; neither runs Playwright, and no committed Git hook setup invokes the gate. `bun run --filter @omm/web test` is Bun-only. [`bin/test-e2e`](../../../../bin/test-e2e) runs only `test:e2e`; also run `bun run --filter @omm/web test:e2e:shop` for shop-enabled coverage. Green CI is required for merging; a local run is not a CI result.

## Links
[app-shell](../app-shell/README.md) · [design-system](../design-system/README.md) · [i18n](../i18n/README.md) · [data-layer](../data-layer/README.md) · [index](../README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [game-server](../../game-server/README.md) · [mock backend](../../../../apps/web/src/lib/mock) · [CLAUDE.md](../../../../CLAUDE.md)
