import type { OperatorConfig } from '../config.ts';

/**
 * Apply the operator's brand at boot. Palette overrides stay **within the dark
 * tokens** — we only retint the accent role, never ship a light theme or a raw
 * hex. Setting the CSS variables on `:root` is the single knob operators get;
 * component code never changes to re-brand. → docs/specs/web-client/operator-brand
 */
export function applyBrand(cfg: OperatorConfig): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  if (cfg.brand.accent) root.style.setProperty('--color-accent', cfg.brand.accent);
  if (cfg.brand.accentStrong) {
    root.style.setProperty('--color-accent-strong', cfg.brand.accentStrong);
  }
  document.title = cfg.brand.realmName;
}
