# Auction House (browser)

> Browse, search, price-history, watchlists. **CRITICAL: this is a read-only view.** The authoritative AH is the transactional [`worldsvc`](../../../../apps/worldsvc) service ([economy](../../game-server/economy/README.md)); Dragonfly caches AH **reads/search only** and never holds authoritative quantity. The web renders listings and lets a logged-in user submit a **bid/buy intent** — worldsvc does the single-transaction move. The web never mints or moves value.

## What it does
The best AH shopping UX, over a first-party cached read API — no addon, no scraper. Search/filter/sort listings, chart price history, keep watchlists. Buying is an **intent submission**: the web POSTs "buy listing X" to worldsvc, which runs the atomic seller-credit + buyer-debit + item move in one Yugabyte transaction and returns the outcome. The web displays it; value moved only in the server txn.

## The authority line (do not cross)
| Step | Where it happens | Web's role |
|---|---|---|
| Browse / search / sort | Dragonfly-cached **read** projection | render, filter UX, paginate |
| Price history chart | worldsvc read projection | visualize, never compute truth |
| Watchlist | web/user pref + read subscription | notify on cached-read change |
| **Bid / buy** | **worldsvc single Yugabyte txn** ([economy](../../game-server/economy/README.md)) | submit intent, show typed result |
| Quantity / ownership | **Yugabyte, in-txn** | never cached, never authoritative in web/cache |

## Design
- **Read view + intent submit — nothing else.** No listing quantity is ever authoritative in the web or in Dragonfly; the cache is a search accelerator, not a ledger.
- **Zod at the boundary** — listings, price series, buy results all parsed to typed shapes (per [`realm.ts`](../../../../apps/web/src/lib/realm.ts)) before render.
- **TanStack Query** for search/listing/price reads (cached, stale-while-revalidate); **mutation** for the buy/bid intent, invalidating the affected queries on the server's confirmed outcome.
- **Idempotent buy intent** — the mutation carries an idempotency key so a retried packet can't double-submit; worldsvc enforces ([economy](../../game-server/economy/README.md)).
- **Commit before ack** — "bought" shows only on worldsvc's confirmed transaction, never optimistic UI.
- **i18n + `Intl`** — labels via `t()`; prices/timestamps via `Intl`, not translation strings.

## Distilled from
| Source | Lesson | Our verdict |
|---|---|---|
| AH addon + scraper sites (TSM data, external AH trackers) | Great search/price UX — but built on scraped, stale, unofficial data with no path to actually transact | **Keep** the search/price-history UX, **replace** the data path with a first-party cached read API |
| Scrapers as a de-facto price authority | Off-server data drifts and gets treated as truth; buying still means alt-tabbing into the game | **Fix the authority** — value moves only in the worldsvc Yugabyte txn; the web submits an intent, never mints value |

## Rules
- **Read-only view + intent submit.** The web never mints or moves value; the worldsvc transaction does.
- **Dragonfly caches reads/search only** — never authoritative quantity or ownership.
- **Buy/bid is an idempotent intent** to worldsvc; "bought" shows only on confirmed commit.
- **Zod-validate every listing, price series, and buy result.**
- **Server state via TanStack Query**; prices/dates via `Intl`.
- **AH view on/off is an operator flag** ([operator-brand](../operator-brand/README.md)).

## Links
[data-layer](../data-layer/README.md) · [app-shell](../app-shell/README.md) · [operator-brand](../operator-brand/README.md) · [i18n](../i18n/README.md) · [architecture/09](../../../architecture/09-operator-web.md) · [economy](../../game-server/economy/README.md) · [game-server](../../game-server/README.md) · [`apps/worldsvc`](../../../../apps/worldsvc) · [`realm.ts`](../../../../apps/web/src/lib/realm.ts) · [CLAUDE.md](../../../../CLAUDE.md)
