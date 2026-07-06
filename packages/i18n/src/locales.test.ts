import { describe, expect, test } from 'bun:test';
import { de } from './locales/de.ts';
import { en } from './locales/en.ts';
import { LOCALES, resolveLocale } from './locales.ts';
import { createTranslator } from './translator.ts';

describe('locales', () => {
  test('includes de locale', () => {
    expect(LOCALES).toContain('de');
  });

  test('resolves de locale', () => {
    expect(resolveLocale('de')).toBe('de');
  });

  test('English catalog has all expected keys', () => {
    const t = createTranslator(en);
    expect(t('common.loading')).toBe('Loading…');
    expect(t('nav.home')).toBe('Home');
    expect(t('auth.login.heading')).toBe('Log in');
    expect(t('armory.heading')).toBe('Armory');
  });

  test('German catalog returns German strings', () => {
    const t = createTranslator(de);
    expect(t('common.loading')).toBe('Laden…');
    expect(t('nav.home')).toBe('Startseite');
    expect(t('auth.login.heading')).toBe('Anmelden');
    expect(t('armory.heading')).toBe('Rüstkammer');
  });

  test('German catalog interpolates params correctly', () => {
    const t = createTranslator(de);
    expect(t('common.gold', { amount: 42 })).toBe('42 Gold');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('Willkommen zurück, Aria.');
  });

  test('German catalog has same key structure as English', () => {
    const tEn = createTranslator(en);
    const tDe = createTranslator(de);

    // Sample a few keys across different sections
    const sampleKeys = [
      'common.loading',
      'nav.home',
      'auth.login.heading',
      'armory.character.level',
      'auction.heading',
      'feed.heading',
    ];

    for (const key of sampleKeys) {
      const enResult = tEn(key);
      const deResult = tDe(key);
      // Both should return strings, not missing-key markers
      expect(enResult).not.toMatch(/^⟦.*⟧$/);
      expect(deResult).not.toMatch(/^⟦.*⟧$/);
    }
  });

  test('missing keys render loudly in German', () => {
    const t = createTranslator(de);
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });
});
