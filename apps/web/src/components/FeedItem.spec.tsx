import { describe, expect, test } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { FeedItem } from './FeedItem';
import type { FeedEntry } from '../lib/feed';

describe('FeedItem component tests', () => {
  const mockEntry: FeedEntry = {
    id: 'boss-1',
    kind: 'boss_kill',
    at: new Date(Date.now() - 120000).toISOString(), // 2 minutes ago
    actor: 'player1',
    target: 'dragon',
  };

  const mockEntryWithoutTimestamp: FeedEntry = {
    id: 'milestone-1',
    kind: 'milestone',
    at: '',
    message: 'Server reached 1000 players!',
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
    const worldBossEntry: FeedEntry = {
      id: 'world-boss-1',
      kind: 'world_boss_spawn',
      at: new Date().toISOString(),
      target: 'Ancient Dragon',
      zone: 'Dragonfire Peaks',
    };
    const { unmount: unmount2 } = render(() => <FeedItem entry={worldBossEntry} now={Date.now()} />);
    expect(screen.getByText('world_boss_spawn')).toBeInTheDocument();
    unmount2();

    // Test faction_shift
    const factionEntry: FeedEntry = {
      id: 'faction-1',
      kind: 'faction_shift',
      at: new Date().toISOString(),
      actor: 'Shadow Syndicate',
      zone: 'Nightfall Valley',
    };
    render(() => <FeedItem entry={factionEntry} now={Date.now()} />);
    expect(screen.getByText('faction_shift')).toBeInTheDocument();
  });
});
