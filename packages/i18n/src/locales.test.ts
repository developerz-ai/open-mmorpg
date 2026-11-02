import { describe, expect, test } from 'bun:test';
import { DEFAULT_LOCALE, LOCALES, resolveLocale } from './locales.ts';
import { createTranslator } from './translator.ts';

const supported = [
  'en',
  'de',
  'es',
  'fr',
  'ja',
  'zh',
  'ko',
  'ru',
  'pt',
  'it',
  'pl',
  'tr',
  'ar',
] as const;

describe('locale registry', () => {
  test('includes exactly all 13 supported locales', () => {
    expect([...LOCALES].sort()).toEqual([...supported].sort());
  });

  for (const locale of supported) {
    test(`resolves ${locale} without fallback`, () => {
      expect(resolveLocale(locale)).toBe(locale);
    });
  }

  test('unknown and empty tags fall back to English', () => {
    expect(DEFAULT_LOCALE).toBe('en');
    for (const tag of ['xx', '', 'de-DE', 'EN']) {
      expect(resolveLocale(tag)).toBe(DEFAULT_LOCALE);
    }
  });
});

describe('catalog fixtures (not shipped copy)', () => {
  const t = createTranslator({
    nested: { message: 'Grüße, {name} — {count} 世界 🌍' },
    arabic: 'مرحبًا، {name}',
  });

  test('translates nested Unicode messages with string and numeric params', () => {
    expect(t('nested.message', { name: 'Élia', count: 0 })).toBe('Grüße, Élia — 0 世界 🌍');
    expect(t('arabic', { name: 'نور' })).toBe('مرحبًا، نور');
  });

  test('missing keys and unresolved placeholders stay loud', () => {
    expect(t('nested.missing')).toBe('⟦nested.missing⟧');
    expect(t('nested.message', { name: 'Élia' })).toBe('Grüße, Élia — {count} 世界 🌍');
  });
});
