# 051 — Web Client Polish (W3, W7-W10)

> **Goal:** Complete the web-client track with remaining locales, auction house polish, world feed enhancements and character/guild pages. The original accessibility/operator-documentation scope (051-5) is superseded by [052-4 in the 052 status record](../052-web-client-production-ready/status.yml); see the [archived 051-5 plan and current disposition](pr-05.md).

## Context
- **W1-W8** merged: app shell, design system, i18n (6/13 locales), data layer, account/auth, armory, AH (basic), world feed (basic), downloads, cash shop
- `bin/check` is green (276 web tests + rust tests)
- Brand theming (`lib/brand.ts`) exists but lacks tests

## Definition of Done
- All 13 i18n locales ship (ko, ru, pt, it, pl, tr, ar added)
- Auction house has filters, sorting, pagination
- World feed has filtering, pagination, event type badges
- Character/guild pages show achievements, history, stats
- Accessibility scope superseded by 052-4: current coverage consists of [source-level ARIA/keyboard/focus checks](../../../../../../packages/ui/src/a11y.test.tsx) and [token-contrast tests](../../../../../../packages/ui/src/contrast.test.ts), not a completed browser audit or WCAG certification
- Operator-documentation scope superseded: use the existing [operator setup guide](../../../../../operations/operator-setup.md), not the archived `docs/operator/*` targets
- `bin/check` green, all PRs merged via `claudetm merge-pr`

## PR Batches

| PR | Batch | Scope | Size |
|---|---|---|---|
| 051-1 | W3 + W9 | Remaining i18n locales + brand theming tests | Medium |
| 051-2 | W7 | Auction house filters, sorting, pagination | Medium |
| 051-3 | W8 | World feed filtering, pagination, event types | Small |
| 051-4 | — | Character/guild pages achievements, history | Medium |
| 051-5 | W10 | Superseded by [052-4](../052-web-client-production-ready/status.yml); [archived accessibility/operator-doc scope](pr-05.md) | Medium |

## Success Criteria
1. All 13 locale files present and tested (ko, ru, pt, it, pl, tr, ar added)
2. Auction house: category filter, price range, sorting, pagination
3. World feed: event type filter, pagination, event badges
4. Character pages: achievements, recent activity
5. Guild pages: member roster, online status
6. Accessibility (superseded by 052-4): source-level ARIA/keyboard/focus checks and token-contrast tests as linked above; the proposed axe route audit was not implemented and is not an active requirement. `bin/check` runs no axe or Playwright scan; these checks do not verify browser keyboard behavior.
7. `bun test` + `bin/check` green
