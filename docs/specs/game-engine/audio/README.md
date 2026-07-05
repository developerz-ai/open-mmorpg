# Audio

> A lean, all-Rust, **open-format** audio stack: Opus/Ogg assets through a mixer with per-emitter spatialization. Dependency-light, license-clean — no FMOD/Wwise C++ SDK. → [engine README](../README.md)

## What it does
Decodes and plays positional sound. Bevy's built-in `bevy_audio` handles Ogg/Vorbis (WAV/FLAC/MP3 behind features) but offers only **rudimentary** spatial audio (stereo pan + distance attenuation from transforms). For richer control the lean upgrade is the **Kira** mixer via [`bevy_kira_audio`](https://github.com/NiklasEi/bevy_kira_audio) — channels, tweens, spatial emitters. → [Bevy audio](https://bevy-cheatbook.github.io/audio.html).

## Design
- **Stack: Opus/Ogg → Kira mixer → per-emitter spatial channels.** All Rust, all open format ([world-and-assets](../../../architecture/06-world-and-assets.md)), staying inside the "open formats only, no proprietary tooling" rule ([CLAUDE.md](../../../../CLAUDE.md)). FMOD/Wwise pull in proprietary C++ SDKs — **rejected** on that rule.
- **Positional by ECS transform.** Emitters are entities; the listener is the camera/player. Spatialization reads the same transforms the [renderer](../rendering/README.md) does — one world, one source of position.
- **Audio is a plugin.** Added to the [core](../core/README.md) `App` for the headful client; the headless build simply doesn't add it — no stubs, no branching ([platform](../../client/platform/README.md)).
- **Interest-scoped.** Only emitters within the player's [AoI](../../game-server/world-model/README.md) are audible/loaded — the same relevance filter that bounds netcode and rendering; a shard's worth of sound never mixes at once.

## Distilled from the references
| Source | Adopt |
|---|---|
| Bevy audio | Simple positional playback for the baseline; fine for prototyping |
| Kira (`bevy_kira_audio`) | Channels/tweens/spatial emitters for production; lean Rust mixer |
| FMOD/Wwise | **Reject** — proprietary SDKs violate open-formats-only |
| Opus/Ogg | Open, compact codecs; the shipping audio format |

## Rules
- **Open codecs only** — Opus/Ogg. No proprietary audio middleware ([CLAUDE.md](../../../../CLAUDE.md)).
- Audio is a **headful-only plugin** — never linked into the headless core ([platform](../../client/platform/README.md)).
- Emitters are AoI-scoped — don't mix sound for entities the player can't perceive ([world-model](../../game-server/world-model/README.md)).

## Links
[core](../core/README.md) · [rendering](../rendering/README.md) · [client audio](../../client/audio/README.md) · [platform](../../client/platform/README.md)
