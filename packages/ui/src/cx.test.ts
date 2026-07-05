import { expect, test } from 'bun:test';
import { cx } from './cx.ts';

test('cx joins truthy classes and drops falsy', () => {
  expect(cx('a', false, 'b', null, undefined, '')).toBe('a b');
  expect(cx()).toBe('');
});
