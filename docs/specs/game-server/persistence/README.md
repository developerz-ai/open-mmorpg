# Persistence & Data Consistency

> The single rule that keeps the economy alive. Dupes are the historical #1 killer of private servers — we make the dupe path fail to compile. → [architecture/04](../../../architecture/04-data-and-consistency.md)

## What it does
Owns all durable state and the **only sanctioned path to move value**. Every ownership change is one transaction against **YugabyteDB** (Postgres-wire, Raft-replicated, strongly consistent). Ephemeral broadcast state lives in **Dragonfly**. The [`persistence`](../../../../crates/persistence/src/lib.rs) crate is the *only* crate that can write ownership; [`cache`](../../../../crates/cache/src/lib.rs) has no API that can.

## 4-tier memory
| Tier | Storage | Crate | Consistency | Holds |
|---|---|---|---|---|
| 1 process-local | in-proc struct | per-conn | none | connection scratch |
| 2 in-shard | ECS world | `ecs-core` | strong (single owner) | live entity state |
| 3 cross-server ephemeral | Dragonfly | `cache` | eventual, broadcast-only | position fanout, chat, presence, AH read cache, rate counters |
| 4 transactional | YugabyteDB | `persistence` | strong (Raft) | **everything ownable** — inventory, currency, characters, ledger |

## The anti-dupe contract (enforced by types)
- Every ownership mutation (trade, mail, loot, vendor, currency, craft consume/produce) is an atomic transaction via [`OwnershipStore`](../../../../crates/persistence/src/lib.rs). [`transfer`](../../../../crates/persistence/src/lib.rs) either moves the item or errors — **no intermediate state where the item exists twice**.
- **`cache` cannot express an ownership write.** The dupe path is a compile error, not a code-review convention.
- Trades/mail are a **two-phase, single-transaction** move: debit source + credit destination in one DB transaction, or neither ([economy](../economy/README.md)).
- **Idempotency keys** on every mutating economy op — a retried/duplicated packet can't double-apply.

## Schema & migrations
- `sqlx` with compile-time-checked queries; migrations numbered, **never edited after ship** (supersede instead).
- Schema by domain: `characters`, `inventory`, `currency`, `ledger`, `auctions`; one re-export.
- Yugabyte speaks Postgres wire → dev on plain Postgres, scale out unchanged.
- Character/world snapshots persisted on a cadence + on logout/handoff; **transient sim state** (velocity, in-flight cast) is not durable — it's rebuilt from durable state on shard handoff ([sharding](../sharding/README.md)).

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| AzerothCore/TrinityCore 3-DB split (`auth`/`characters`/`world`) on MySQL | Clean separation of auth vs player vs static game data. But dupes historically slipped in via caching, non-atomic mail/AH/trade paths, and delayed async saves ("save the character later"). | **Keep** the auth/player/static separation as schema domains. **Replace** delayed async saves for ownership with synchronous single-transaction commits; **replace** MySQL single-primary with Yugabyte for horizontal strong consistency. |
| TrinityCore async character save queue | Player state written on a timer → a crash between action and save can roll back or, worse, combined with caching, dupe. | **Never** let an ownership change live only in memory/cache awaiting a flush. Ownership is committed before it is acknowledged to the client. |

## Invariants to `proptest`
- **Inventory conservation** — total item count invariant across any trade/mail/craft.
- **Currency ledger sums to zero** — double-entry ([economy](../economy/README.md)).
- **No spawn without a source** — no op yields more than it consumed unless it's an explicit, sourced spawn.

## Rules
- Typed errors ([`omm_errors`](../../../../crates/errors/src/lib.rs)); never leak credentials/internal detail in an error or log.
- Ownership commits are synchronous w.r.t. the client ack — you cannot report success before Yugabyte confirms.
- Dragonfly may cache AH *reads* and broadcast positions; it must **never** hold the authoritative quantity of anything ownable.

## Links
[economy](../economy/README.md) · [security](../security/README.md) · [sharding](../sharding/README.md) · [`crates/persistence`](../../../../crates/persistence/src/lib.rs) · [`crates/cache`](../../../../crates/cache/src/lib.rs)
