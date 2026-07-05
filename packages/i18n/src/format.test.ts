import { describe, expect, test } from 'bun:test';
import { createFormatters } from './format.ts';

describe('formatters (en)', () => {
  const f = createFormatters('en');

  test('integer groups thousands', () => {
    expect(f.integer(1204)).toBe('1,204');
  });

  test('gold truncates and groups', () => {
    expect(f.gold(12_500.9)).toBe('12,500');
  });

  test('compact abbreviates', () => {
    expect(f.compact(12_400)).toBe('12.4K');
  });

  test('date is deterministic for a fixed instant', () => {
    // Fixed UTC instant → medium date. Locale data is stable in CI.
    const d = new Date('2026-07-05T12:00:00Z');
    expect(f.date(d)).toContain('2026');
  });

  test('relative time reads from a fixed clock', () => {
    const now = new Date('2026-07-05T12:00:00Z');
    const twoHoursAgo = new Date('2026-07-05T10:00:00Z');
    expect(f.relative(twoHoursAgo, now)).toBe('2 hours ago');
  });

  test('relative time handles the future', () => {
    const now = new Date('2026-07-05T12:00:00Z');
    const inThreeDays = new Date('2026-07-08T12:00:00Z');
    expect(f.relative(inThreeDays, now)).toBe('in 3 days');
  });
});
