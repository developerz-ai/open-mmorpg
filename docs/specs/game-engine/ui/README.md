# UI

> Two UI modes, one engine: **retained** `bevy_ui` for persistent game UI (HUD, nameplates, inventory) and **immediate-mode** egui for tools/debug/editor. **i18n from day one** — every string keyed, missing keys render loudly. → [engine README](../README.md)

## What it does
Lays out and renders in-world and screen-space UI. `bevy_ui` is a **retained-mode**, ECS-native UI on the Taffy flexbox/grid engine — right for persistent hierarchies (HUD, minimap, nameplates, inventory panels). [`bevy_egui`](https://github.com/mvlabat/bevy_egui) provides **immediate-mode** UI for debug overlays, dev consoles, and the [editor](../editor/README.md) — fast to author, self-contained. Teams run both side by side. → [Bevy UI vs egui](https://github.com/bevyengine/bevy/discussions/21030).

## Design
- **Retained for game UI, immediate for tools.** HUD/nameplates/inventory = `bevy_ui` (persistent, styled, part of the world). Debug/editor/operator panels = egui (throwaway, dense, reflection-friendly). Don't force one paradigm on both.
- **Reflection-driven property UI.** The [editor](../editor/README.md) inspector and any tool panel are **generated from `bevy_reflect`**, not hand-built per type — add a component field, get an editor widget free ([scene](../scene/README.md), [ai-native-dx](../ai-native-dx/README.md)).
- **i18n is mandatory, not a feature.** Every user-facing string goes through `t(key)`. Missing keys render **loudly** (`⟦key⟧`), never silently blank — the same rule as [apps/web](../../../../CLAUDE.md). Plurals/gender/ordering via **ICU MessageFormat** (Fluent/`icu` crates); dates/money via `Intl`-equivalent locale formatting, **not** the string catalog. → [ICU MessageFormat](https://phrase.com/blog/posts/guide-to-the-icu-message-format/), [Project Fluent](https://github.com/projectfluent/fluent).
- **UI is a headful plugin.** The headless core has no UI; agents drive game state directly, not through widgets ([platform](../../client/platform/README.md)).

## Distilled from the references
| Source | Adopt |
|---|---|
| `bevy_ui` + Taffy | Retained flexbox/grid UI for persistent game HUD |
| `bevy_egui` | Immediate-mode UI for tools, debug, editor — fast, reflection-friendly |
| ICU MessageFormat / Fluent | Correct plurals/gender/ordering; locale formatting outside the catalog |
| apps/web i18n rule | Loud missing keys (`⟦key⟧`), `t()` everywhere — consistency across the product |

## Rules
- **Every string via `t()`.** Missing key → `⟦key⟧`, loud, never blank ([CLAUDE.md](../../../../CLAUDE.md)).
- Dates/money/numbers via locale formatting (`Intl`-style), never hardcoded or shoved in the i18n catalog.
- Retained for game UI, immediate for tools — pick per surface, don't monoculture.
- Inspector/tool panels are **reflection-generated**, not hand-authored per type ([editor](../editor/README.md)).

## Links
[editor](../editor/README.md) · [scene](../scene/README.md) · [ai-native-dx](../ai-native-dx/README.md) · [client HUD](../../client/hud-ui/README.md) · [operator-web](../../../architecture/09-operator-web.md)
