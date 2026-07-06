# omm-engine-ui

UI engine substrate for Open-MMORPG: **Fluent i18n + optional bevy_ui rendering**.

## Features

- **i18n first**: Fluent-based localization with ICU MessageFormat interpolation.
  - Asset loader for `.ftl` (Fluent Translation) files.
  - Runtime bundle management per locale.
  - Headless-compatible (no rendering dependency).

- **Optional rendering**: `ui` feature gates `bevy_ui` + `bevy_render` for client-side HUD/UI.
  - Disabled by default for server/tests (headless).
  - Enable with `omm-engine-ui = { version = "0.0.0", features = ["ui"] }`.

## Architecture

### Modules

- **`i18n`**: Fluent asset loader, bundle state, plugin registration.
  - `I18nAsset` — compiled Fluent bundle for a locale.
  - `I18nBundle` — game-wide state (current locale + handles).
  - `I18nLoader` — Bevy asset loader for `.ftl` files.
  - `I18nPlugin` — registers asset infrastructure.

- **`plugin`**: Core `UiPlugin` aggregator.
  - Always includes `I18nPlugin`.
  - Conditionally adds `bevy_ui::UiPlugin` when `ui` feature is enabled.

- **`ui`** (feature-gated, `ui`): Rendering substrate + helpers.
  - Not compiled for headless.
  - `UiElement` marker component.
  - Placeholder for future layout/styling tooling.

## Usage

### Headless (server, deterministic tests)

```rust,ignore
app.add_plugins(UiPlugin);
// i18n available, no rendering substrate
```

### Client (with rendering)

```toml
[dependencies]
omm-engine-ui = { version = "0.0.0", features = ["ui"] }
```

```rust,ignore
app.add_plugins(UiPlugin);  // bevy_ui now included
```

### Fluent Localization

1. Author `.ftl` files:
   ```fluent
   # en.ftl
   greeting = Hello, { $name }!
   ```

2. Load as asset:
   ```rust,ignore
   let en_handle = asset_server.load("locales/en.ftl");
   ```

3. Lookup at runtime via `I18nBundle`.

## Testing

```bash
cargo test -p omm-engine-ui
cargo test -p omm-engine-ui --features ui  # include rendering tests
```

## Dependencies

- **i18n**: `fluent`, `fluent-langneg`, `icu_locid`, `icu_provider_blob` (MIT).
- **ui (optional)**: `bevy_ui`, `bevy_render` (0.19).
