# Environment Variables Reference

> Complete reference for all `VITE_*` environment variables in the operator portal.

These variables are configured at **build time** and bundled into the static artifact. Runtime configuration is not supported — rebuild and redeploy to change values.

## Table of Contents

- [Realm Identity](#realm-identity)
- [Backend Endpoints](#backend-endpoints)
- [Branding & Theming](#branding--theming)
- [Feature Flags](#feature-flags)
- [Development](#development)
- [Downloads](#downloads)

---

## Realm Identity

### `VITE_REALM_NAME`

**Required** — The name of your realm/operation. Appears in:
- Page `<title>`
- Header/brand area
- SEO metadata

**Type:** `string` (min length: 1)
**Default:** `"Open-MMORPG"`

```bash
VITE_REALM_NAME="Azeroth Realms"
```

---

### `VITE_REALM_TAGLINE`

**Optional** — Tagline/slogan for the realm. Appears in the hero section.

**Type:** `string`
**Default:** (none)

```bash
VITE_REALM_TAGLINE="Your adventure begins"
```

---

### `VITE_LOGO_URL`

**Optional** — URL to a custom logo image. Replaces the text-based realm name in the header.

**Type:** `string` (URL)
**Default:** (none)

```bash
VITE_LOGO_URL="https://cdn.azerothrealms.com/logo.png"
```

**Constraints:**
- Should be a PNG, SVG, or WebP
- Recommended height: 1.75rem (28px) for header fit
- HTTPS only in production

---

## Backend Endpoints

### `VITE_GATEWAY_URL`

**Required** — Base URL for the gateway service (auth, realm status, account management).

**Type:** `string` (URL)
**Default:** `"http://localhost:8080"`

```bash
VITE_GATEWAY_URL="https://api.myrealm.com"
```

**Used by:**
- Authentication (login/register/logout)
- Realm status API
- Account management

---

### `VITE_WORLDSVC_URL`

**Required** — Base URL for the world service (armory, auction house, world feed).

**Type:** `string` (URL)
**Default:** `"http://localhost:8081"`

```bash
VITE_WORLDSVC_URL="https://world.myrealm.com"
```

**Used by:**
- Armory (character lookup)
- Auction house (browse listings, price history)
- World feed (recent events)

---

## Branding & Theming

### `VITE_BRAND_ACCENT`

**Optional** — Override the accent color (primary links, buttons, highlights). Value is **space-separated RGB channels** (0-255).

**Type:** `string` (format: `"R G B"`)
**Default:** (uses `--color-accent: 96 170 240` — blue)

```bash
# Gold theme
VITE_BRAND_ACCENT="226 186 110"

# Purple theme
VITE_BRAND_ACCENT="170 140 240"

# Green theme
VITE_BRAND_ACCENT="140 200 160"
```

**How it works:**
- Overrides the `--color-accent` CSS token on `:root`
- Components reference tokens, not raw hex — no code changes needed
- Applied via `applyBrand()` in `lib/brand.ts` at boot

**Constraints:**
- Must be three space-separated integers (0-255)
- No commas, no parentheses, no `rgb()` prefix

---

### `VITE_BRAND_ACCENT_STRONG`

**Optional** — Override the strong accent color (hover states, active elements). Value is **space-separated RGB channels**.

**Type:** `string` (format: `"R G B"`)
**Default:** (uses `--color-accent-strong: 130 190 248` — lighter blue)

```bash
# Gold theme (brighter for hover)
VITE_BRAND_ACCENT_STRONG="248 210 140"

# Purple theme
VITE_BRAND_ACCENT_STRONG="200 170 255"
```

**Note:** Should be a **lighter/brighter** version of `VITE_BRAND_ACCENT` for hover states.

---

## Feature Flags

Control UI visibility. The **server re-enforces these** — frontend toggles can't bypass server-side checks.

### `VITE_REGISTRATION_OPEN`

Show/hide the registration form. When `false`, only login is available.

**Type:** `boolean` (`"true"` | `"false"`)
**Default:** `"true"`

```bash
# Temporarily close registration
VITE_REGISTRATION_OPEN=false
```

---

### `VITE_CASH_SHOP`

Enable/disable the cash shop tab and routes.

**Type:** `boolean` (`"true"` | `"false"`)
**Default:** `"false"`

```bash
# Enable cash shop
VITE_CASH_SHOP=true
```

---

### `VITE_ARMORY_PUBLIC`

Show/hide the armory to unauthenticated visitors. When `false`, armory requires login.

**Type:** `boolean` (`"true"` | `"false"`)
**Default:** `"true"`

```bash
# Make armory login-only
VITE_ARMORY_PUBLIC=false
```

---

### `VITE_AUCTION_HOUSE`

Enable/disable the auction house browser.

**Type:** `boolean` (`"true"` | `"false"`)
**Default:** `"true"`

```bash
# Disable auction house (e.g., for realm launch)
VITE_AUCTION_HOUSE=false
```

---

### `VITE_WORLD_FEED`

Enable/disable the world feed (recent in-game events).

**Type:** `boolean` (`"true"` | `"false"`)
**Default:** `"true"`

```bash
# Disable world feed temporarily
VITE_WORLD_FEED=false
```

---

## Development

### `VITE_USE_MOCKS`

Route the API client to in-memory mocks instead of real backends. For development only — allows full site usage without running servers.

**Type:** `boolean` (`"true"` | `"false"`)
**Default:** `"false"`

```bash
# Development with mocks
VITE_USE_MOCKS=true
```

**Warning:** Never set to `true` in production.

---

### `VITE_LOCALE`

Default locale for the site. Determines the initial language catalog.

**Type:** `enum` (`"en"`)
**Default:** `"en"`

```bash
# Set locale (currently only English is supported)
VITE_LOCALE=en
```

**Supported locales:**
- `en` — English (only supported locale currently)

**Note:** Additional locales (de/es/fr/ja/zh) are planned. Set `VITE_LOCALE` to any other value will cause validation to fail at startup.

---

## Downloads

Client download metadata for the downloads page. All are **optional** — omit to hide the download section.

### `VITE_DOWNLOAD_VERSION`

Version string for the client download.

**Type:** `string`
**Default:** (none)

```bash
VITE_DOWNLOAD_VERSION="1.0.0"
```

---

### Platform Download URLs

**Type:** `string` (URL)
**Default:** (none)

```bash
# Windows
VITE_DOWNLOAD_URL_WINDOWS_X64="https://cdn.myrealm.com/client-win-x64.zip"
VITE_DOWNLOAD_URL_WINDOWS_ARM64="https://cdn.myrealm.com/client-win-arm64.zip"

# macOS
VITE_DOWNLOAD_URL_MACOS_X64="https://cdn.myrealm.com/client-macos-x64.zip"
VITE_DOWNLOAD_URL_MACOS_ARM64="https://cdn.myrealm.com/client-macos-arm64.zip"

# Linux
VITE_DOWNLOAD_URL_LINUX_X64="https://cdn.myrealm.com/client-linux-x64.zip"
```

**Constraints:**
- HTTPS only in production
- Should point to a `.zip` archive or installer

---

### Platform Checksums

**Optional** — SHA256 checksums for client verification. Displayed in the UI for users to verify download integrity.

**Type:** `string` (format: `"sha256:..."`)
**Default:** (none)

```bash
# Windows checksums
VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64="sha256:a1b2c3d4e5f6..."
VITE_DOWNLOAD_CHECKSUM_WINDOWS_ARM64="sha256:f6e5d4c3b2a1..."

# macOS checksums
VITE_DOWNLOAD_CHECKSUM_MACOS_X64="sha256:1a2b3c4d5e6f..."
VITE_DOWNLOAD_CHECKSUM_MACOS_ARM64="sha256:6f5e4d3c2b1a..."

# Linux checksums
VITE_DOWNLOAD_CHECKSUM_LINUX_X64="sha256:9a8b7c6d5e4f..."
```

**How to generate:**
```bash
sha256sum client-win-x64.zip
# Output: a1b2c3d4e5f6...  client-win-x64.zip
# Use: VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64="sha256:a1b2c3d4e5f6..."
```

---

## Validation

At boot/load time, all env vars are validated via **Zod** (`apps/web/src/config.ts`):

```typescript
const EnvSchema = z.object({
  VITE_REALM_NAME: z.string().min(1).default('Open-MMORPG'),
  VITE_GATEWAY_URL: z.string().url().default('http://localhost:8080'),
  VITE_BRAND_ACCENT: rgbChannels, // Regex: /^\d{1,3} \d{1,3} \d{1,3}$/
  // ...
});

export const config: OperatorConfig = parseConfig(import.meta.env);
```

A misconfigured env **fails loud** — the app loads blank with a clear console error at startup, not at build time.

---

## Examples

### Minimal Development Config

```bash
# .env.development
VITE_REALM_NAME="Dev Realm"
VITE_GATEWAY_URL=http://localhost:8080
VITE_WORLDSVC_URL=http://localhost:8081
VITE_USE_MOCKS=true
```

### Production Config

```bash
# .env.production.local
VITE_REALM_NAME="Azeroth Realms"
VITE_REALM_TAGLINE="Your adventure awaits"
VITE_LOGO_URL="https://cdn.azerothrealms.com/logo.png"
VITE_GATEWAY_URL=https://api.azerothrealms.com
VITE_WORLDSVC_URL=https://world.azerothrealms.com
VITE_BRAND_ACCENT="226 186 110"
VITE_BRAND_ACCENT_STRONG="248 210 140"
VITE_LOCALE=en
VITE_REGISTRATION_OPEN=true
VITE_CASH_SHOP=true
VITE_ARMORY_PUBLIC=true
VITE_AUCTION_HOUSE=true
VITE_WORLD_FEED=true
VITE_DOWNLOAD_VERSION="1.0.0"
VITE_DOWNLOAD_URL_WINDOWS_X64="https://cdn.azerothrealms.com/client-win-x64.zip"
VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64="sha256:..."
```

### German-Only Realm

```bash
VITE_REALM_NAME="Azeroth Deutsch"
VITE_LOCALE=de
VITE_GATEWAY_URL=https://api.azeroth-de.com
VITE_WORLDSVC_URL=https://world.azeroth-de.com
```

---

## Next Steps

- Setup guide → `docs/operations/operator-setup.md`
- Architecture → `docs/architecture/09-operator-web.md`
- Configuration code → `apps/web/src/config.ts`
