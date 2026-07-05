# 02 — Server Topology

```
                    Cloudflare Spectrum / Anycast edge  (DDoS)
                                  │
                            ┌─────────────┐
   client ───TLS/QUIC──────▶│   gateway   │  axum · auth · issues session token · routes to shard
                            └─────┬───────┘
              ┌───────────────────┼───────────────────┐
        ┌─────▼─────┐       ┌─────▼─────┐       ┌─────▼─────┐
        │  shard A  │       │  shard B  │  ...  │  shard N  │   headless zone servers (tokio + ECS)
        │ (zone/    │       │           │       │           │   autoscaled by live pop (k8s HPA)
        │  instance)│       └─────┬─────┘       └───────────┘
        └─────┬─────┘             │
              └──────── pub/sub bus (NATS or Dragonfly streams) ────────┐
                      chat · guild · AH · world events · presence        │
                                  │                                      │
                        ┌─────────▼─────────┐              ┌─────────────▼────────────┐
                        │  worldsvc (apps/) │              │  YugabyteDB (ownership)  │
                        │  cross-shard svcs │              │  Dragonfly (ephemeral)   │
                        └───────────────────┘              └──────────────────────────┘
```

## Components
| Component | Role | Crate/app |
|---|---|---|
| **gateway** | auth, connection routing, session tokens, L7 rate-limit | `apps/gateway` |
| **shard** | one zone/instance, authoritative sim, tick loop | `apps/shard` |
| **worldsvc** | chat, guild, auction house, world feed, presence | `apps/worldsvc` |
| **cross-shard bus** | pub/sub for anything spanning shards | NATS or Dragonfly streams |
| **state** | ownership (Yugabyte) + ephemeral (Dragonfly) | `crates/persistence`, `crates/cache` |

## Autoscaling & queues
- Shards are **stateless w.r.t. durable data** — all durable state in Yugabyte; a shard can die and respawn.
- **k8s HPA** scales shard replicas by live player count per zone. No fixed realm caps → **no queues**.
- Population shifts trigger **zone/session merge-split** ([03](03-netcode-and-sharding.md)) so one logical world spans a variable shard count seamlessly.

## Never
- Never expose shard IPs directly (only the edge/gateway is public).
- Never route ownership changes through the bus/cache — direct to Yugabyte ([04](04-data-and-consistency.md)).
