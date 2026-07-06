# Operator Setup Guide

> How to deploy, configure, and brand your Open-MMORPG operator portal.

The operator portal (`apps/web`) is a **single codebase, many branded deployments** system — you configure your realm at deploy via environment variables, not code edits. This mirrors the data-driven core philosophy: one compiled binary serves many operators.

## Quick Start

1. **Clone & install** → `bin/setup` (handles Bun deps, builds packages)
2. **Configure** → Copy `.env.development` to `.env.production.local` and set your values
3. **Build** → `bun run --filter @omm/web build`
4. **Deploy** → Serve `apps/web/dist/` with any static host (Nginx, Cloudflare, Netlify, etc.)

## Configuration via Environment Variables

All configuration happens through `VITE_*` environment variables at **build time**. These are bundled into the static artifact — no runtime config needed.

### Required Variables

```bash
# Realm identity
VITE_REALM_NAME="My Realm"                    # Appears in <title>, header, SEO

# Backend endpoints (where the gateway/worldsvc live)
VITE_GATEWAY_URL="https://api.myrealm.com"    # Auth, realm status
VITE_WORLDSVC_URL="https://world.myrealm.com" # Armory, auction house, feed
```

### Optional Variables

```bash
# Branding
VITE_REALM_TAGLINE="Adventure awaits"           # Hero subtitle
VITE_LOGO_URL="https://cdn.myrealm.com/logo.png" # Custom logo (replaces text)
VITE_FAVICON_URL="https://cdn.myrealm.com/favicon.ico" # Custom favicon

# Theming (dark theme only — accent retint within the palette)
VITE_BRAND_ACCENT="96 170 240"                # R G B channels (overrides --color-accent)
VITE_BRAND_ACCENT_STRONG="130 190 248"        # R G B channels (overrides --color-accent-strong)

# Feature flags (UX only — server re-enforces these)
VITE_REGISTRATION_OPEN=true                    # Hide/show registration form
VITE_CASH_SHOP=false                          # Enable/disable cash shop tab
VITE_ARMORY_PUBLIC=true                       # Show armory to unauth visitors
VITE_AUCTION_HOUSE=true                        # Enable auction house browser
VITE_WORLD_FEED=true                          # Enable world feed

# Development
VITE_USE_MOCKS=false                          # Use in-memory mock backend (dev only)
VITE_LOCALE=en                                # Default locale (en | de | es | fr | ja | zh)

# Downloads (client download metadata)
VITE_DOWNLOAD_VERSION="1.0.0"
VITE_DOWNLOAD_URL_WINDOWS_X64="https://cdn.myrealm.com/client-win-x64.zip"
VITE_DOWNLOAD_URL_WINDOWS_ARM64="https://cdn.myrealm.com/client-win-arm64.zip"
VITE_DOWNLOAD_URL_MACOS_X64="https://cdn.myrealm.com/client-macos-x64.zip"
VITE_DOWNLOAD_URL_MACOS_ARM64="https://cdn.myrealm.com/client-macos-arm64.zip"
VITE_DOWNLOAD_URL_LINUX_X64="https://cdn.myrealm.com/client-linux-x64.zip"
VITE_DOWNLOAD_CHECKSUM_WINDOWS_X64="sha256:..." # For client verification UI
VITE_DOWNLOAD_CHECKSUM_WINDOWS_ARM64="sha256:..."
VITE_DOWNLOAD_CHECKSUM_MACOS_X64="sha256:..."
VITE_DOWNLOAD_CHECKSUM_MACOS_ARM64="sha256:..."
VITE_DOWNLOAD_CHECKSUM_LINUX_X64="sha256:..."
```

## Deployment Patterns

### Static Hosting (Recommended)

Deploy the `dist/` directory to any static host:

```bash
# Build for production
bun run --filter @omm/web build

# Output: apps/web/dist/
# Upload this to your static host
```

**Examples:**
- **Nginx** → Point `root` to `dist/`
- **Cloudflare Pages** → Connect your Git repo, set build command to `bun run build`
- **Netlify** → Set build command to `bun run build`, publish directory `apps/web/dist`

### Docker (Optional)

For containerized deployments:

```dockerfile
FROM oven/bun:1 AS build
WORKDIR /app
COPY . .
RUN bun install && bun run build

FROM nginx:alpine
COPY --from=build /app/apps/web/dist /usr/share/nginx/html
```

## Branding & Theming

### Dark Theme Only

The portal ships a **single dark theme** — no light mode, no toggle. Operators re-brand by retinting the accent tokens, not by editing component code or shipping alternate palettes.

### Token-Based Theming

Colors are defined as **semantic tokens** (role, not value) in `:root`:

```css
:root {
  --color-bg: 18 18 20;           /* Darkest — page background */
  --color-bg-soft: 28 28 32;      /* Raised surfaces */
  --color-surface: 34 34 39;      /* Cards, modals */
  --color-fg: 228 226 222;        /* Primary text */
  --color-fg-strong: 248 247 245; /* Headings, emphasis */
  --color-fg-muted: 150 146 140;  /* Secondary text */
  --color-line: 54 54 60;         /* Borders, dividers */
  --color-accent: 96 170 240;     /* Links, buttons */
  --color-accent-strong: 130 190 248; /* Hover states */
  --color-success: 120 200 140;
  --color-danger: 232 120 120;
  --color-warning: 226 186 110;
}
```

