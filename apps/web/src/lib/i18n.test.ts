import { describe, expect, test } from 'bun:test';
import { fmt, t } from './i18n.ts';

describe('i18n', () => {
  test('translator resolves nested keys via dot lookup', () => {
    expect(t('common.loading')).toBe('Loading…');
    expect(t('nav.home')).toBe('Home');
    expect(t('auth.login.heading')).toBe('Log in');
  });

  test('translator interpolates params', () => {
    expect(t('common.gold', { amount: 42 })).toBe('42 gold');
    expect(t('realm.title', { name: 'Azeroth' })).toBe('Azeroth');
    expect(t('auth.login.success', { name: 'Aria' })).toBe('Welcome back, Aria.');
  });

  test('missing keys render loudly, never blank', () => {
    expect(t('does.not.exist')).toBe('⟦does.not.exist⟧');
    expect(t('realm.missing')).toBe('⟦realm.missing⟧');
  });

  test('formatters provide integer grouping', () => {
    expect(fmt.integer(1204)).toBe('1,204');
    expect(fmt.integer(42)).toBe('42');
  });

  test('formatters provide decimal formatting', () => {
    expect(fmt.decimal(12.5)).toBe('12.5');
    expect(fmt.decimal(1204.567)).toBe('1,204.57');
  });

  test('formatters provide compact notation', () => {
    expect(fmt.compact(12_400)).toBe('12.4K');
    expect(fmt.compact(1_200_000)).toBe('1.2M');
  });

  test('formatters provide gold truncation and grouping', () => {
    expect(fmt.gold(12_500.9)).toBe('12,500');
    expect(fmt.gold(42.7)).toBe('42');
  });

  test('formatters provide date formatting', () => {
    const d = new Date('2026-07-05T12:00:00Z');
    expect(fmt.date(d)).toContain('2026');
  });

  test('formatters provide date-time formatting', () => {
    const d = new Date('2026-07-05T12:00:00Z');
    expect(fmt.dateTime(d)).toContain('2026');
  });

  test('formatters provide relative time', () => {
    const now = new Date('2026-07-05T12:00:00Z');
    const twoHoursAgo = new Date('2026-07-05T10:00:00Z');
    expect(fmt.relative(twoHoursAgo, now)).toBe('2 hours ago');
  });

  test('formatters handle future relative time', () => {
    const now = new Date('2026-07-05T12:00:00Z');
    const inThreeDays = new Date('2026-07-08T12:00:00Z');
    expect(fmt.relative(inThreeDays, now)).toBe('in 3 days');
  });
});
