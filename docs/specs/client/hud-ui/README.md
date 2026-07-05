# HUD & Game UI

> The player-facing UI — HUD, nameplates, minimap, inventory, chat — on the [engine UI](../../game-engine/ui/README.md) substrate, **i18n from day one**. Retained-mode for persistent game UI; every string keyed and loud on miss. → [engine UI](../../game-engine/ui/README.md)

## What it does
Draws the persistent game interface with retained-mode [`bevy_ui`](../../game-engine/ui/README.md): health/resource bars, action bars, floating **nameplates** over entities, **minimap** from the [world model](../../game-server/world-model/README.md), inventory/character panels, and chat. All strings pass through `t(key)`. Debug/dev overlays use the engine's immediate-mode egui path — separate from shipped game UI.

## Design
- **Nameplates from the render set.** Nameplates/health bars attach to the AoI-bounded set of visible entities ([rendering](../rendering/README.md)) — the UI never references an entity the client can't see ([world-model](../../game-server/world-model/README.md)).
- **Minimap = the same quadtree.** The minimap reads the client's streamed world partition, not a separate map asset — one spatial model ([world-model](../../game-server/world-model/README.md)).
- **i18n is mandatory.** Every user-facing string via `t(key)`; missing keys render **`⟦key⟧`**, loud, never blank — the exact rule [apps/web](../../../../CLAUDE.md) and the [engine UI](../../game-engine/ui/README.md) hold. Plurals/gender via **ICU MessageFormat**; dates/money/numbers via locale formatting, **not** the string catalog. → [ICU MessageFormat](https://phrase.com/blog/posts/guide-to-the-icu-message-format/), [Project Fluent](https://github.com/projectfluent/fluent).
- **UI reads state, never asserts it.** Inventory/health panels display predicted/authoritative state; actions taken in the UI emit [`Intent`](../../../../crates/protocol/src/lib.rs)s through the [input](../input/README.md) path — the server validates, the panel reflects the result ([security](../../game-server/security/README.md)).
- **Headful-only.** The HUD is a plugin on the render build; the headless core has no UI — agents read game state directly ([platform](../platform/README.md)).

## Distilled from the references
| Source | Adopt |
|---|---|
| [Engine UI](../../game-engine/ui/README.md) | Retained `bevy_ui` for game UI, egui for tools; i18n substrate |
| apps/web i18n rule | `t()` everywhere, loud `⟦key⟧` on miss, `Intl`-style formatting outside the catalog |
| ICU / Fluent | Correct plurals/gender/ordering across locales |
| [World-model](../../game-server/world-model/README.md) | Nameplates + minimap read the AoI/quadtree, not a second data source |

## Rules
- **Every string via `t()`**; missing → `⟦key⟧`, loud, never blank ([CLAUDE.md](../../../../CLAUDE.md)).
- Dates/money/numbers via locale formatting, never hardcoded or in the i18n catalog.
- UI **displays** predicted/authoritative state and **emits intents** — it never asserts state ([security](../../game-server/security/README.md)).
- HUD is a **headful-only plugin**; headless reads state directly ([platform](../platform/README.md)).

## Links
[rendering](../rendering/README.md) · [input](../input/README.md) · [networking](../networking/README.md) · [platform](../platform/README.md) · [engine UI](../../game-engine/ui/README.md) · [operator-web](../../../architecture/09-operator-web.md)
