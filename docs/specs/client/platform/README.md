# Platform & Targets

> One core, many shells: **headless** (CI, agents) vs **headful** (rendered); **native desktop** (the AAA path) vs **web** (baseline); and **operator client builds** on top of the same prediction core. The renderer/UI/audio are plugins you add — never a dependency you can't remove. → [client README](../README.md) · [engine core](../../game-engine/core/README.md)

## Headless vs headful (from one core)
The same `App`/ECS/schedule runs with or without a window. Headless = `MinimalPlugins` (a `ScheduleRunnerPlugin` drives the loop, no winit, no `RenderPlugin`); headful adds render/UI/audio/input plugins on top. No logic forks on "are we rendering?" → [Bevy headless](https://taintedcoders.com/bevy/how-to/headless-mode), [`no_renderer` example](https://github.com/bevyengine/bevy/blob/main/examples/app/no_renderer.rs).
- **Headless is a first-class target**, not a degraded mode: it's what CI and **AI agents** run — connect, feed [intents](../input/README.md), assert [predicted state](../prediction-core/README.md) ([ai-native-dx](../../game-engine/ai-native-dx/README.md)). The shipped [`apps/client`](../../../../apps/client/src/main.rs) *is* this core.
- **Determinism holds across both** only if sim runs on a fixed timestep decoupled from frame rate — headless must reproduce headful ([engine core](../../game-engine/core/README.md), [prediction-core](../prediction-core/README.md)).

## Native vs web (feature tiers, not forks)
| Target | Backend | Renderer path | Transport |
|---|---|---|---|
| **Native desktop** (Win/macOS/Linux) | wgpu → Vulkan/Metal/DX12 | **AAA**: meshlets, GI, DLSS where present ([rendering](../../game-engine/rendering/README.md)) | UDP (`renet`) |
| **Web** (browser) | wgpu → WebGPU (WebGL2 fallback) | **Baseline** forward + spatial AA | WebTransport (QUIC) |

The web/WebGPU target **can't run the high-end path** — no bindless, mesh shaders, or ray tracing in browsers, and WebGPU/WebGL2 need separate wasm builds ([Bevy WebGPU](https://bevy.org/news/bevy-webgpu/), [wgpu limits](https://github.com/gfx-rs/wgpu)). So it's a **capability tier**, gated at boot, sharing one material set — not a code fork ([rendering](../../game-engine/rendering/README.md)). Browser can't do raw UDP, so game state rides **WebTransport** (unreliable datagrams + reliable streams over HTTP/3) ([networking](../networking/README.md)). Linux-first for now; other targets follow ([apps/client](../../../../apps/client/src/main.rs)).

## Operator client builds
Operators ship their **own client build** — extra maps, modified rules, reskins — on the *same prediction core*. Core stays authoritative and strong ([security](../../game-server/security/README.md)); the surface (content, UI, assets) is theirs, all open-format and moddable ([content-scripting](../../game-server/content-scripting/README.md), [modding](../../../architecture/05-ecs-and-scripting.md)). The core they can't fork is exactly the part that must stay trusted.

## Distilled from the references
| Source | Adopt |
|---|---|
| Bevy `MinimalPlugins` / no-renderer | Headless = same core minus render plugins; first-class CI/agent target |
| wgpu / WebGPU status | Native = AAA tier, web = baseline tier; separate wasm build, one material set |
| WebTransport | Browser low-latency transport (datagrams + reliable streams) |
| [apps/client](../../../../apps/client/src/main.rs) | Linux-first headless core shipped; build headful/native/web on top |

## Rules
- **One core, plugins for shells** — never branch logic on render/no-render ([engine core](../../game-engine/core/README.md)).
- **Headless is first-class** — deterministic, agent-drivable, CI-run ([ai-native-dx](../../game-engine/ai-native-dx/README.md)).
- **Web = baseline tier**, capability-gated — don't assume the AAA path exists ([rendering](../../game-engine/rendering/README.md)).
- **Operators build on the core, not into it** — the authoritative prediction core stays un-forkable ([security](../../game-server/security/README.md)).

## Links
[prediction-core](../prediction-core/README.md) · [rendering](../rendering/README.md) · [networking](../networking/README.md) · [engine core](../../game-engine/core/README.md) · [ai-native-dx](../../game-engine/ai-native-dx/README.md) · [apps/client](../../../../apps/client/src/main.rs)
