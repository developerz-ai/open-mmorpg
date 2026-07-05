# Netcode

> Move authoritative state to hundreds of nearby clients at 30 Hz within a sane bandwidth budget, over a lossy network, without ever trusting the client. → [architecture/03](../../../architecture/03-netcode-and-sharding.md)

## What it does
Carries [`ClientMsg`](../../../../crates/protocol/src/lib.rs) intents up and [`ServerMsg`](../../../../crates/protocol/src/lib.rs) snapshots down. The shard is authoritative; the client **predicts locally and reconciles** to the server. [`protocol`](../../../../crates/protocol/src/lib.rs) is the single wire source of truth.

## Transport
| Traffic | Channel | Why |
|---|---|---|
| Snapshots / position | **unreliable UDP datagram** | newest wins; never resend — a lost snapshot is superseded by the next ([Quake3 model](https://fabiensanglard.net/quake3/network.php)) |
| RPC / chat / ability cast / economy | **reliable-ordered** | must arrive exactly once, in order |
| Auth / session | axum over TLS (gateway) | request/response |
| Browser client | QUIC / WebTransport | reliable streams + datagrams, no cross-stream head-of-line block |

**Never TCP for game state** — one lost packet head-of-line-blocks everything behind it and retransmits stale position ([Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)). Start on **`renet`** (UDP + Netcode.io, channel model built in) for the native shard; keep transport behind a trait so **`aeronet`/`quinn`** (QUIC+WebTransport) can back the wasm client later.

## Reliability layer
Pure-math, socket-free, in [`netcode`](../../../../crates/netcode/src/lib.rs): wrapping 16-bit sequence numbers ([`seq_greater_than`](../../../../crates/netcode/src/lib.rs), RFC 1982) + a 32-entry ack bitfield ([`AckTracker`](../../../../crates/netcode/src/lib.rs)). Each packet's ack is piggybacked and effectively sent 32× → survives loss. This is the [Gaffer On Games reliability model](https://gafferongames.com/post/reliable_ordered_messages/).

## Prediction & reconciliation (the canonical loop)
1. Client stamps each [`Intent`](../../../../crates/protocol/src/lib.rs) with a sequence, sends it, **and applies it immediately** via the same deterministic [`sim::step`](../../../../crates/sim/src/lib.rs) — no RTT wait.
2. Server processes inputs authoritatively; each [`Snapshot`](../../../../crates/protocol/src/lib.rs) echoes the **last input sequence processed** for that client.
3. On snapshot, client snaps its entity to authoritative state, then **replays all unacked inputs** on top. Match → invisible; mismatch → smooth correction.

Because `sim` is deterministic and shared client/server, prediction and server re-sim line up exactly. → [Gambetta](https://www.gabrielgambetta.com/client-server-game-architecture.html).

## Snapshots, delta, interpolation
- **Per-client acked baseline + delta.** Send only changed components, delta-compressed against the last snapshot that client acked; a dropped snapshot just deltas against an older baseline — nothing resent (Quake3/Source).
- **Quantize + bit-pack** before send: mm-quantized positions, "smallest-three" quaternions, drop velocity (interp covers it). Fiedler: ~98% reduction on dense state.
- **Remote entities interpolated in the past** by ~2 snapshot intervals (~66 ms at 30 Hz) so a late packet has a successor to interpolate toward. Local player predicted; everyone else interpolated.
- **Interest-filtered** — a client only receives entities in its AoI ([world-model](../world-model/README.md)). This makes bandwidth **O(nearby)**, not O(world).

## Bandwidth budget
Naive 100 entities × 24 B × 30 Hz ≈ 0.5 Mbit/s/client — untenable at scale. Three multipliers stack: **AoI** (cap replicated entities to the visible set) → **delta** (only changed fields) → **quantization** (~2–4×). Budget **~128–256 kbit/s down/client**; the replication system fills a per-client per-tick byte budget by priority (**self > combat > near > far**), low-priority entities skip ticks when over budget. Instrument bytes/client/tick from day one.

## Distilled from the references
| Source | Adopt |
|---|---|
| Quake3 / Source snapshot model | single delta-vs-acked-baseline packet type; unreliable state, reliable RPC |
| Gaffer On Games | ack-bitfield reliability (already in [`netcode`](../../../../crates/netcode/src/lib.rs)); snapshot quantization |
| Gambetta | predict + reconcile via replayed unacked inputs |
| Overwatch (GDC) | fixed command tick, adaptive send rate (20→60 Hz "high bandwidth") |
| GTA (P2P) — *avoid* | dead-reckoning is fine, but **never** peer-authoritative; snapshots come from the authoritative shard ([security](../security/README.md)) |

## Lag compensation
Server keeps a bounded per-tick snapshot history. **Rewind only for hitscan/skillshot** validation (cap ~200–300 ms); tab-target/ability combat validates against present state — full FPS rewind is overkill and a wider cheat window ([combat](../combat/README.md)).

## Rules
- Client never asserts state — intents only; server validates every one ([security](../security/README.md)).
- No `std::sync::Mutex` on the send/tick path; `tokio::sync` only.
- Every snapshot carries its [`Tick`](../../../../crates/protocol/src/lib.rs). Wire changes are one edit in [`protocol`](../../../../crates/protocol/src/lib.rs).

## Links
[tick-loop](../tick-loop/README.md) · [world-model](../world-model/README.md) · [sharding](../sharding/README.md) · [security](../security/README.md)
