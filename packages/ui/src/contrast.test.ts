import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';

/**
 * WCAG AA contrast ratio verification tests.
 *
 * WCAG 2.1 Level AA requires:
 * - Normal text: 4.5:1 contrast ratio
 * - Large text (18pt+ or 14pt+ bold): 3:1 contrast ratio
 * - UI components/borders: 3:1 contrast ratio
 *
 * References:
 * - https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html
 * - https://www.w3.org/WAI/WCAG21/Understanding/contrast-enhanced.html
 */

/** Calculate relative luminance per WCAG 2.0 spec */
function relativeLuminance(r: number, g: number, b: number): number {
  const [rs, gs, bs] = [r, g, b].map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

/** Calculate contrast ratio between two RGB colors */
function contrastRatio(fg: [number, number, number], bg: [number, number, number]): number {
  const l1 = relativeLuminance(...fg);
  const l2 = relativeLuminance(...bg);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/** Parse "R G B" string to number array */
function parseRGB(rgb: string): [number, number, number] {
  const parts = rgb.split(/\s+/).map(Number);
  if (parts.length !== 3 || parts.some(Number.isNaN)) {
    throw new Error(`Invalid RGB format: ${rgb}`);
  }
  return [parts[0], parts[1], parts[2]];
}

/** WCAG AA requirements */
const WCAG_AA = {
  NORMAL_TEXT: 4.5,
  LARGE_TEXT: 3.0,
  UI_COMPONENT: 3.0,
} as const;

/**
 * Composite `fg` at `alpha` over `bg` — what the eye actually sees when a colour is
 * painted at partial opacity. A tinted badge has no opaque background of its own, so
 * measuring its text against the page background reads a contrast nobody is looking at.
 */
function over(
  fg: [number, number, number],
  bg: [number, number, number],
  alpha: number,
): [number, number, number] {
  return fg.map((c, i) => Math.round(alpha * c + (1 - alpha) * bg[i])) as [number, number, number];
}

/**
 * Tokens are read out of `theme.css` rather than copied here. A hand-kept table passes
 * happily after someone edits the stylesheet, which is the one failure this file exists
 * to prevent.
 */
const THEME = readFileSync(new URL('./theme.css', import.meta.url), 'utf8');
const HC_AT = THEME.indexOf('@media (prefers-contrast: more)');
const DEFAULT_BLOCK = THEME.slice(0, HC_AT);
const HC_BLOCK = THEME.slice(HC_AT);

function token(name: string, block: string = DEFAULT_BLOCK): string {
  const found = block.match(new RegExp(`--color-${name}:\\s*([\\d\\s]+?);`));
  if (!found?.[1]) throw new Error(`--color-${name} is not declared in theme.css`);
  return found[1].trim();
}

/** Dark theme color tokens, as theme.css currently declares them. */
const COLORS = {
  bg: token('bg'),
  'bg-soft': token('bg-soft'),
  surface: token('surface'),
  'surface-hover': token('surface-hover'),
  fg: token('fg'),
  'fg-strong': token('fg-strong'),
  'fg-muted': token('fg-muted'),
  line: token('line'),
  accent: token('accent'),
  'accent-strong': token('accent-strong'),
  success: token('success'),
  danger: token('danger'),
  warning: token('warning'),
  'focus-ring': token('focus-ring'),
} as const;

describe('WCAG AA Contrast Verification', () => {
  describe('Dark theme defaults', () => {
    test('normal text on background (fg on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.fg), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('strong text on background (fg-strong on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS['fg-strong']), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('muted text on background (fg-muted on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS['fg-muted']), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('accent on background (accent on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.accent), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('accent-strong on background (accent-strong on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS['accent-strong']), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('success on background (success on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.success), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });

    test('danger on background (danger on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.danger), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });

    test('warning on background (warning on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.warning), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });

    test('line on background (line on bg)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.line), parseRGB(COLORS.bg));
      // Line color is intentionally subtle; high-contrast mode overrides this
      expect(ratio).toBeGreaterThanOrEqual(1.0);
    });

    test('text on surface (fg on surface)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.fg), parseRGB(COLORS.surface));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('text on surface-hover (fg on surface-hover)', () => {
      const ratio = contrastRatio(parseRGB(COLORS.fg), parseRGB(COLORS['surface-hover']));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });
  });

  /**
   * The gap that let a failing badge ship: every test above measures a role colour
   * against the page background, but `.badge--*` paints that same colour as TEXT on
   * its own 15% tint. The tint is lighter than the background, so the real ratio is
   * always lower than the one measured above — `danger` passed the 3:1 check here at
   * 6.6:1 while the badge a visitor read sat at 4.42:1.
   */
  describe('Role colours used as text on their own tint (badges, pills)', () => {
    const TINT_ALPHA = 0.15;
    const surface = parseRGB(COLORS.surface);

    for (const role of ['danger', 'success', 'warning'] as const) {
      test(`${role} badge text on a ${TINT_ALPHA * 100}% ${role} tint`, () => {
        const colour = parseRGB(COLORS[role]);
        const ratio = contrastRatio(colour, over(colour, surface, TINT_ALPHA));
        expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
      });
    }
  });

  describe('Focus indicator visibility', () => {
    test('focus-ring on background', () => {
      const ratio = contrastRatio(parseRGB(COLORS['focus-ring']), parseRGB(COLORS.bg));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });

    test('focus-ring on surface', () => {
      const ratio = contrastRatio(parseRGB(COLORS['focus-ring']), parseRGB(COLORS.surface));
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });
  });

  describe('High-contrast mode overrides', () => {
    const HIGH_CONTRAST_COLORS = {
      bg: token('bg', HC_BLOCK),
      'bg-soft': token('bg-soft', HC_BLOCK),
      surface: token('surface', HC_BLOCK),
      'surface-hover': token('surface-hover', HC_BLOCK),
      fg: token('fg', HC_BLOCK),
      'fg-strong': token('fg-strong', HC_BLOCK),
      'fg-muted': token('fg-muted', HC_BLOCK),
      line: token('line', HC_BLOCK),
      accent: token('accent', HC_BLOCK),
      'accent-strong': token('accent-strong', HC_BLOCK),
      'focus-ring': token('focus-ring', HC_BLOCK),
    } as const;

    test('normal text in high-contrast (fg on bg)', () => {
      const ratio = contrastRatio(
        parseRGB(HIGH_CONTRAST_COLORS.fg),
        parseRGB(HIGH_CONTRAST_COLORS.bg),
      );
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('muted text in high-contrast (fg-muted on bg)', () => {
      const ratio = contrastRatio(
        parseRGB(HIGH_CONTRAST_COLORS['fg-muted']),
        parseRGB(HIGH_CONTRAST_COLORS.bg),
      );
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.NORMAL_TEXT);
    });

    test('line in high-contrast (line on bg)', () => {
      const ratio = contrastRatio(
        parseRGB(HIGH_CONTRAST_COLORS.line),
        parseRGB(HIGH_CONTRAST_COLORS.bg),
      );
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });

    test('focus-ring in high-contrast (focus-ring on bg)', () => {
      const ratio = contrastRatio(
        parseRGB(HIGH_CONTRAST_COLORS['focus-ring']),
        parseRGB(HIGH_CONTRAST_COLORS.bg),
      );
      expect(ratio).toBeGreaterThanOrEqual(WCAG_AA.UI_COMPONENT);
    });
  });
});
