import { describe, expect, test } from 'bun:test';
import { fetchFeed } from './feed.ts';

describe('world feed (mock worldsvc)', () => {
  test('parses each known event into its typed variant', async () => {
    const entries = await fetchFeed();
    const bossKill = entries.find((e) => e.kind === 'boss_kill');
    expect(bossKill).toBeDefined();
    if (bossKill?.kind === 'boss_kill') expect(bossKill.target).toBe('Emberdrake');
  });

  test('an unknown variant degrades gracefully instead of crashing the stream', async () => {
    const entries = await fetchFeed();
    // The seed includes a `meteor_shower` event we do not model.
    const unknown = entries.find((e) => e.kind === 'unknown');
    expect(unknown).toBeDefined();
    expect(unknown?.id).toBe('e6');
    // The rest of the stream still parsed.
    expect(entries.length).toBe(6);
  });
});
