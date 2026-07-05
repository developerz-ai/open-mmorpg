# 🎮 Client Specs

> **Scope:** the player-facing app built **on the [game-engine](../game-engine/README.md)**, talking to the [game-server](../game-server/README.md). One deterministic core runs **headless** (tests, CI, AI agents) or **headful** (rendered) — the renderer is a plugin you add, not a dependency you can't remove. The engine folder is the reusable layer; this folder is *our game's* client. → [architecture/03](../../architecture/03-netcode-and-sharding.md)

## Thesis
The client is **thin and honest**: it sends [`Intent`](../../../crates/protocol/src/lib.rs), predicts locally with the *same* [`sim`](../../../crates/sim/src/lib.rs) the server runs, and renders whatever that predicts. It never asserts state. The interesting engineering is (1) making prediction/interpolation feel instant over a lossy network, and (2) making the **exact same core** drivable by an AI agent headless — connect, feed intents, assert predicted state ([apps/client](../../../apps/client/src/main.rs)).

## Index
| Spec | Subsystem | Distilled from |
|---|---|---|
| [prediction-core/](prediction-core/README.md) | Shared deterministic `sim`, headless core, prediction + reconciliation | Gambetta, Overwatch; [apps/client](../../../apps/client/src/main.rs) |
| [networking/](networking/README.md) | Client transport, snapshot ingest, interpolation buffer, reconciliation glue | Quake3/Source, Gaffer; [netcode spec](../game-server/netcode/README.md) |
| [rendering/](rendering/README.md) | Driving the engine renderer: camera, LOD/AoI budget, character/crowd rendering | [engine rendering](../game-engine/rendering/README.md) |
| [input/](input/README.md) | Device input → logical actions → `Intent`; rebindable keybinds, gamepad | `leafwing-input-manager` |
| [hud-ui/](hud-ui/README.md) | HUD, nameplates, minimap, inventory; i18n from day one | [engine UI](../game-engine/ui/README.md); ICU/Fluent |
| [audio/](audio/README.md) | Client audio wiring: listener, emitters, AoI-scoped mixing | [engine audio](../game-engine/audio/README.md) |
| [platform/](platform/README.md) | Headless vs headful; native (AAA path) vs web (baseline); operator client builds | Bevy `MinimalPlugins`, wgpu/WebGPU |

## Non-negotiable principles (inherited → [CLAUDE.md](../../../CLAUDE.md))
1. **Client sends intent, never state.** Every [`Intent`](../../../crates/protocol/src/lib.rs) is validated server-side; the client is never trusted ([security](../game-server/security/README.md)).
2. **Same `sim` client and server.** Prediction and server re-sim line up bit-for-bit because both run the deterministic [`sim`](../../../crates/sim/src/lib.rs) ([prediction-core](prediction-core/README.md)).
3. **One core, headless or headful.** The rendered client = the headless core + the engine's render/UI/audio plugins. No logic forks on "are we rendering?" ([platform](platform/README.md), [engine core](../game-engine/core/README.md)).
4. **Agent-drivable by construction.** The headless client is a first-class AI-agent target — connect, feed intents, assert state ([ai-native-dx](../game-engine/ai-native-dx/README.md), [mcp-companions](../../architecture/07-mcp-companions.md)).
5. **Operators ship their own builds.** Extra maps, modified rules — on top of the same prediction core. Core stays strong; the surface is theirs ([platform](platform/README.md)).

## Client ↔ engine ↔ server
```
 device input ─▶ input ─▶ Intent ──(netcode)──▶ shard (authoritative sim)
                                                     │ snapshot (delta, AoI-filtered)
 render ◀─ engine renderer ◀─ prediction-core ◀──────┘
   ▲              ▲                  ▲
  hud-ui        audio         (same crates/sim as server)
```
The **left column is the [game-engine](../game-engine/README.md)** (reusable); the **arrows are our game**. Headless drops the render/hud/audio row and keeps prediction-core + networking + input intact.

## Reference index
**Internal:** [game-engine specs](../game-engine/README.md) · [game-server specs](../game-server/README.md) · [architecture/03 netcode](../../architecture/03-netcode-and-sharding.md) · [apps/client](../../../apps/client/src/main.rs) · [`crates/sim`](../../../crates/sim/src/lib.rs) · [`crates/protocol`](../../../crates/protocol/src/lib.rs).
**External** (per-spec inline): [Gambetta client-server architecture](https://www.gabrielgambetta.com/client-server-game-architecture.html) · [Valve Source networking](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking) · [Gaffer On Games](https://gafferongames.com/) · Overwatch GDC netcode · [Bevy web/WebGPU](https://bevy.org/news/bevy-webgpu/) · [leafwing-input-manager](https://github.com/Leafwing-Studios/leafwing-input-manager).

> Each doc ≤ ~1 screen: what it does, the design, distilled lesson, rules, links. Grows past that → split it.
