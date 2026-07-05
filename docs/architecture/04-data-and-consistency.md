# 04 — Data & Consistency

> The single rule that keeps the economy alive. Get this wrong and dupes kill the server — the historical #1 cause of private-server death.

## 4-tier memory (recap → [../initial-idea/04-data-storage.md](../initial-idea/04-data-storage.md))
| Tier | Storage | Crate | Consistency |
|---|---|---|---|
| 1 process-local | in-proc struct | (per-conn) | none |
| 2 in-shard | ECS world | `ecs-core` | strong (single owner) |
| 3 cross-server ephemeral | **Dragonfly** | `cache` | eventual — broadcast only |
| 4 transactional | **YugabyteDB** | `persistence` | strong (Raft) |

## The anti-dupe contract (enforced by types)
- **Every ownership change** (trade, mail, loot, vendor, currency, craft consume/produce) is a **transaction in `persistence`** against Yugabyte. Atomic, serializable where it matters.
- **`cache` has no API that can express an ownership write.** The type system forbids the dupe path — not a code-review convention, a compile error.
- Trades/mail use a **two-phase, single-transaction** move: debit source + credit destination in one DB transaction, or neither.
- **Idempotency keys** on every mutating economy op so a retried packet can't double-apply.

## What Dragonfly is allowed to hold
Position broadcast, chat, presence, auction-house **read cache**, world-feed fanout, rate-limit counters. **Never** the authoritative quantity of anything ownable.

## Migrations & schema
- `sqlx` with compile-time-checked queries; migrations numbered, never edited after ship.
- Schema by domain (`characters`, `inventory`, `currency`, `ledger`, `auctions`), one re-export.
- Yugabyte speaks Postgres wire → dev on plain Postgres, scale out unchanged.

## Invariants to `proptest`
- Inventory conservation: total item count is invariant across any trade/mail/craft.
- Currency ledger sums to zero (double-entry).
- No operation yields more than it consumed unless it's an explicit, sourced spawn.
