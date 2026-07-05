# Rendering

> Unreal-class visuals on **wgpu**: a GPU-driven, clustered-forward+ PBR pipeline with virtual geometry, real-time GI, virtualized shadows, and temporal upscaling — **native-desktop for the AAA path, a clean baseline everywhere else.** We ship what Bevy has, fill the gaps, and document what we can't yet match. → [engine README](../README.md) · [world-and-assets](../../../architecture/06-world-and-assets.md)

## What it does
Turns the ECS world into frames. **Metallic-roughness PBR** from glTF ([assets](../assets/README.md)) via Bevy's `StandardMaterial`, a **clustered forward+** renderer (lights bucketed per view-frustum cluster; optional deferred + prepass), and **GPU-driven** draw submission (indirect multi-draw + bindless on native desktop) so CPU draw-call cost stays flat as scenes grow. → [Bevy 0.16 GPU-driven rendering](https://bevy.org/news/bevy-0-16/), [glTF 2.0 PBR](https://www.khronos.org/gltf/).

## The AAA techniques (and where we stand)
| Technique | Unreal | Ours (Bevy/wgpu) | Ship plan |
|---|---|---|---|
| **Virtual geometry** | [Nanite](https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine): meshlet DAG, GPU cull, per-pixel LOD, streamed | Bevy [`meshlet`](https://jms55.github.io/posts/2025-03-27-virtual-geometry-bevy-0-16/) feature — compute cluster-cull + software raster, **experimental** | Meshlets for hero/high-poly static assets; discrete [LOD chains](../assets/README.md) + imposters elsewhere |
| **Real-time GI** | [Lumen](https://dev.epicgames.com/documentation/unreal-engine/lumen-technical-details-in-unreal-engine): SW (distance-field) + HW ray tracing, any GPU | Bevy [Solari](https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18/) (ReSTIR DI+GI) — HW-RT, **NVIDIA-only**, experimental | **Baked irradiance volumes + light probes as default**; Solari opt-in on RTX; SSAO always |
| **Shadows** | [Virtual Shadow Maps](https://dev.epicgames.com/documentation/unreal-engine/virtual-shadow-maps-in-unreal-engine) (16K, page-cached) | Cascaded Shadow Maps + contact shadows (screen-space) | CSM + contact shadows now; track VSM upstream |
| **Anti-aliasing / upscaling** | [TSR](https://www.techpowerup.com/review/ghostwire-tokyo-dlss-vs-tsr-vs-fsr-comparison/) (cross-vendor) + DLSS/FSR3/XeSS | MSAA/FXAA/SMAA/TAA/CAS + DLSS (0.17+) | TAA+CAS baseline; DLSS on RTX; **no cross-vendor temporal upscaler yet** |

## Honest gaps vs Unreal
1. **GI without RT hardware** — no Lumen-software / DDGI equivalent in Bevy. We lean on **baked** irradiance volumes; real-time GI is an RTX-only opt-in. Documented, not hidden ([tech-stack honesty note](../../../initial-idea/03-tech-stack.md#client)).
2. **No cross-vendor temporal upscaler** — quality upscaling is DLSS (NVIDIA) only; AMD/Intel/web get spatial AA.
3. **Meshlets + VSM are experimental/absent.** Treat virtual geometry as opt-in for specific assets, not the default path, until it stabilizes upstream.
4. **wgpu ceiling.** Bindless is an experimental native-only extension; **mesh shaders and ray tracing don't exist on the web target** — the browser client runs the baseline forward path only ([platform](../../client/platform/README.md)).

## Design rules for the pipeline
- **Tiered feature levels.** `Ultra` (native, meshlets + Solari + DLSS) · `High` (native, LOD + baked GI + TAA) · `Web` (WebGPU baseline, forward + SMAA). One material set, capabilities queried at boot, no per-feature code forks.
- **Everything is a render plugin.** The renderer is added to the [core](../core/README.md) `App`; removing it yields the headless build with zero logic change ([platform](../../client/platform/README.md)).
- **Budget-driven.** Frame time and draw count are instrumented from day one, same discipline as the netcode [bandwidth budget](../../game-server/netcode/README.md). LOD/AoI ([world-model](../../game-server/world-model/README.md)) cap what's ever submitted.

## Distilled from the references
| Source | Adopt |
|---|---|
| Nanite | Meshlet DAG + GPU cluster culling for high-poly static geometry |
| Lumen | GI is layered: screen-space → world-space fallback; amortize over frames. We do the baked-first version |
| VSM | Page-cached shadows are the goal; ship CSM + contact shadows meanwhile |
| Bevy 0.16+ | GPU-driven + bindless on desktop; clustered forward+ as the base |
| wgpu | One API, all platforms — but design to the **WebGPU baseline**, gate the AAA path behind native capability checks |

## Rules
- Never ship a technique that only works on one vendor as the **default** — it's a tier, gated by capability query.
- Web target = baseline forward renderer. Don't assume bindless/RT/mesh shaders exist ([platform](../../client/platform/README.md)).
- Materials are metallic-roughness glTF PBR, loaded as data — no hand-authored shaders per asset ([assets](../assets/README.md)).

## Links
[assets](../assets/README.md) · [animation](../animation/README.md) · [platform](../../client/platform/README.md) · [client rendering](../../client/rendering/README.md) · [world-and-assets](../../../architecture/06-world-and-assets.md)
