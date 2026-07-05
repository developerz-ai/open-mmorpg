import { Alert, Card, Spinner } from '@omm/ui';
import type { JSX } from 'solid-js';
import { For, Match, Show, Switch } from 'solid-js';
import { FeedItem } from '../components/FeedItem.tsx';
import { errorKey } from '../lib/errors.ts';
import { t } from '../lib/i18n.ts';
import { useFeed } from '../queries/useFeed.ts';

/** World feed — a scrollable stream of living-world events (read-only projection). */
export default function WorldFeed(): JSX.Element {
  const feed = useFeed();
  const now = Date.now();

  return (
    <Card title={t('feed.heading')}>
      <Switch>
        <Match when={feed.isPending}>
          <Spinner label={t('common.loading')} />
        </Match>
        <Match when={feed.isError}>
          <Alert tone="error">{t(errorKey(feed.error))}</Alert>
        </Match>
        <Match when={feed.data}>
          {(items) => (
            <Show
              when={items().length > 0}
              fallback={<p class="text-fg-muted">{t('feed.empty')}</p>}
            >
              <ul class="feed-list">
                <For each={items()}>{(entry) => <FeedItem entry={entry} now={now} />}</For>
              </ul>
            </Show>
          )}
        </Match>
      </Switch>
    </Card>
  );
}
