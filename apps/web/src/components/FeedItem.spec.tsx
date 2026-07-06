import { describe, expect, test } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { FeedItem } from './FeedItem';
import type { FeedEntry } from '../lib/feed';

describe('FeedItem component tests', () => {
  const mockEntry: FeedEntry = {
    kind: 'boss_kill',
    at: Date.now() - 120000, // 2 minutes ago
  };

  const mockEntryWithoutTimestamp: FeedEntry = {
    kind: 'milestone',
    at: undefined,
  };

  test('renders feed item with timestamp', () => {
    const now = Date.now();
    render(() => <FeedItem entry={mockEntry} now={now} />);

    // Check badge renders
    expect(screen.getByText('boss_kill')).toBeInTheDocument();

    // Check relative time is shown
    expect(screen.getByText(/ago/)).toBeInTheDocument();
  });

  test('renders feed item without timestamp', () => {
    render(() => <FeedItem entry={mockEntryWithoutTimestamp} now={Date.now()} />);

    // Check badge renders
    expect(screen.getByText('milestone')).toBeInTheDocument();

    // No timestamp should be shown
    const timeElements = document.querySelectorAll('time');
    expect(timeElements.length).toBe(0);
  });

  test('uses correct badge for different kinds', () => {
    // Test boss_kill
    const { unmount: unmount1 } = render(() => <FeedItem entry={mockEntry} now={Date.now()} />);
    expect(screen.getByText('boss_kill')).toBeInTheDocument();
    unmount1();

    // Test world_boss_spawn
    const worldBossEntry: FeedEntry = { kind: 'world_boss_spawn', at: Date.now() };
    const { unmount: unmount2 } = render(() => <FeedItem entry={worldBossEntry} now={Date.now()} />);
    expect(screen.getByText('world_boss_spawn')).toBeInTheDocument();
    unmount2();

    // Test faction_shift
    const factionEntry: FeedEntry = { kind: 'faction_shift', at: Date.now() };
    render(() => <FeedItem entry={factionEntry} now={Date.now()} />);
    expect(screen.getByText('faction_shift')).toBeInTheDocument();
  });
});
