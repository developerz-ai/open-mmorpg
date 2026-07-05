# Prediction Core

> The heart of the client: it runs the **same deterministic [`sim`](../../../../crates/sim/src/lib.rs)** the server runs, predicts locally from the player's own intents with no RTT wait, and reconciles to authoritative snapshots. Headless and pure — the part that runs in CI and under AI agents today. → [apps/client](../../../../apps/client/src/main.rs) · [netcode spec](../../game-server/netcode/README.md)

## What it does
Given the last acknowledged authoritative [`EntityState`](../../../../crates/sim/src/lib.rs) and the player's unacked [`Intent`](../../../../crates/protocol/src/lib.rs)s, it produces the predicted present via [`simulate`](../../../../crates/sim/src/lib.rs) — the exact function the [shard](../../game-server/tick-loop/README.md) uses. Because `sim` is deterministic and shared, prediction and server re-sim agree bit-for-bit. This is the [Gambetta](https://www.gabrielgambetta.com/client-side-prediction-server-reconciliation.html) model, and it's already the shipped [`apps/client`](../../../../apps/client/src/main.rs) core.

## The canonical loop (client side)
1. **Predict.** Stamp each `Intent` with a sequence, send it, **and apply it immediately** via `sim::step` — the local player moves with zero input latency ([netcode](../../game-server/netcode/README.md)).
2. **Reconcile.** Each [`Snapshot`](../../../../crates/protocol/src/lib.rs) echoes the last input sequence the server processed. On arrival: snap the local entity to authoritative state, then **replay all still-unacked inputs** on top. Match → invisible; mismatch → smooth correction, not a snap ([networking](../networking/README.md)).
3. **Interpolate others.** Remote entities aren't predicted — they're rendered ~2 snapshot intervals in the past by interpolating buffered snapshots ([networking](../networking/README.md)).

## Design
- **Determinism is the contract.** Fixed timestep (`FixedUpdate`), stable ordering, no wall-clock or unseeded randomness — the same discipline the [engine core](../../game-engine/core/README.md) and [server](../../game-server/tick-loop/README.md) hold. Break it and prediction diverges from authority.
- **Pure and headless.** The core has no renderer, no window, no I/O beyond intents in / state out — CI-friendly and agent-drivable ([platform](../platform/README.md)). The [renderer](../rendering/README.md) draws whatever this predicts; it's added on top, never entangled.
- **Client never asserts state.** It predicts for *feel*; the server is the sole authority. A divergence is corrected by the server, and large/repeated divergence is a [cheat signal](../../game-server/security/README.md), not a client override.

## Distilled from the references
| Source | Adopt |
|---|---|
| Gambetta (predict + reconcile) | Apply local inputs immediately; replay unacked inputs on each snapshot |
| Overwatch (GDC) | Fixed command tick; deterministic shared sim client/server |
| Our [netcode spec](../../game-server/netcode/README.md) | Snapshot echoes last-processed input seq; delta + AoI upstream |
| [apps/client](../../../../apps/client/src/main.rs) | The headless prediction core is the shipped starting point — build headful on top |

## Rules
- **Same `sim`, both sides.** Never fork prediction logic from server logic — one [`crates/sim`](../../../../crates/sim/src/lib.rs) ([CLAUDE.md](../../../../CLAUDE.md)).
- **Deterministic sim path** — fixed dt, stable order, no wall-clock/random ([engine core](../../game-engine/core/README.md)).
- **Predict, don't assert.** The client corrects to the server, never the reverse ([security](../../game-server/security/README.md)).
- Core stays **pure/headless** — the renderer is added by the headful build, not depended on here ([platform](../platform/README.md)).

## Links
[networking](../networking/README.md) · [rendering](../rendering/README.md) · [platform](../platform/README.md) · [netcode](../../game-server/netcode/README.md) · [tick-loop](../../game-server/tick-loop/README.md) · [engine core](../../game-engine/core/README.md)
