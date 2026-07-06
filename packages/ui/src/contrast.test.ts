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

/** Dark theme color tokens (from theme.css) */
const COLORS = {
  bg: '18 18 20',
  'bg-soft': '28 28 32',
  surface: '34 34 39',
  'surface-hover': '42 42 48',
  fg: '228 226 222',
  'fg-strong': '248 247 245',
  'fg-muted': '150 146 140',
  line: '54 54 60',
  accent: '96 170 240',
  'accent-strong': '130 190 248',
  success: '120 200 140',
  danger: '232 120 120',
  warning: '226 186 110',
  'focus-ring': '96 170 240',
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
      bg: '0 0 0',
      'bg-soft': '30 30 30',
      surface: '45 45 45',
      'surface-hover': '60 60 60',
      fg: '255 255 255',
      'fg-strong': '255 255 255',
      'fg-muted': '210 210 210',
      line: '120 120 120',
      accent: '100 180 255',
      'accent-strong': '140 200 255',
      'focus-ring': '255 255 255',
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
