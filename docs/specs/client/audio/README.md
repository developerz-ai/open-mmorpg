# Client Audio

> Wiring the [engine audio](../../game-engine/audio/README.md) into the game: the listener follows the camera/player, emitters attach to visible entities, and mixing is **AoI-scoped**. Thin — the mixer and format decisions live in the engine. → [engine audio](../../game-engine/audio/README.md)

## What it does
Places the audio **listener** on the local player/camera and spawns positional **emitters** on entities and world events (footsteps, abilities, ambient), mixed through the engine's Kira-based spatial stack over Opus/Ogg ([engine audio](../../game-engine/audio/README.md)). Only sound within the player's [area of interest](../../game-server/world-model/README.md) is mixed.

## Design
- **Listener = the predicted player.** Spatialization reads the same predicted/interpolated transforms the [renderer](../rendering/README.md) uses — audio and visuals share one world position ([prediction-core](../prediction-core/README.md)).
- **Emitters are AoI-scoped.** Sound sources exist only for entities in the render/AoI set — a shard's worth of audio never mixes; the same relevance filter bounds bandwidth, draw calls, and now voices ([world-model](../../game-server/world-model/README.md), [networking](../networking/README.md)).
- **Event-driven from snapshots.** Ability/impact sounds fire off server-confirmed events in snapshots (or predicted locally then confirmed) — audio reflects authoritative state, it never asserts it ([security](../../game-server/security/README.md)).
- **Headful-only plugin.** The headless client adds no audio — nothing to stub, nothing to branch ([platform](../platform/README.md), [engine audio](../../game-engine/audio/README.md)).

## Distilled from the references
| Source | Adopt |
|---|---|
| [Engine audio](../../game-engine/audio/README.md) | Kira mixer, Opus/Ogg, per-emitter spatial — inherited |
| [World-model](../../game-server/world-model/README.md) AoI | Voices scoped to the interest set, like bandwidth and draws |
| [Prediction core](../prediction-core/README.md) | Listener/emitter positions from predicted/interpolated transforms |

## Rules
- **Open codecs only** — Opus/Ogg; no proprietary audio middleware ([CLAUDE.md](../../../../CLAUDE.md)).
- Emitters are **AoI-scoped** — never mix sound for imperceptible entities ([world-model](../../game-server/world-model/README.md)).
- Audio reflects authoritative/predicted state; it never asserts state ([security](../../game-server/security/README.md)).
- Headful-only plugin — never linked into the headless core ([platform](../platform/README.md)).

## Links
[rendering](../rendering/README.md) · [networking](../networking/README.md) · [platform](../platform/README.md) · [engine audio](../../game-engine/audio/README.md) · [world-model](../../game-server/world-model/README.md)
