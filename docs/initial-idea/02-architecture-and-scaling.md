# 02 — Architecture & Scaling (target: 1M+ concurrent)

> Deep dive: [../architecture/](../architecture/README.md). This is the intent; that folder is the design.

## Core pattern
- **Zone/realm sharding** — stateless game-logic servers per zone instance, autoscaled by live player count (k8s HPA). No fixed realm caps, no queues.
- **Gateway layer** — auth + connection routing to the correct shard. Never expose shard IPs.
- **Cross-shard messaging** — pub/sub (NATS or Dragonfly streams) for chat, guild, auction house, cross-zone events.
- **Seamless world** — quadtree-based zone streaming; no loading screens *by design*.
- **Session merge/split** — dynamically merge/split shard instances into one persistent world as population shifts (concept studied from Rockstar's approach — [09-gta6-inspiration.md](09-gta6-inspiration.md)).

## DDoS protection
- Cloudflare Spectrum / AWS Shield in front of game ports (TCP/UDP) — never expose game-server IPs.
- L7 rate-limiting + connection-churn detection at the gateway (login floods, packet spam).
- Anycast routing — spread attack surface across edge nodes, not origin.

## Anti-cheat / anti-dupe (top priority — this is what kills private servers)
- **All ownership-changing transactions** (trades, currency, item transfers) go through the strongly-consistent DB layer **directly** — never through the fast/eventual cache. This is the #1 dupe-prevention rule.
- **Server-authoritative everything.** Client never trusted for state.

## Scaling philosophy
- **Horizontal by default from day 1** — not retrofitted like retail WoW's realm architecture.
- Autoscale relative to real-time population, not static provisioning.
