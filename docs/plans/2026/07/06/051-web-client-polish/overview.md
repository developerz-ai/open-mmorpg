# 051 — Web Client Polish (W3, W7-W10)

> **Goal:** Complete the web-client track with remaining locales, auction house polish, world feed enhancements, character/guild pages, accessibility audit, and operator documentation.

## Context
- **W1-W8** merged: app shell, design system, i18n (6/13 locales), data layer, account/auth, armory, AH (basic), world feed (basic), downloads, cash shop
- `bin/check` is green (276 web tests + rust tests)
- Brand theming (`lib/brand.ts`) exists but lacks tests

## Definition of Done
- All 13 i18n locales ship (ko, ru, pt, it, pl, tr, ar added)
- Auction house has filters, sorting, pagination
- World feed has filtering, pagination, event type badges
- Character/guild pages show achievements, history, stats
- Accessibility audit complete with automated tests
- Operator setup documentation complete
- `bin/check` green, all PRs merged via `claudetm merge-pr`

## PR Batches

| PR | Batch | Scope | Size |
|---|---|---|---|
| 051-1 | W3 + W9 | Remaining i18n locales + brand theming tests | Medium |
| 051-2 | W7 | Auction house filters, sorting, pagination | Medium |
| 051-3 | W8 | World feed filtering, pagination, event types | Small |
| 051-4 | — | Character/guild pages achievements, history | Medium |
| 051-5 | W10 | Accessibility audit + operator docs | Medium |

## Success Criteria
1. All 13 locale files present and tested (ko, ru, pt, it, pl, tr, ar added)
2. Auction house: category filter, price range, sorting, pagination
3. World feed: event type filter, pagination, event badges
4. Character pages: achievements, recent activity
5. Guild pages: member roster, online status
6. Accessibility: axe-core passes, ARIA labels complete, keyboard nav verified
7. `bun test` + `bin/check` green
