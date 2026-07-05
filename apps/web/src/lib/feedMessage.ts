import type { FeedEntry } from './feed.ts';
import { t } from './i18n.ts';

/**
 * The `t()` message for one feed entry, by variant. Pure (no JSX) so it's unit-
 * testable in isolation; the `FeedItem` component renders around it.
 */
export function feedMessage(entry: FeedEntry): string {
  switch (entry.kind) {
    case 'boss_kill':
      return t('feed.kind.boss_kill', { actor: entry.actor, target: entry.target });
    case 'world_boss_spawn':
      return t('feed.kind.world_boss_spawn', { target: entry.target, zone: entry.zone });
    case 'faction_shift':
      return t('feed.kind.faction_shift', { actor: entry.actor, zone: entry.zone });
    case 'bounty_posted':
      return t('feed.kind.bounty_posted', { target: entry.target });
    case 'milestone':
      return t('feed.kind.milestone', { target: entry.message });
    default:
      return t('feed.kind.unknown');
  }
}
