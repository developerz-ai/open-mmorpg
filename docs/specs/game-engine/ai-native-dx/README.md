# AI-Native Developer Experience

> **The differentiator.** Incumbent engines were built for humans clicking in an editor. This one is built so an **AI coding agent** — plus one human — out-ships a studio. Six properties, each deliberately engineered, not accidental. → [engine README](../README.md)

## The thesis, concretely
A studio's advantage is a huge tools team. We replace it with an engine an LLM can *read, drive, verify, and iterate* end to end. The gap between big studios and indies collapses when a solo dev's "team" is a fleet of agents and the engine is designed for them to succeed. Every property below is chosen because it raises **agent first-try correctness** or shortens the **agent iterate-observe loop**.

## Six engineered properties
| Property | What it buys an agent | How the engine delivers it |
|---|---|---|
| **Strong static types** | Bad code fails at `cargo check`, not at runtime in front of players | Rust + newtypes; make illegal states unrepresentable — the type system is the first reviewer ([CLAUDE.md](../../../../CLAUDE.md), [core](../core/README.md)) |
| **Reflection / introspection** | Agent can *enumerate* every component + field and edit data without reading all the code | `bevy_reflect` + a mandatory type registry; scenes, inspector, and MCP all read it ([scene](../scene/README.md)) |
| **Determinism** | Agent can *verify* a change by replaying inputs and asserting state | Fixed-timestep sim, stable ordering, no wall-clock/random on the sim path ([core](../core/README.md), [prediction-core](../../client/prediction-core/README.md)) |
| **Declarative, data-driven APIs** | Agent writes intent (spawn X with components Y), not boilerplate | ECS + required-components + scene/prefab data; content is data, not code ([scene](../scene/README.md)) |
| **Headless + testable** | Agent runs the *whole game* in CI, no GPU, and asserts on state | One core, render plugin removable; deterministic headless run ([platform](../../client/platform/README.md)) |
| **Hot reload** | Tight observe loop: change asset/scene → see result in ms | Asset file-watcher + reflected scene reload ([assets](../assets/README.md)) |

## MCP: agents drive the engine directly
The industry is converging on **MCP** as the agent↔engine interface — Unreal 5.8 ships a first-party MCP server (spawn actors, set lighting, make materials, run tests); Unity and Blender have community ones. → [Unreal 5.8 MCP](https://byteiota.com/unreal-engine-5-8-ships-mcp-server-ai-agents-can-now-drive-the-editor/), [Unity MCP](https://github.com/coplaydev/unity-mcp). **We go deeper**: our MCP surface ([`apps/mcp`](../../../../apps/mcp)) is not a bolt-on — it reads the same `bevy_reflect` registry the [editor](../editor/README.md) does, so anything the editor can do, an agent can do, headless, verifiably:
- **Introspect** — list entities/components/fields, read current values, read a scene's schema.
- **Author** — spawn/edit prefabs and scenes as reflected data; validated at load, fails loud.
- **Drive & verify** — run the headless app, feed [intents](../../game-server/netcode/README.md), assert predicted/authoritative state ([prediction-core](../../client/prediction-core/README.md)); connect an agent as a player ([mcp-companions](../../../architecture/07-mcp-companions.md)).
- **Iterate** — hot-reload the change, re-run, diff state. No human in the loop for the inner cycle.

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| Unreal 5.8 / Unity / Blender MCP | Agents driving an editor over MCP is now real and adopted | **Adopt + surpass**: MCP reads the reflection registry, works headless, and is verifiable — not a GUI puppeteer |
| Bevy reflection + headless ECS | Introspectable, testable-without-GPU engine is the substrate | **Adopt** as the whole foundation ([core](../core/README.md)) |
| Determinism (lockstep/replay) | Reproducible state is what lets an agent *check its own work* | **Adopt**: sim is bit-deterministic; verification is a first-class workflow |

## Rules
- **Reflection is not optional.** A gameplay type not in the registry is invisible to agents — that's a bug ([scene](../scene/README.md)).
- **The MCP surface = the editor surface.** Never build an editor capability an agent can't reach ([editor](../editor/README.md)).
- **Determinism is protected.** No wall-clock, no unseeded randomness, no order-dependent iteration on the sim path ([core](../core/README.md)).
- **Every change is agent-verifiable headless.** If you can't assert it in CI without a GPU, the design isn't finished ([platform](../../client/platform/README.md)).

## Links
[core](../core/README.md) · [scene](../scene/README.md) · [editor](../editor/README.md) · [platform](../../client/platform/README.md) · [mcp-companions](../../../architecture/07-mcp-companions.md) · [apps/mcp](../../../../apps/mcp)
