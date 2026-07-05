import { Badge, type BadgeTone } from '@omm/ui';
import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import type { FeedEntry } from '../lib/feed.ts';
import { feedMessage } from '../lib/feedMessage.ts';
import { fmt } from '../lib/i18n.ts';

const TONES: Record<FeedEntry['kind'], BadgeTone> = {
  boss_kill: 'danger',
  world_boss_spawn: 'warning',
  faction_shift: 'accent',
  bounty_posted: 'danger',
  milestone: 'success',
  unknown: 'neutral',
};

/** One feed line: a kind badge, the event copy, and a relative timestamp. */
export function FeedItem(props: { entry: FeedEntry; now: number }): JSX.Element {
  return (
    <li class="feed-item">
      <Badge tone={TONES[props.entry.kind]}>{props.entry.kind}</Badge>
      <span class="feed-item__text">{feedMessage(props.entry)}</span>
      <Show when={props.entry.at}>
        {(at) => (
          <time class="feed-item__time text-fg-muted" datetime={at()}>
            {fmt.relative(new Date(at()), props.now)}
          </time>
        )}
      </Show>
    </li>
  );
}