Components reference tokens via utilities (`bg-bg text-fg`), never raw hex. You override **token values only** at deploy via `VITE_BRAND_ACCENT`:

```bash
# Retint to a gold theme
VITE_BRAND_ACCENT="226 186 110"     # Gold accent
VITE_BRAND_ACCENT_STRONG="248 210 140" # Brighter gold for hover
```

### Logo & Copy

Replace the default logo with your realm's branding:

```bash
VITE_REALM_NAME="Azeroth Realms"
VITE_REALM_TAGLINE="Your adventure begins"
VITE_LOGO_URL="https://cdn.azerothrealms.com/logo.png"
```

The portal applies these at boot (`applyBrand()` in `lib/brand.ts`) — components require no code changes.

## i18n (Internationalization)

### Supported Locales

The portal currently supports:
- `en` — English (default)
- `de` — German
- `es` — Spanish
- `fr` — French
- `ja` — Japanese
- `zh` — Chinese (Simplified)
- `ko` — Korean
- `ru` — Russian
- `pt` — Portuguese
- `it` — Italian
- `pl` — Polish
- `tr` — Turkish
- `ar` — Arabic

All 13 locales have complete catalogs and are fully implemented.

### Adding a New Locale (when infrastructure is complete)

To add a new language (e.g., `ko` for Korean) when the multi-locale support is complete:

1. **Create the catalog** → `packages/i18n/src/locales/ko.ts`
   ```typescript
   import type { Catalog } from '../catalog.ts';
   export const ko: Catalog = {
     common: { loading: '로딩 중…', ... },
     nav: { home: '홈', ... },
     // ... rest of the catalog, matching the en.ts structure
   };
   ```

2. **Register the locale** → Edit `packages/i18n/src/locales.ts`
   ```typescript
   export const LOCALES = ['en', 'ko'] as const;
   ```

3. **Update the schema** → Edit `apps/web/src/config.ts`
   ```typescript
   VITE_LOCALE: z.enum(['en', 'ko']).default('en'),
   ```

4. **Set as default** → `VITE_LOCALE=ko` (or use a locale-detecting route)

### String Authoring

Every user-facing string lives in the locale catalog (`packages/i18n/src/locales/en.ts`), not in component code. Components call `t('nav.home')` — missing keys render as `⟦nav.home⟧` (loud, never silent).

Dates, numbers, and money use `Intl` (built-in browser API), not i18n strings:

```typescript
new Intl.NumberFormat(locale, { style: 'currency', currency: 'GOLD' }).format(amount);
```

## Feature Flags

Control UX visibility via `VITE_*` flags. The **server re-enforces** these — a frontend toggle can't bypass server-side checks.

| Flag | Default | Purpose |
|------|---------|---------|
| `VITE_REGISTRATION_OPEN` | `true` | Show/hide registration form |
| `VITE_CASH_SHOP` | `false` | Enable/disable cash shop tab |
| `VITE_ARMORY_PUBLIC` | `true` | Show armory to unauth visitors |
| `VITE_AUCTION_HOUSE` | `true` | Enable auction house browser |
| `VITE_WORLD_FEED` | `true` | Enable world feed |

Example — close registration temporarily:

```bash
VITE_REGISTRATION_OPEN=false
```

## Mock Backend (Development)

For development without a live backend, set:

```bash
VITE_USE_MOCKS=true
```

This routes the API client to in-memory mocks (`apps/web/src/lib/mock-api/`) — the site is fully usable (auth, armory, auction house, feed) with no server running.

## Verification

### Pre-Deploy Checklist

- [ ] Set `VITE_REALM_NAME`, `VITE_GATEWAY_URL`, `VITE_WORLDSVC_URL`
- [ ] Test with `VITE_USE_MOCKS=false` (against a real backend)
- [ ] Verify branding (logo, accent color, tagline)
- [ ] Test feature flags (registration, cash shop, etc.)
- [ ] Run `bin/check` (lint, typecheck, tests pass)
- [ ] Build production bundle: `bun run --filter @omm/web build`
- [ ] Check `apps/web/dist/` size and contents

### Runtime Checks

At boot, the portal validates the env via **Zod** (`src/config.ts`). A misconfigured env fails loud (blank screen with console error) instead of silently half-branding.

Common errors:
- `VITE_GATEWAY_URL` missing or not a valid URL
- `VITE_BRAND_ACCENT` not in "R G B" format (e.g., `"96 170 240"`)
- `VITE_LOCALE` not in the supported list

## Monitoring & Debugging

### Browser Console

The portal logs configuration at boot:

```javascript
console.log('Realm:', config.brand.realmName);
console.log('Endpoints:', config.endpoints);
console.log('Features:', config.features);
```

Check these to verify your env is applied.

