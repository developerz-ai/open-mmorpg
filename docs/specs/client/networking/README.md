# Client Networking

> The client half of [netcode](../../game-server/netcode/README.md): send stamped intents, ingest AoI-filtered delta snapshots, de-jitter them into a smooth **interpolation buffer**, and feed reconciliation. Transport is pluggable — UDP native, WebTransport in the browser. → [architecture/03](../../../architecture/03-netcode-and-sharding.md)

## What it does
Carries [`ClientMsg`](../../../../crates/protocol/src/lib.rs) intents up (stamped with a sequence) and receives [`ServerMsg`](../../../../crates/protocol/src/lib.rs) snapshots down. Snapshots are **unreliable** (newest wins, never resent); RPC/chat/ability/economy are **reliable-ordered**. The transport sits behind a trait so the native client uses **`renet`** (UDP) and the web client **`aeronet`/WebTransport** (QUIC) without touching game code ([netcode transport table](../../game-server/netcode/README.md)).

## Design
- **Interpolation buffer.** Remote entities are rendered ~2 snapshot intervals (~66 ms at 30 Hz) **in the past**, interpolating between the two most recent buffered snapshots — a late packet has a successor to interpolate toward, so motion stays smooth over jitter/loss ([Gambetta entity interpolation](https://www.gabrielgambetta.com/entity-interpolation.html), [Gaffer snapshot interpolation](https://gafferongames.com/post/snapshot_interpolation/)). Local player is **predicted**, not interpolated ([prediction-core](../prediction-core/README.md)).
- **Reconciliation glue.** On each snapshot the client hands the authoritative state + echoed last-input-seq to the [prediction core](../prediction-core/README.md), which snaps + replays unacked inputs. Correction is **smoothed** over a few frames — never a visible teleport.
- **Extrapolate briefly, then wait.** If the next snapshot is missing, extrapolate a remote entity a short bounded time, then hold — never invent state ([Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)).
- **Interest-scoped receive.** The client only gets entities in its [AoI](../../game-server/world-model/README.md), so bandwidth and decode are **O(nearby)**, not O(world) — this is what caps client cost as the shard fills.
- **Reliability is server-shared math.** Wrapping 16-bit sequences + ack bitfield live in [`crates/netcode`](../../../../crates/netcode/src/lib.rs) — the client uses the same socket-free layer the server does ([Gaffer reliability](https://gafferongames.com/post/reliable_ordered_messages/)).

## Distilled from the references
| Source | Adopt |
|---|---|
| Quake3/Source snapshot model | Unreliable state (newest wins), reliable RPC; per-client delta baseline |
| Gambetta / Gaffer | Interpolation buffer for remotes; predict + reconcile for local |
| Valve Source | Bounded extrapolation on packet loss; interp delay ~100 ms |
| Our [netcode spec](../../game-server/netcode/README.md) | Same wire types ([protocol](../../../../crates/protocol/src/lib.rs)), same reliability crate, AoI filtering |

## Rules
- **Intents only up, never state** — every intent is validated server-side ([security](../../game-server/security/README.md)).
- **Local predicted, remote interpolated** — never predict entities you don't control.
- **Smooth corrections**, don't snap; extrapolation is bounded, never fabricated state.
- Transport behind a trait — UDP native, WebTransport web; game code is transport-agnostic ([platform](../platform/README.md)).
- Wire changes are one edit in [`crates/protocol`](../../../../crates/protocol/src/lib.rs) — the single source of truth.

## Links
[prediction-core](../prediction-core/README.md) · [rendering](../rendering/README.md) · [platform](../platform/README.md) · [netcode](../../game-server/netcode/README.md) · [world-model](../../game-server/world-model/README.md)
