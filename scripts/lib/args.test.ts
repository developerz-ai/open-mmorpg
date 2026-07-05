import { expect, test } from 'bun:test';
import { parseArgs } from './args.ts';

test('parses positionals', () => {
  expect(parseArgs(['web', 'shard']).positionals).toEqual(['web', 'shard']);
});

test('parses boolean and valued flags', () => {
  const { flags } = parseArgs(['--verbose', '--target=web', '--name', 'aria']);
  expect(flags.verbose).toBe(true);
  expect(flags.target).toBe('web');
  expect(flags.name).toBe('aria');
});

test('mixes positionals and a trailing boolean flag', () => {
  const parsed = parseArgs(['build', 'web', '--release']);
  expect(parsed.positionals).toEqual(['build', 'web']);
  expect(parsed.flags.release).toBe(true);
});

test('greedy value capture binds the next token to a flag', () => {
  const parsed = parseArgs(['--target', 'web', 'extra']);
  expect(parsed.flags.target).toBe('web');
  expect(parsed.positionals).toEqual(['extra']);
});
