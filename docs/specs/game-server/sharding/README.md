# Sharding & Merge/Split

> One logical world, a variable number of stateless shard processes, autoscaled by live population. No realm caps, no queues — the failure mode that kills WoW-likes, refused by design. → [architecture/02](../../../architecture/02-server-topology.md) · [architecture/03](../../../architecture/03-netcode-and-sharding.md)

## What it does
A **shard** = one authoritative process owning one zone/instance (or a slice of a hot zone). Shards are **stateless w.r.t. durable data** — everything ownable lives in Yugabyte ([persistence](../persistence/README.md)), so a shard can die and respawn. **k8s HPA** scales shard replicas by live player count. Players crossing a shard boundary are **handed off** server-side; hot zones **split**, quiet ones **merge**.

## Handoff (crossing a shard boundary)
- Boundary = a quadtree cell seam ([world-model](../world-model/README.md)). The source shard serializes the player's **transient sim state** (velocity, in-flight cast) and hands it to the target shard over the cross-shard bus.
- **Durable state is never moved** — it already lives in Yugabyte; handoff transfers only transient state. Target shard rebuilds the rest from durable state.
- Server issues entity IDs; the client sees continuous world, no loading screen ([world-model](../world-model/README.md) tunnel-masking).

## Merge / split (elastic population)
- **Split:** density crosses a threshold → a hot zone is split into parallel copies; the tick-loop overrun signal ([tick-loop](../tick-loop/README.md)) is one trigger.
- **Merge:** population falls → copies collapse into fewer shards.
- **Party coherence:** members follow the party leader's shard (within the same zone) — a group is never fragmented by an automatic shard shift.
- **Drain, don't yank:** merges are **event- and combat-aware** — never merge a shard hosting an active world event or boss; drain gracefully. (WoW's SoD Blood Moon desync is the cautionary tale.)
- **Anti-dupe on transfer:** entity-ID reconciliation on merge must not duplicate an item — and because ownership only ever moves via a Yugabyte transaction ([persistence](../persistence/README.md)), a merge/split cannot mint value even if IDs collide. Rockstar's own session-merge patents center this same dupe-guard.

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| WoW **sharding** (WoD 2014) | Auto-split a crowded zone into parallel copies, load-driven, invisible, no loading screen. Follow-the-party-leader assignment. | **Adopt** as our first-class split model + the party-leader follow rule. **Avoid** shard identity feeling arbitrary — make it stable and inspectable, transitions rare/predictable. |
| WoW **layering** (Classic 2019) | Whole-realm sticky copies at launch, carried across zone borders, collapsed as pop falls. Live-collapse mid-event broke things. | **Adopt** "sticky world-wide layer at launch, collapse as pop drops" as an explicit launch mode; **gate merges on event/combat state**. |
| WoW **CRZ / connected realms** | Merge low-pop realms into one shard; "realm" becomes a social/economic namespace, not a world boundary. | Our elastic shards make CRZ redundant as a mechanism — we merge natively. **Adopt** realm-as-namespace; **avoid** bolted-on timezone/type/excluded-zone special cases — one policy layer. |
| WoW **phasing / War Mode** | One coordinate resolves to different copies by story-phase + ruleset flag + shard-id + layer — four stacked systems. | **Adopt** a single **world-state key** `(story_phase, ruleset_flags, shard_id)` that deterministically maps a player to a copy. **Avoid** subdividing an opt-in pool along secondary axes (WoW's RP/Normal War-Mode split → imbalance). |
| **GTA Online** session merge/split (Take-Two patent US10814233) | Limited-capacity sessions merged by spatial proximity with player/object-ID reconciliation + explicit dupe-guard + session/volume locks; a thin coordinator holds only positions/velocity/locks. | **Adopt** proximity merge + ID reconciliation as **authoritative shard handoff** (server owns IDs, never a peer host). The patent's dupe-guard = our Yugabyte-transactional ownership move. |
| **GTA** request-based object ownership (Amazon patent US10911535) | Single-writer object ownership with explicit hand-off/claim + suspend/ack transfer preserving request order. | **Adopt** single-writer ownership + explicit handoff **arbitrated by the shard**, never peer-to-peer. |

## Rules
- Shards **never** hold authoritative ownership in memory-only — durable value is in Yugabyte; a shard crash loses nothing ownable.
- Never expose shard IPs — only edge/gateway is public ([security](../security/README.md)).
- Ownership changes never route through the bus/cache — direct to Yugabyte ([persistence](../persistence/README.md)).
- Offer a **low-elasticity mode** for communities that value a stable world (WoW RP realms opt out of sharding — immersion-sensitive players reject aggressive elasticity).

## Links
[tick-loop](../tick-loop/README.md) · [world-model](../world-model/README.md) · [persistence](../persistence/README.md) · [netcode](../netcode/README.md) · [security](../security/README.md)
