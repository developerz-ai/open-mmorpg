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
});
