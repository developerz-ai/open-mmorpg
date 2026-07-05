# Editor & Tooling

> An Unreal-class authoring loop — outliner, inspector, content browser, viewport — but **reflection-generated and MCP-first**, so a human clicks and an **agent drives the exact same surface**. We don't out-build Epic's GUI team; we make the GUI a thin, generated view over introspectable data. → [engine README](../README.md)

## What it does
Gives content authors (human or agent) the loop Unreal defines: *place in the Viewport, select in the Outliner, configure in the Details/Inspector, manage assets in the Content Browser.* → [Unreal Editor interface](https://dev.epicgames.com/documentation/unreal-engine/unreal-editor-interface). Bevy has **no first-party editor yet** ([status](https://bevy-cheatbook.github.io/setup/bevy-tools.html)); the strongest third-party is [`space_editor`](https://github.com/rewin123/space_editor). We build ours **reflection-first** so it's cheap to generate and identical to the agent API.

## Design
- **Reflection-generated inspector.** The Details panel is auto-built from `bevy_reflect` — every registered component field renders an editor widget with zero per-type UI code, the same pattern Unreal gets from `UPROPERTY`. Add a field → it's editable everywhere. This is why the editor stays lean without a tools team ([scene](../scene/README.md), [ui](../ui/README.md)).
- **The editor is a view over the same registry the agent uses.** Outliner = entity/relationship tree; Inspector = reflected components; Content Browser = the [asset](../assets/README.md) manifest; Viewport = the live [renderer](../rendering/README.md). Every one reads the shared registry/manifest — so the [MCP surface](../ai-native-dx/README.md) and the GUI are the same capability, never forked.
- **Live, hot-reloading.** Edits apply to the running app via scene/asset hot reload ([assets](../assets/README.md)) — Godot-style remote-scene liveness, not a rebuild ([Godot editor reference](https://docs.godotengine.org/)).
- **egui-based, self-contained.** Built on immediate-mode [`bevy_egui`](../ui/README.md) — dense, fast, reflection-friendly — so the editor ships as a plugin, not a separate app.

## Why this beats a bigger GUI
| Incumbent | Ours | Payoff |
|---|---|---|
| Editor UI hand-built per type by a tools team | Inspector generated from reflection | Solo dev + agents keep pace with a studio's tooling |
| GUI is the primary (often only) authoring path | GUI and **MCP agent API are the same surface** ([ai-native-dx](../ai-native-dx/README.md)) | Agents author content at scale, headless, verifiably |
| Proprietary, monolithic | Plugin over open Bevy crates + open formats | Other games reuse it; operators extend it ([legal](../../../initial-idea/01-legal-and-licensing.md)) |

## Distilled from the references
| Source | Lesson | Our verdict |
|---|---|---|
| Unreal Editor (Outliner/Details/Content Browser/Viewport) | The authoring loop worth copying | **Adopt the loop**, generate the panels from reflection |
| Unreal `UPROPERTY` → auto UI | Reflection → editor UI is the force multiplier | **Adopt** via `bevy_reflect` |
| `space_editor` / Bevy editor prototypes | A Bevy editor is feasible today as a plugin | **Build on the pattern**, keep it MCP-parity |
| Godot editor | Lean, live remote-scene inspection during play | **Adopt** liveness via hot reload |

## Rules
- **Every editor action is reachable over MCP** — GUI and agent API never diverge ([ai-native-dx](../ai-native-dx/README.md)).
- **Panels are reflection-generated**, not hand-coded per type — no bespoke inspector UI ([scene](../scene/README.md), [ui](../ui/README.md)).
- Editor is a **plugin**, headful-only; it never links into the headless core ([platform](../../client/platform/README.md)).
- Authoring outputs are **open formats** — scenes as text, assets as glTF/KTX2 — diffable and reusable ([assets](../assets/README.md)).

## Links
[ai-native-dx](../ai-native-dx/README.md) · [scene](../scene/README.md) · [ui](../ui/README.md) · [assets](../assets/README.md) · [rendering](../rendering/README.md)
