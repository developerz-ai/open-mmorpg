import { describe, expect, test } from 'bun:test';
import { flatten } from './catalog.ts';
import { interpolate } from './interpolate.ts';
import { resolveLocale } from './locales.ts';
import { createTranslator } from './translator.ts';

describe('i18n', () => {
  const t = createTranslator({
    realm: { status: { online: 'Online', population: '{count} online' } },
    greeting: 'Welcome, {name}',
  });

  test('resolves nested keys via dot lookup', () => {
    expect(t('realm.status.online')).toBe('Online');
  });

  test('interpolates params', () => {
    expect(t('greeting', { name: 'Aria' })).toBe('Welcome, Aria');
    expect(t('realm.status.population', { count: 42 })).toBe('42 online');
  });

  test('missing keys render loudly, never blank', () => {
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
  });

  test('unknown placeholder is left visible', () => {
    expect(interpolate('hi {who}', {})).toBe('hi {who}');
  });

  test('flatten produces dot-keys', () => {
    expect(flatten({ a: { b: 'c' } })).toEqual({ 'a.b': 'c' });
  });

  test('unknown locale falls back to default', () => {
    expect(resolveLocale('xx')).toBe('en');
    expect(resolveLocale('en')).toBe('en');
  });

  describe('edge cases', () => {
    test('interpolates repeated placeholders', () => {
      const t2 = createTranslator({ message: '{name} and {name} again' });
      expect(t2('message', { name: 'Bob' })).toBe('Bob and Bob again');
    });

    test('handles missing params in interpolation', () => {
      const t2 = createTranslator({ msg: 'Hello {name}, welcome' });
      expect(t2('msg', {})).toBe('Hello {name}, welcome');
    });

    test('handles numeric params', () => {
      const t2 = createTranslator({ price: '{gold}g {silver}s' });
      expect(t2('price', { gold: 10, silver: 50 })).toBe('10g 50s');
    });

    test('handles missing params', () => {
      const t2 = createTranslator({ val: 'Value: {x}' });
      expect(t2('val', {})).toBe('Value: {x}');
    });

    test('preserves whitespace', () => {
      const t2 = createTranslator({ spaces: '  lots  of  spaces  ' });
      expect(t2('spaces')).toBe('  lots  of  spaces  ');
    });

    test('handles empty catalog', () => {
      const t3 = createTranslator({});
      expect(t3('any.key')).toBe('⟦any.key⟧');
    });

    test('handles very long keys', () => {
      const t3 = createTranslator({});
      const longKey = 'a'.repeat(100);
      expect(t3(longKey)).toBe(`⟦${longKey}⟧`);
    });
  });
});
