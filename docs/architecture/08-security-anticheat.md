# 08 — Security & Anti-Cheat

> Anti-cheat + anti-dupe is the **#1 priority** — it's what historically kills private servers. Everything below is a hard rule.

## Server authority (the foundation)
- **Client never asserts state.** It sends *intent* (inputs); the server decides outcomes.
- Server re-validates every action against authoritative state: range, cooldown, resource cost, line-of-sight, ownership.
- `crates/sim` is **deterministic** → server can re-simulate to catch impossible client claims ([03](03-netcode-and-sharding.md)).

## Anti-dupe (see [04](04-data-and-consistency.md) for the full contract)
- Ownership changes → **YugabyteDB transaction**, atomic, idempotent. Never via cache/bus.
- The `cache` crate has **no API** that can express an ownership write — the dupe path won't compile.
- `proptest` invariants: inventory conservation, double-entry currency ledger sums to zero.

## Network & edge
| Threat | Mitigation |
|---|---|
| DDoS on game ports | Cloudflare Spectrum / Anycast; never expose shard IPs |
| Login floods / packet spam | L7 rate-limit + connection-churn detection at gateway |
| Session hijack | short-lived signed session tokens issued by gateway, validated per shard |
| Speed/teleport hacks | server-side movement validation vs. max speed + tick delta |

## Untrusted content (operator mods)
- Operator WASM/Lua runs **sandboxed**, **fuel-metered**, with a **narrow capability API** — no direct DB/cache/filesystem/network ([05](05-ecs-and-scripting.md)).
- A misbehaving script can be starved (fuel) or unloaded without taking down the shard.

## Rust-level hygiene (gold standard)
- **No `unwrap`/`expect` on hot paths** — a panic drops all in-flight players on that shard.
- **Typed errors** (`thiserror`), stable client codes; never leak internal detail or credentials in an error/log.
- **Make illegal states unrepresentable** — the type system is the first anti-cheat layer.
- Newtypes over bare primitives (`AccountId`, `ItemId`), never raw `u64` you can mix up.

## The quality net (defense in depth)
Tests → automated AI review on every PR → human review + supervised deploy. No single layer is trusted to be perfect → `../../gold-standards-in-ai/docs/00-philosophy.md`.
