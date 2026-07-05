# Economy

> Trade, mail, auction house, loot, crafting — every value move is one Yugabyte transaction. Dupes are the historical #1 killer of private servers; here the dupe path does not compile. → [architecture/04](../../../architecture/04-data-and-consistency.md)

## What it does
Runs cross-player value exchange. All state-of-record is durable ([persistence](../persistence/README.md)); [`worldsvc`](../../../../apps/worldsvc/src/main.rs) hosts the global services (auction house, mail, guild bank) that span shards. Dragonfly may cache AH **reads** and fan out listings; it **never** holds an authoritative quantity.

## The transactional rule (every economy op)
- **One move = one transaction.** Trade/mail/vendor/loot/craft are a two-phase **debit-source + credit-destination** in a single Yugabyte transaction via [`OwnershipStore::transfer`](../../../../crates/persistence/src/lib.rs) — or neither side changes. No intermediate state where an item exists twice.
- **Idempotency keys** on every mutating op — a retried or duplicated packet can't double-apply.
- **Commit before ack.** The client is told "traded" only after Yugabyte confirms — never on optimistic in-memory state.

## Services
| Service | Home | Value path |
|---|---|---|
| **Trade** (player↔player) | shard, arbitrated | two-phase single-txn swap; both confirm, then one atomic move |
| **Mail** (with attachments) | worldsvc | attach = escrow via txn; claim = txn credit; no attachment exists in two mailboxes |
| **Auction house** | worldsvc | list = escrow txn; buy = atomic seller-credit + buyer-debit + item move; Dragonfly caches read/search only |
| **Loot / vendor / craft** | shard | spawn/consume are explicit, sourced txns ([combat](../combat/README.md)) |

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| TrinityCore AH/mail/trade on MySQL with in-memory authority + async character saves | Non-atomic multi-step economy ops + the gap between memory and periodic DB save = the classic dupe vector. Global services assume single-process shared memory (a stated blocker to distribution). | **Replace** with single-transaction atomic moves committed before ack, and put global services in `worldsvc` over a cross-shard bus so distribution isn't blocked. |
| GTA session merge/split dupe-guard (Take-Two patent) | Even Rockstar's seamless-world tech centers guarding against object duplication on session merge. | Our merge/split can't mint value: IDs reconcile, but value only moves via a Yugabyte txn ([sharding](../sharding/README.md)). |

## Invariants (`proptest`, enforced continuously)
- **Inventory conservation** — total item count invariant across any trade/mail/craft.
- **Currency ledger sums to zero** — double-entry; every credit has a matching debit.
- **No unsourced spawn** — no op yields more than it consumed unless it's an explicit, sourced spawn (loot table, quest reward).

## Rules
- Ownership never routes through cache or bus — direct to Yugabyte ([persistence](../persistence/README.md)).
- Money sinks/faucets are explicit and logged; the ledger is auditable double-entry.
- AI companions transacting the AH use the **same** transactional path, rate-limited to human-plausible rates ([security](../security/README.md), [architecture/07](../../../architecture/07-mcp-companions.md)).

## Links
[persistence](../persistence/README.md) · [security](../security/README.md) · [sharding](../sharding/README.md) · [`crates/persistence`](../../../../crates/persistence/src/lib.rs)
