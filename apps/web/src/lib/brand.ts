import type { OperatorConfig } from '../config.ts';

/**
 * Parse RGB string to channels for contrast validation.
 * Format: "R G B" where each is 0-255.
 */
function parseRGB(rgb: string): [number, number, number] {
  const parts = rgb.split(/\s+/).map(Number);
  if (
    parts.length !== 3 ||
    parts.some((c) => c === undefined || c < 0 || c > 255 || Number.isNaN(c))
  ) {
    throw new Error(`Invalid RGB format: ${rgb}`);
  }
  return [parts[0] as number, parts[1] as number, parts[2] as number];
}

/**
 * Calculate relative luminance per WCAG 2.0 spec.
 */
function relativeLuminance(r: number, g: number, b: number): number {
  const rs = ((s: number) => (s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4))(r / 255);
  const gs = ((s: number) => (s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4))(g / 255);
  const bs = ((s: number) => (s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4))(b / 255);
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

/**
 * Calculate contrast ratio between two RGB colors.
 */
function contrastRatio(fg: [number, number, number], bg: [number, number, number]): number {
  const l1 = relativeLuminance(...fg);
  const l2 = relativeLuminance(...bg);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Validate that custom accent colors meet WCAG AA contrast requirements.
 * Returns warnings if contrast is below thresholds.
 */
export function validateAccentContrast(
  accent: string,
  accentStrong: string,
): Array<{ key: string; message: string; ratio: number; required: number }> {
  const warnings: Array<{ key: string; message: string; ratio: number; required: number }> = [];
  const bg = '18 18 20'; // Default background
  const fg = '228 226 222'; // Default foreground

  try {
    const accentRGB = parseRGB(accent);
    const accentStrongRGB = parseRGB(accentStrong);
    const bgRGB = parseRGB(bg);
    const fgRGB = parseRGB(fg);

    // Check accent on background (WCAG AA: 4.5:1 for normal text)
    const accentOnBg = contrastRatio(accentRGB, bgRGB);
    if (accentOnBg < 3.0) {
      warnings.push({
        key: 'accent-on-bg',
        message: 'Accent color has low contrast on background',
        ratio: accentOnBg,
        required: 3.0,
      });
    }

    // Check accent-strong on background (WCAG AA: 4.5:1 for normal text)
    const accentStrongOnBg = contrastRatio(accentStrongRGB, bgRGB);
    if (accentStrongOnBg < 4.5) {
      warnings.push({
        key: 'accent-strong-on-bg',
        message: 'Accent strong has low contrast on background for text',
        ratio: accentStrongOnBg,
        required: 4.5,
      });
    }

    // Check foreground on background (should always pass, but validate)
    const fgOnBg = contrastRatio(fgRGB, bgRGB);
    if (fgOnBg < 4.5) {
      warnings.push({
        key: 'fg-on-bg',
        message: 'Foreground has insufficient contrast on background',
        ratio: fgOnBg,
        required: 4.5,
      });
    }
  } catch (e) {
    warnings.push({
      key: 'parse-error',
      message: `Failed to parse colors: ${e}`,
      ratio: 0,
      required: 4.5,
    });
  }

  return warnings;
}

/**
 * Set favicon and OG meta tags based on operator config.
 */
function setMetaTags(cfg: OperatorConfig): void {
  if (typeof document === 'undefined') return;

  // Update favicon if URL provided
  if (cfg.brand.faviconUrl) {
    let link = document.querySelector("link[rel~='icon']") as HTMLLinkElement | null;
    if (!link) {
      link = document.createElement('link');
      link.rel = 'icon';
      document.head.appendChild(link);
    }
    link.href = cfg.brand.faviconUrl;
  }

  // Update OG meta tags
  const ogTitle = document.querySelector("meta[property='og:title']") as HTMLMetaElement | null;
  if (ogTitle) ogTitle.content = cfg.brand.realmName;

  const ogDesc = document.querySelector(
    "meta[property='og:description']",
  ) as HTMLMetaElement | null;
  if (ogDesc && cfg.brand.tagline) ogDesc.content = cfg.brand.tagline;

  // Update OG image if logo URL provided
  if (cfg.brand.logoUrl) {
    let ogImage = document.querySelector("meta[property='og:image']") as HTMLMetaElement | null;
    if (!ogImage) {
      ogImage = document.createElement('meta');
      ogImage.setAttribute('property', 'og:image');
      document.head.appendChild(ogImage);
    }
    ogImage.content = cfg.brand.logoUrl;
  }
}

/**
 * Apply the operator's brand at boot. Palette overrides stay **within the dark
 * tokens** — we only retint the accent role, never ship a light theme or a raw
 * hex. Setting the CSS variables on `:root` is the single knob operators get;
 * component code never changes to re-brand. → docs/specs/web-client/operator-brand
 */
export function applyBrand(cfg: OperatorConfig): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;

  // Apply accent overrides
  if (cfg.brand.accent) {
    root.style.setProperty('--color-accent', cfg.brand.accent);
    // Update focus-ring to match accent
    root.style.setProperty('--color-focus-ring', cfg.brand.accent);
  }
  if (cfg.brand.accentStrong) {
    root.style.setProperty('--color-accent-strong', cfg.brand.accentStrong);
  }

  // Set document title
  document.title = cfg.brand.realmName;

  // Set meta tags (favicon, OG)
  setMetaTags(cfg);

  // Validate contrast and log warnings in development
  if (cfg.brand.accent && cfg.brand.accentStrong && import.meta.env.DEV) {
    const warnings = validateAccentContrast(cfg.brand.accent, cfg.brand.accentStrong);
    if (warnings.length > 0) {
      console.warn('[Brand] Contrast warnings:', warnings);
    }
  }
}

/**
 * Enable brand preview mode for operators testing themes.
 * Adds a visual indicator and logs the current configuration.
 */
export function enableBrandPreview(cfg: OperatorConfig): void {
  if (typeof document === 'undefined') return;

  // Add a visual indicator in development
  if (import.meta.env.DEV) {
    const indicator = document.createElement('div');
    indicator.id = 'brand-preview-indicator';
    indicator.style.cssText = `
      position: fixed;
      bottom: 1rem;
      right: 1rem;
      padding: 0.5rem 0.75rem;
      background: rgb(var(--color-surface));
      border: 1px solid rgb(var(--color-line));
      border-radius: var(--radius-sm);
      font-size: 0.75rem;
      font-family: system-ui;
      color: rgb(var(--color-fg-muted));
      z-index: 9999;
      pointer-events: none;
      box-shadow: 0 2px 8px rgb(0 0 0 / 0.4);
    `;
    indicator.textContent = `🎨 ${cfg.brand.realmName}`;
    document.body.appendChild(indicator);

    console.log('[Brand Preview]', {
      realm: cfg.brand.realmName,
      accent: cfg.brand.accent || 'default',
      accentStrong: cfg.brand.accentStrong || 'default',
    });
  }
}