### Network Tab

Verify API calls go to the correct endpoints:
- `/auth/*` → `VITE_GATEWAY_URL`
- `/armory/*`, `/auction/*`, `/feed/*` → `VITE_WORLDSVC_URL`

### Lighthouse

Run Lighthouse on the deployed site to verify:
- Performance (target 90+)
- Accessibility (target 100 — semantic HTML, ARIA labels)
- SEO (meta tags, `<title>`, robots.txt)

## Security Notes

1. **No secrets in `VITE_*`** — These are bundled into the client at build time. Never put API keys, database passwords, or session tokens here.
2. **HTTPS only** — The gateway/worldsvc endpoints should be HTTPS in production.
3. **CORS** — Your backend must allow CORS from the portal's origin.
4. **Server authority** — The portal is read-only (except auth mutations). All game state lives on the server.

## Troubleshooting

### Build Fails

```bash
# Missing dependencies
bun install

# Type errors
bun run --filter @omm/web typecheck
```

### Blank Page on Load

Check the browser console for Zod validation errors (bad env format). Verify:
- `VITE_GATEWAY_URL` and `VITE_WORLDSVC_URL` are valid URLs
- `VITE_LOCALE` is in the supported list
- No trailing spaces in RGB channel values

### API Calls Fail

1. Verify backend is reachable: `curl https://your-gateway.com/health`
2. Check CORS headers on the backend response
3. Verify `VITE_USE_MOCKS=false` in production

### Strings Show as `⟦key⟧`

Missing i18n keys — check the locale catalog has the required key. Example: `⟦nav.home⟧` means `nav.home` is missing from the active locale.

## Brand Preview Mode

For operators testing custom themes, the portal includes a brand preview mode in development. When enabled, it displays a visual indicator and logs the current configuration to the browser console.

To enable brand preview:

```typescript
import { enableBrandPreview } from './lib/brand';
import { config } from './config';

// In your development entry point
if (import.meta.env.DEV) {
  enableBrandPreview(config);
}
```

This will:
- Add a floating indicator showing the realm name
- Log the current theme configuration to the console
- Validate contrast ratios and warn if they don't meet WCAG AA

## Example Configurations

### Default (Blue Accent)

```bash
VITE_REALM_NAME="Open-MMORPG"
VITE_BRAND_ACCENT="96 170 240"
VITE_BRAND_ACCENT_STRONG="130 190 248"
```

### Gold Theme

```bash
VITE_REALM_NAME="Azeroth Realms"
VITE_REALM_TAGLINE="Your legend awaits"
VITE_BRAND_ACCENT="226 186 110"
VITE_BRAND_ACCENT_STRONG="248 210 140"
```

### Green Theme

```bash
VITE_REALM_NAME="Nature's Realm"
VITE_REALM_TAGLINE="Explore the wild"
VITE_BRAND_ACCENT="120 200 140"
VITE_BRAND_ACCENT_STRONG="140 220 160"
```

### Purple Theme

```bash
VITE_REALM_NAME="Mystic Realms"
VITE_REALM_TAGLINE="Magic awaits"
VITE_BRAND_ACCENT="168 130 240"
VITE_BRAND_ACCENT_STRONG="190 160 255"
```

### Red/Danger Theme

```bash
VITE_REALM_NAME="Bloodlust Realms"
VITE_REALM_TAGLINE="Battle for glory"
VITE_BRAND_ACCENT="232 120 120"
VITE_BRAND_ACCENT_STRONG="255 140 140"
```

## Contrast Validation

When setting custom accent colors, ensure they meet WCAG AA contrast requirements:

- **Accent on background**: Minimum 3:1 contrast ratio (for large UI elements)
- **Accent-strong on background**: Minimum 4.5:1 contrast ratio (for normal text)
- **Focus indicators**: Minimum 3:1 contrast ratio against adjacent colors

The portal automatically validates contrast in development mode and logs warnings to the console if ratios are below thresholds.

To manually validate:

1. Build your config with custom accents
2. Load the site in development mode
3. Check the browser console for contrast warnings
4. Use the browser's contrast checker (DevTools) to verify specific elements

Example warning:
```
[Brand] Contrast warnings: [
  {
    key: 'accent-strong-on-bg',
    message: 'Accent strong has low contrast on background for text',
    ratio: 3.8,
    required: 4.5
  }
]
```

## Favicon and OG Meta Tags

The portal automatically sets favicon and Open Graph meta tags based on your configuration:

- **Favicon**: Set `VITE_FAVICON_URL` to your favicon path
- **OG Image**: Set `VITE_LOGO_URL` to use as the Open Graph image
- **OG Title**: Automatically set to `VITE_REALM_NAME`
- **OG Description**: Automatically set to `VITE_REALM_TAGLINE`

These tags improve how your realm appears when shared on social media platforms.

## Next Steps

- Full env var reference → `docs/operations/env-vars.md`
- Architecture → `docs/architecture/09-operator-web.md`
- i18n deep dive → `packages/i18n/README.md` (when written)
- Theming tokens → `packages/ui/src/theme.css`
