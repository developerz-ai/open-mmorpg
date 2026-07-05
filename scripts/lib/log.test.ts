import { expect, test } from 'bun:test';
import { paint } from './log.ts';

test('paint wraps text in ansi codes and resets', () => {
  const out = paint('green', 'ok');
  expect(out).toContain('ok');
  expect(out.startsWith('\x1b[32m')).toBe(true);
  expect(out.endsWith('\x1b[0m')).toBe(true);
});
