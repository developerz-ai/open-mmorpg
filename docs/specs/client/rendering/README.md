# Client Rendering

> How our game drives the [engine renderer](../../game-engine/rendering/README.md): camera, feeding the predicted/interpolated world to the GPU, and holding the frame within an **LOD/AoI budget** so a crowded city still hits frame rate. Thin — the heavy lifting is the engine's; this is the game-specific wiring. → [engine rendering](../../game-engine/rendering/README.md)

## What it does
Takes the state the [prediction core](../prediction-core/README.md) produces — local player predicted, remotes interpolated ([networking](../networking/README.md)) — and hands it to the engine's GPU-driven PBR pipeline for drawing: camera follow, character/crowd rendering, world-tile display. The renderer is a **plugin on the shared core** ([engine core](../../game-engine/core/README.md)); removing it yields the headless client with identical game logic ([platform](../platform/README.md)).

## Design
- **Draw the prediction, not the network.** The local player renders from the predicted transform (zero input latency); remotes render from the interpolation buffer (smooth over jitter). Reconciliation corrections are **smoothed** across frames, never snapped ([prediction-core](../prediction-core/README.md)).
- **AoI = render set.** The client only *has* entities in its [area of interest](../../game-server/world-model/README.md), so the render set is naturally bounded — the same quadtree filter that caps bandwidth caps draw calls. Tile streaming loads world geometry by camera position, no loading screens ([assets](../../game-engine/assets/README.md)).
- **Character LOD tiers.** Local player + nearby = full skeletal + [animation graph](../../game-engine/animation/README.md); mid = reduced-bone skeletal; distant crowd = **VAT instances**. Keeps hundreds of visible characters affordable ([animation](../../game-engine/animation/README.md)).
- **Feature tier by platform.** Native desktop gets the AAA path (meshlets/GI/DLSS where present); web gets the baseline forward renderer — one material set, capability-gated, no code fork ([engine rendering](../../game-engine/rendering/README.md), [platform](../platform/README.md)).
- **Budget-instrumented.** Frame time + draw count tracked from day one, mirroring the netcode [bandwidth budget](../../game-server/netcode/README.md); over budget → drop distant LOD tiers first.

## Distilled from the references
| Source | Adopt |
|---|---|
| [Engine rendering](../../game-engine/rendering/README.md) | GPU-driven PBR, feature tiers, honest gaps — inherited wholesale |
| Gambetta/Gaffer | Render predicted-local + interpolated-remote; smooth corrections |
| [World-model](../../game-server/world-model/README.md) AoI | Render set = AoI set; O(nearby) draw cost |
| [Animation](../../game-engine/animation/README.md) VAT tiers | Distance-tiered character rendering for MMO crowds |

## Rules
- **Renderer is a removable plugin** — headless client = this minus rendering, same logic ([platform](../platform/README.md)).
- Render **predicted local / interpolated remote**; smooth reconciliation, never teleport.
- Render set is **AoI-bounded** — never attempt to draw the whole shard.
- Web target = **baseline forward** renderer; don't assume the AAA path exists ([engine rendering](../../game-engine/rendering/README.md)).

## Links
[prediction-core](../prediction-core/README.md) · [networking](../networking/README.md) · [hud-ui](../hud-ui/README.md) · [platform](../platform/README.md) · [engine rendering](../../game-engine/rendering/README.md) · [engine animation](../../game-engine/animation/README.md)
