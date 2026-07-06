# omm-engine-ui

UI engine substrate for Open-MMORPG: a **pure, headless i18n substrate** (Fluent +
locale formatting) and a **reflection→widget-descriptor** generator, with
**optional `bevy_ui` rendering + retained HUD widgets** behind the `ui` feature.

## i18n — the real deliverable

Every user-facing string goes through `t(key, args)`. **Missing keys render loudly
as `⟦key⟧`** — never a silent blank — the same rule as [`apps/web`](../../apps/web)
(→ [`CLAUDE.md`](../../CLAUDE.md), [UI spec](../../docs/specs/game-engine/ui/README.md)).

- **`Catalog`** — compiled Fluent bundles per locale + a fallback chain
  (`fr-CA` → `fr` → default). Pure, `Send + Sync`, no I/O; build it from FTL sources.
- **`t(locale, key, args)`** — resolves through the fallback chain; ICU **plurals**
  (`$n -> [one] … *[other] …`) and **gender/select** (`$g -> [male] … *[other] …`)
  come from Fluent, with correct CLDR plural categories per language (e.g. Polish
  `one`/`few`/`many`). Missing → `⟦key⟧`.
- **`TransArgs`** — chainable args; numbers drive plurals, strings drive gender.
- **`LocaleFormatter`** — dates, money, numbers, and percent formatted **from locale
  rules, not the string catalog** (the `Intl`-equivalent surface). Money is carried
  as integer minor units (cents) to avoid float drift.
- **`I18nAsset` / `I18nBundle` / `I18nPlugin`** — the Bevy glue: `.ftl` files as
  hot-reloadable assets, a runtime resource (active locale + catalog + formatter),
  and headless plugin registration.

### Example

```rust,ignore
use omm_engine_ui::{Currency, I18nBundle, LocaleFormatter, TransArgs};

let mut ui = I18nBundle::from_sources("en", [("en", EN_FTL), ("de", DE_FTL)])?;

// Keyed strings, with plurals:
ui.t("enemies-left", &TransArgs::new().set("n", 1)); // "1 enemy remaining"
ui.t("nope", &TransArgs::new());                     // "⟦nope⟧" — loud, never blank

// Numbers / money / dates: locale formatting, NOT catalog strings:
let fmt = ui.formatter();
fmt.format_currency(1_999, Currency::USD); // "$19.99"
LocaleFormatter::new("de").format_date(2026, 7, 6); // "06.07.2026"
```

### FTL authoring

```fluent
# en.ftl
hud-gold = Gold
enemies-left = { $n ->
    [one] { $n } enemy remaining
   *[other] { $n } enemies remaining
}
login =
    .submit = Sign in    # addressed as key "login.submit"
```

## Reflection → widget descriptors (headless — the E9→E10 bridge)

`inspector` turns any registered `bevy_reflect` type into a UI-agnostic
[`WidgetDescriptor`] tree: every field becomes a widget the [editor](../../docs/specs/game-engine/editor/README.md)
Details panel and the [MCP](../../docs/specs/game-engine/ai-native-dx/README.md)
surface render with **zero per-type UI code** (the `UPROPERTY`→auto-UI pattern).
Pure and headless; the output is `serde`-serializable, so it crosses the MCP
boundary as JSON.

- **`WidgetKind`** — `checkbox` (bool), `integer`, `float`, `text`, `struct`,
  `enum`, `list`, `map`, `opaque` (read-only). Serialized `snake_case`.
- **`describe_type` / `describe_registration` / `describe_by_path`** — one type →
  one descriptor tree; enums expand to one group per variant, collections to an
  element/key/value *template*. Recursion is bounded ([`MAX_DEPTH`]); a
  self-referential type truncates loudly (`truncated = true`) instead of
  overflowing.
- **`inspect_components`** — every registered `#[reflect(Component)]` type as a
  descriptor, **sorted by type path** for deterministic agent output.

```rust,ignore
use omm_engine_ui::inspect_components;
let registry = app.world().resource::<AppTypeRegistry>().read();
for widget in inspect_components(&registry) {
    // widget.type_path, widget.kind, widget.children — feed the editor/MCP.
}
```

## Rendering + retained HUD (optional, `ui` feature)

`ui` gates `bevy_ui` + `bevy_render` + `bevy_color` for the client-side retained
HUD. Disabled by default so the server and tests stay headless.

```toml
omm-engine-ui = { version = "0.0.0", features = ["ui"] }
```

**`hud`** ships **retained** (persistent, ECS-native) game widgets — spawn once,
then a per-frame system binds visual state to gameplay data:

- **`HealthBar { current, max }`** — a node whose width tracks a clamped fill
  `fraction()` in `[0,1]` (robust against zero/negative/NaN). `spawn_health_bar`
  spawns it; `sync_health_bars` keeps the width current.
- **`Nameplate { name, subtitle_key }`** — a verbatim display name plus an
  optional **translated** subtitle (via `t()`, loud on a missing key).
  `spawn_nameplate` spawns it; `sync_nameplates` resolves the text.
- **`HudPlugin`** — registers both types for reflection (so the inspector above
  enumerates them) and wires the sync systems. Added by `UiPlugin` under `ui`.

## Modules

- **`catalog`** — pure `Catalog` + `t()` + `TransArgs` (Fluent, fallback, loud misses).
- **`format`** — `LocaleFormatter` + `Currency` (numbers/money/dates by locale rules).
- **`error`** — `I18nError` (authoring faults: malformed `.ftl`).
- **`i18n`** — Bevy glue: `I18nAsset`, `I18nBundle`, `I18nPlugin`.
- **`inspector`** — reflection → `WidgetDescriptor` / `WidgetKind` (headless; feeds editor/MCP).
- **`hud`** (feature `ui`) — retained HUD widgets: `HealthBar`, `Nameplate`, `HudPlugin`.
- **`ui`** (feature `ui`) — `bevy_ui` rendering substrate, not compiled headless.

## Scope & honest gaps

Locale formatting covers a curated launch set (unknown locales fall back to `en`
rules); group separators use an ASCII space where CLDR prefers a narrow no-break
space. Both are deliberate simplifications with a clean path to full ICU4X data
behind the same `LocaleFormatter` API.

## Testing

```bash
cargo nextest run -p omm-engine-ui                 # headless: i18n + inspector
cargo nextest run -p omm-engine-ui --features ui   # + rendering + retained HUD
```
