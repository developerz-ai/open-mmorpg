import { Badge, type BadgeTone } from '@omm/ui';
import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import type { FeedEntry } from '../lib/feed.ts';
import { feedMessage } from '../lib/feedMessage.ts';
import { fmt, t } from '../lib/i18n.ts';

const TONES: Record<FeedEntry['kind'], BadgeTone> = {
  boss_kill: 'danger',
  world_boss_spawn: 'warning',
  faction_shift: 'accent',
  bounty_posted: 'danger',
  milestone: 'success',
  unknown: 'neutral',
};

/** One feed line: a kind badge, the event copy, a relative timestamp, and share button. */
export function FeedItem(props: {
  entry: FeedEntry & { isNew?: boolean };
  now: number;
  isHighlighted?: boolean;
  onShare?: () => void;
}): JSX.Element {
  return (
    <li
      class="feed-item"
      classList={{
        'feed-item--new': props.entry.isNew,
        'feed-item--highlighted': props.isHighlighted,
      }}
      id={props.isHighlighted ? `event-${props.entry.id}` : undefined}
    >
      <Show when={props.entry.isNew}>
        <span class="feed-item__new-badge" role="status">
          <span class="sr-only">{t('feed.newBadge')}</span>
          <span aria-hidden="true">●</span>
        </span>
      </Show>
      <Badge tone={TONES[props.entry.kind]}>{props.entry.kind}</Badge>
      <span class="feed-item__text">{feedMessage(props.entry)}</span>
      <Show when={props.entry.at}>
        {(at) => (
          <time class="feed-item__time text-fg-muted" datetime={at()}>
            {fmt.relative(new Date(at()), props.now)}
          </time>
        )}
      </Show>
      <Show when={props.onShare}>
        <button
          type="button"
          class="feed-item__share link-button"
          onClick={props.onShare}
          aria-label={t('feed.shareEvent')}
          title={t('feed.shareEvent')}
        >
          {t('feed.share')}
        </button>
      </Show>
    </li>
  );
}
