# Security & Anti-Cheat

> Anti-cheat + anti-dupe is the **#1 priority** — it's what historically kills private servers. Everything here is a hard rule. → [architecture/08](../../../architecture/08-security-anticheat.md)

## Server authority (the foundation)
- **Client sends [`Intent`](../../../../crates/protocol/src/lib.rs), never state.** The server decides every outcome and re-validates every action against authoritative state: range, cooldown, resource cost, line-of-sight, ownership ([combat](../combat/README.md)).
- **`sim` is deterministic** → the server can **re-simulate** a suspicious client's inputs and compare — impossible client claims are caught ([tick-loop](../tick-loop/README.md)).
- **Movement is intent-based.** The client requests a move direction; the server integrates position under its own rules (max speed × tick delta). This is the deliberate fix for TrinityCore's **client-tells-server-its-position** model, where anti-cheat is validation-*after-the-fact* — a known cheat surface.

## Anti-dupe (the type-level guarantee)
- Ownership changes → **one Yugabyte transaction**, atomic, idempotent, never via cache/bus ([persistence](../persistence/README.md)).
- The [`cache`](../../../../crates/cache/src/lib.rs) crate has **no API** that can express an ownership write — the dupe path won't compile.
- `proptest` invariants run continuously: inventory conservation, double-entry ledger sums to zero ([economy](../economy/README.md)).

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| TrinityCore movement anti-cheat (`DoAntiCheatCheck`) | Client is authoritative for its own position; server validates speed/teleport/fly *after* the packet. Catches some cheats, prevents none. | **Replace** with intent-based movement — server integrates position, client can't assert it. |
| TrinityCore async saves + caching | The memory↔DB gap is a dupe vector. | **Replace** with commit-before-ack ownership ([persistence](../persistence/README.md)). |
| **GTA Online P2P** (Amazon/Take-Two patents) | Peers hold real authority over owned objects and trust each other's reports → money/stat/teleport/"delete player" exploits; direct peer connections leak IPs → DDoS/host-boot. It was a cost tradeoff, not a technical ideal. | **Avoid** entirely: no client ever owns state; ownership is shard-arbitrated ([sharding](../sharding/README.md)); no peer IPs — clients reach only edge/gateway. |

## Network & edge
| Threat | Mitigation |
|---|---|
| DDoS on game ports | Cloudflare Spectrum / Anycast; **never expose shard IPs** ([sharding](../sharding/README.md)) |
| Login floods / packet spam | L7 rate-limit + connection-churn detection at the gateway |
| Session hijack | short-lived signed session tokens issued by gateway, validated per shard |
| Speed/teleport hacks | server-side movement integration vs. max speed × tick delta |
| Bot farming (incl. AI companions) | action rates clamped to **human-plausible**; every companion action logged with account + token id ([architecture/07](../../../architecture/07-mcp-companions.md)) |

## Untrusted content (operator mods)
- Operator WASM/Lua runs **sandboxed**, **fuel-metered**, with a **narrow capability API** — no direct DB/cache/filesystem/network ([content-scripting](../content-scripting/README.md)).
- A misbehaving script is starved (fuel) or unloaded without taking down the shard.

## Rust-level hygiene (first anti-cheat layer)
- **No `unwrap`/`expect` on hot paths** — a panic drops every in-flight player on that shard.
- **Typed errors** ([`omm_errors`](../../../../crates/errors/src/lib.rs)), stable client codes; never leak internal detail or credentials in an error/log.
- **Make illegal states unrepresentable** — newtypes ([`ItemId`/`AccountId`](../../../../crates/protocol/src/ids.rs)) over raw `u64`; you cannot hand an `ItemId` where an `AccountId` belongs.

## Defense in depth
Tests → automated AI review on every PR → human review + supervised deploy. No single layer is trusted to be perfect. → [gold standards](../../../../CLAUDE.md#standards)

## Links
[persistence](../persistence/README.md) · [economy](../economy/README.md) · [combat](../combat/README.md) · [content-scripting](../content-scripting/README.md) · [sharding](../sharding/README.md)
