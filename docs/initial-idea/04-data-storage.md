# 04 — Data & Memory Tiers

## 4-tier memory hierarchy
| Tier | What | Storage | Consistency |
|---|---|---|---|
| 1. Process-local | packet parse, input buffer, per-connection state | in-process struct/map | none (per-conn) |
| 2. Shared-in-shard | entity state within one zone shard | ECS world / `tokio` actors | strong (single owner) |
| 3. Fast cross-server | position broadcast, chat, presence, AH read cache | **DragonflyDB** (Redis-compat, multi-threaded) | eventual — **ephemeral/broadcast only** |
| 4. At-rest, transactional | character, inventory, currency, item ownership | **YugabyteDB** (Postgres-wire, Raft) | strong |

## Critical rule (the #1 anti-dupe rule)
**Anything touching item/currency ownership goes through YugabyteDB directly** — even if it feels hot-path. Never through Dragonfly. Dupes are the historical #1 killer of private servers.

- Dragonfly = cache / broadcast / ephemeral only. **Never** a source of truth for ownership.
- Enforce in code: ownership writes are typed so they can *only* target the transactional layer — make the illegal path unrepresentable ([03-tech-stack.md](03-tech-stack.md)).

## Notes
- YugabyteDB speaks the Postgres wire protocol → same schema/driver as a plain Postgres dev box; scale out is a drop-in.
- Give each shard type its own Dragonfly DB index — no shared keys.
