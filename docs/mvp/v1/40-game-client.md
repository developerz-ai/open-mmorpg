# 40 — Game Client · Wave 3

> **Big idea.** The **thin, honest player app**: it sends [`Intent`](../../../crates/protocol/src/lib.rs), predicts locally with the *same* [`sim`](../../../crates/sim/src/lib.rs) the server runs, and renders what that predicts — it **never asserts state**. One deterministic core runs **headless** (tests, CI, AI agents) or **headful** (rendered). The engineering is making prediction feel instant over a lossy network, and making the exact same core **agent-drivable headless**. **Owner: [`gameclient`](../../../.claude/agents/gameclient.md).** → [client specs](../../specs/client/README.md)

## Reads
[client README](../../specs/client/README.md) and each subsystem: [prediction-core](../../specs/client/prediction-core/README.md) · [networking](../../specs/client/networking/README.md) · [rendering](../../specs/client/rendering/README.md) · [input](../../specs/client/input/README.md) · [hud-ui](../../specs/client/hud-ui/README.md) · [audio](../../specs/client/audio/README.md) · [platform](../../specs/client/platform/README.md) · [03 netcode-and-sharding](../../architecture/03-netcode-and-sharding.md) · [07 mcp-companions](../../architecture/07-mcp-companions.md).

## Depends on
[Foundation](00-foundation.md) `protocol`; [Server](30-game-server.md) `crates/sim` (shared) + gateway handshake + snapshot format; [Engine](20-game-engine.md) render/UI/audio/input plugins. Builds `apps/client`.

## PR batches (~100 PRs)
| Batch | Scope | Spec | Definition of Done |
|---|---|---|---|
| **C1 · Headless core** | The prediction core as `MinimalPlugins` — no GPU. Loads shared `sim`. **This ships first: it's the AI-agent target.** | [prediction-core](../../specs/client/prediction-core/README.md) · [platform](../../specs/client/platform/README.md) | Headless client boots, ticks `sim`, no renderer. Agent-drivable. |
| **C2 · Networking ingest** | Client transport, snapshot ingest, interpolation buffer, connection to gateway→shard. | [networking](../../specs/client/networking/README.md) | Connects, receives AoI snapshots, buffers for interpolation. |
| **C3 · Prediction + reconciliation** | Predict locally with `sim`, reconcile against authoritative snapshot, replay unacked intents. Bit-aligned with server. | [prediction-core](../../specs/client/prediction-core/README.md) | Predicted state == server re-sim; reconciliation is smooth, no rubber-band on clean links. |
| **C4 · Input → Intent** | Device input → logical actions → `Intent`; rebindable keybinds, gamepad (`leafwing-input-manager`). | [input](../../specs/client/input/README.md) | Movement/ability inputs become validated `Intent`s; rebinding works. |
| **C5 · Rendering wiring** | Drive the [engine renderer](20-game-engine.md): camera, LOD/AoI budget, character + crowd rendering. | [rendering](../../specs/client/rendering/README.md) | Slice zone renders; AoI budget respected. |
| **C6 · HUD/UI** | HUD, nameplates, minimap, inventory — **i18n from day one**, on the [engine UI substrate](20-game-engine.md). | [hud-ui](../../specs/client/hud-ui/README.md) | Playable HUD; every string via `t()`; missing keys render loud. |
| **C7 · Audio wiring** | Listener, emitters, AoI-scoped mixing on the [engine audio](20-game-engine.md) stack. | [audio](../../specs/client/audio/README.md) | Spatial SFX follow entities within AoI. |
| **C8 · Platform builds** | Headless vs headful; native (AAA path) vs web (baseline WebGPU); operator can ship their own build. | [platform](../../specs/client/platform/README.md) | Native + web builds run the slice; operators reskin without touching the core. |
| **C9 · Agent-drive harness** | The headless client as a first-class MCP/agent target: connect, feed intents, assert predicted/authoritative state ([companions](../../architecture/07-mcp-companions.md)). | [ai-native-dx](../../specs/game-engine/ai-native-dx/README.md) | An agent plays the slice end-to-end headless and asserts state in CI — this feeds the [release gate](70-integration-and-release.md). |

## Interfaces this track owns
- **`apps/client`** — headless + headful player app. The headless mode is the **E2E test driver** the [release gate](70-integration-and-release.md) uses.
- **Consumes** `crates/sim` (verbatim), `protocol`, the engine plugins, the gateway handshake — owns none of them.

## Rules
- **Client sends intent, never state** — every `Intent` validated server-side; the client is never trusted.
- **Same `sim` client and server** — prediction lines up bit-for-bit because both run the deterministic `sim`. Never fork sim logic client-side.
- **One core, headless or headful** — no logic branches on "are we rendering?"; the rendered client = headless core + engine plugins.
- **Agent-drivable by construction** — if the headless client can't be driven and asserted in CI, the batch isn't done.
- **i18n from day one**; **files ≤300 LOC, no `unwrap` on the frame/tick path.**

## Definition of Done (track)
A player launches native or web, connects, moves and fights with instant-feeling prediction that reconciles cleanly; the *same* core runs headless and an agent plays the full slice asserting state in CI; operators can ship a reskinned build on the same prediction core.

## Links
[client specs](../../specs/client/README.md) · [server](30-game-server.md) · [engine](20-game-engine.md) · [`gameclient` subagent](../../../.claude/agents/gameclient.md)
