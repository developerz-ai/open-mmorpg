import { describe, expect, test } from 'bun:test';
import { validateAccentContrast } from './brand';

describe('Brand Utilities', () => {
  describe('validateAccentContrast', () => {
    test('passes with high contrast colors', () => {
      const warnings = validateAccentContrast('100 180 255', '140 200 255');
      expect(warnings.length).toBe(0);
    });

    test('warns on low contrast accent on background', () => {
      // Dark gray on black background (low contrast)
      const warnings = validateAccentContrast('30 30 30', '40 40 40');
      expect(warnings.some((w) => w.key === 'accent-on-bg')).toBe(true);
    });

    test('warns on low contrast accent-strong for text', () => {
      // Medium gray on dark background (below 4.5:1 for text)
      const warnings = validateAccentContrast('100 100 100', '120 120 120');
      expect(warnings.some((w) => w.key === 'accent-strong-on-bg')).toBe(true);
    });

    test('handles invalid RGB format', () => {
      const warnings = validateAccentContrast('invalid', '100 100 100');
      expect(warnings.some((w) => w.key === 'parse-error')).toBe(true);
    });

    test('includes contrast ratio in warnings', () => {
      const warnings = validateAccentContrast('30 30 30', '40 40 40');
      const accentWarning = warnings.find((w) => w.key === 'accent-on-bg');
      expect(accentWarning).toBeDefined();
      expect(accentWarning?.ratio).toBeGreaterThan(0);
      expect(accentWarning?.required).toBe(3.0);
    });
  });
});
