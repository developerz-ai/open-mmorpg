# 10 — Database & Persistence · Wave 1

> **Big idea.** The **only place ownership is written** — and it writes to **one YugabyteDB transaction, never through a cache or bus**. This track builds [`crates/persistence`](../../../crates/persistence) (durable, strongly consistent) and [`crates/cache`](../../../crates/cache) (Dragonfly, ephemeral only, *with no ownership-write API so the dupe path won't compile*). Anti-dupe by **type**, not by convention. **Owner: [`db`](../../../.claude/agents/db.md).** → [persistence spec](../../specs/game-server/persistence/README.md) · [04 data-and-consistency](../../architecture/04-data-and-consistency.md)

## Reads
[persistence spec](../../specs/game-server/persistence/README.md) · [04 data-and-consistency](../../architecture/04-data-and-consistency.md) · [04 data-storage](../../initial-idea/04-data-storage.md) · [economy spec](../../specs/game-server/economy/README.md) (ledger) · gold standards `stack/rust-apis.md`.

## Depends on
[Foundation](00-foundation.md): `errors`, newtypes (`AccountId`/`CharacterId`/`ItemId`), Docker compose with Yugabyte + Dragonfly.

## PR batches (~100 PRs)
| Batch | Scope | Definition of Done |
|---|---|---|
| **D1 · Connection & pool** | `sqlx` pool to Yugabyte (Postgres wire), health checks, typed config, ret/backoff. Async end-to-end. | Integration test connects to compose Yugabyte, round-trips a row. |
| **D2 · Migrations** | `sqlx migrate` framework, versioned, forward-only + tested rollback path, CI runs them on a fresh DB. | `bin/setup` migrates cleanly; drift detected in CI. |
| **D3 · Schema: accounts & characters** | Tables + typed repos for `accounts`, `characters`, position/stats. Newtype-keyed. | CRUD repos with tests; no raw `u64` keys. |
| **D4 · Ownership txn API** | The **single** transactional write path: move/grant/consume items, `Tx` handle, idempotency keys. Returns typed errors. | Concurrent transfer test proves **no dupe, no loss** under contention. |
| **D5 · Double-entry ledger** | Append-only ledger for every value move ([economy](../../specs/game-server/economy/README.md)); balances derived, reconcilable. | Ledger sums to zero; replay reconstructs balances. |
| **D6 · `cache` (Dragonfly)** | Ephemeral KV/pubsub: presence, AoI broadcast, session tokens. **Compile-time: no ownership-write method exists.** | Type-level proof — a test that tries to persist ownership via `cache` fails to compile (doc'd). |
| **D7 · Anti-dupe invariants** | Property tests + a re-sim harness: fuzz concurrent transfers/mail/AH against the txn API. | Dupe attempts provably fail; documented in [persistence](../../specs/game-server/persistence/README.md). |
| **D8 · Ops docs** | `docs/operations/` — backup/restore, scaling Yugabyte, connection tuning, migration runbook. | An operator stands up + backs up a realm DB from docs alone. |

## Interfaces this track owns
- **`crates/persistence`** — the ownership-write API. Every mutation of owned state goes through a `Tx` here. Consumed by [server](30-game-server.md) (shard, worldsvc, gateway).
- **`crates/cache`** — ephemeral state only. Consumed by [server](30-game-server.md) for presence/broadcast.
- **DB schema + migrations** — the source of truth for durable shape.

## Rules (hard)
- **Ownership writes → one Yugabyte txn. Never cache/bus.** This is the #1 anti-dupe rule ([CLAUDE.md](../../../CLAUDE.md)).
- **`cache` has no ownership-write API.** Enforce with types — the dupe path must not compile.
- **Idempotent by key** — every mutating op takes an idempotency key; retries never double-apply.
- **No secrets in errors/logs**; typed errors with stable codes.
- **Async on the request/tick path** — `tokio`, never blocking or `std::sync::Mutex`.

## Definition of Done (track)
A player's items survive relog and shard handoff with zero dupes under concurrent load; the ledger reconciles; `cache` cannot compile an ownership write; migrations run clean in CI; operators can back up and restore from the docs.

## Links
[persistence](../../specs/game-server/persistence/README.md) · [04](../../architecture/04-data-and-consistency.md) · [economy](../../specs/game-server/economy/README.md) · [server track](30-game-server.md) · [`db` subagent](../../../.claude/agents/db.md)
