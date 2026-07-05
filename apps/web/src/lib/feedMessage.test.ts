import { describe, expect, test } from 'bun:test';
import type { FeedEntry } from './feed.ts';
import { feedMessage } from './feedMessage.ts';

describe('feedMessage', () => {
  test('interpolates a boss kill', () => {
    const entry: FeedEntry = {
      kind: 'boss_kill',
      id: 'e1',
      at: '2026-07-05T00:00:00Z',
      actor: 'Vanguard',
      target: 'Emberdrake',
    };
    expect(feedMessage(entry)).toBe('Vanguard felled Emberdrake.');
  });

  test('renders a milestone from its message', () => {
    const entry: FeedEntry = {
      kind: 'milestone',
      id: 'e5',
      at: '2026-07-05T00:00:00Z',
      message: 'The realm reached 100,000 souls.',
    };
    expect(feedMessage(entry)).toBe('The realm reached 100,000 souls.');
  });

  test('an unknown entry gets neutral copy, never a crash', () => {
    const entry: FeedEntry = { kind: 'unknown', id: 'x', at: '' };
    expect(feedMessage(entry)).toBe('Something stirs in the world.');
  });
});
