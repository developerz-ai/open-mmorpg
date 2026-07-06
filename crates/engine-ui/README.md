# omm-engine-ui

UI engine substrate for Open-MMORPG: a **pure, headless i18n substrate** (Fluent +
locale formatting) with **optional `bevy_ui` rendering** behind the `ui` feature.

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

## Rendering (optional, `ui` feature)

`ui` gates `bevy_ui` + `bevy_render` for the client-side retained HUD. Disabled by
default so the server and tests stay headless.

```toml
omm-engine-ui = { version = "0.0.0", features = ["ui"] }
```

## Modules

- **`catalog`** — pure `Catalog` + `t()` + `TransArgs` (Fluent, fallback, loud misses).
- **`format`** — `LocaleFormatter` + `Currency` (numbers/money/dates by locale rules).
- **`error`** — `I18nError` (authoring faults: malformed `.ftl`).
- **`i18n`** — Bevy glue: `I18nAsset`, `I18nBundle`, `I18nPlugin`.
- **`ui`** (feature `ui`) — `bevy_ui` rendering substrate, not compiled headless.

## Scope & honest gaps

Locale formatting covers a curated launch set (unknown locales fall back to `en`
rules); group separators use an ASCII space where CLDR prefers a narrow no-break
space. Both are deliberate simplifications with a clean path to full ICU4X data
behind the same `LocaleFormatter` API.

## Testing

```bash
cargo nextest run -p omm-engine-ui                 # headless i18n
cargo nextest run -p omm-engine-ui --features ui   # include rendering
```
