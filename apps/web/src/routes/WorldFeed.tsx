import { Alert, Button, Card, Select, SelectOption, Spinner } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createEffect, createMemo, createSignal, For, Match, Show, Switch } from 'solid-js';
import { FeedItem } from '../components/FeedItem.tsx';
import { errorKey } from '../lib/errors.ts';
import { t } from '../lib/i18n.ts';
import { useFeed } from '../queries/useFeed.ts';

/** Event type filter options */
const EVENT_TYPES = [
  'all',
  'boss_kill',
  'world_boss_spawn',
  'faction_shift',
  'bounty_posted',
  'milestone',
] as const;
type EventTypeFilter = (typeof EVENT_TYPES)[number];

const ITEMS_PER_PAGE = 20;

/** World feed — event type filter + pagination + a scrollable stream of living-world events (read-only projection). */
export default function WorldFeed(): JSX.Element {
  const feed = useFeed();
  const [eventType, setEventType] = createSignal<EventTypeFilter>('all');
  const [page, setPage] = createSignal(1);

  // Reset page on filter change
  createEffect(() => {
    setPage(1);
  });

  // Filter by event type
  const filteredFeed = createMemo(() => {
    if (feed.data === undefined) return [];
    const filter = eventType();
    if (filter === 'all') return feed.data;
    return feed.data.filter((entry) => entry.kind === filter);
  });

  // Pagination
  const totalPages = createMemo(() => Math.ceil(filteredFeed().length / ITEMS_PER_PAGE) || 1);
  const paginatedFeed = createMemo(() => {
    const start = (page() - 1) * ITEMS_PER_PAGE;
    return filteredFeed().slice(start, start + ITEMS_PER_PAGE);
  });

  const paginationInfo = createMemo(() => {
    const total = filteredFeed().length;
    const start = total === 0 ? 0 : (page() - 1) * ITEMS_PER_PAGE + 1;
    const end = Math.min(page() * ITEMS_PER_PAGE, total);
    return { start, end, total };
  });

  const now = Date.now();

  return (
    <Card title={t('feed.heading')} class="stack">
      {/* Event type filter */}
      <div class="toolbar">
        <Select
          id="feed-filter"
          label={t('feed.filter.label')}
          value={eventType()}
          onChange={(v) => setEventType(v as EventTypeFilter)}
        >
          <SelectOption value="all">{t('feed.filter.all')}</SelectOption>
          <SelectOption value="boss_kill">{t('feed.filter.bossKill')}</SelectOption>
          <SelectOption value="world_boss_spawn">{t('feed.filter.worldBossSpawn')}</SelectOption>
          <SelectOption value="faction_shift">{t('feed.filter.factionShift')}</SelectOption>
          <SelectOption value="bounty_posted">{t('feed.filter.bountyPosted')}</SelectOption>
          <SelectOption value="milestone">{t('feed.filter.milestone')}</SelectOption>
        </Select>
      </div>

      <Switch>
        <Match when={feed.isPending}>
          <Spinner label={t('common.loading')} />
        </Match>
        <Match when={feed.isError}>
          <Alert tone="error">{t(errorKey(feed.error))}</Alert>
        </Match>
        <Match when={feed.data}>
          <Show
            when={paginatedFeed().length > 0}
            fallback={<p class="text-fg-muted">{t('feed.empty')}</p>}
          >
            <ul class="feed-list">
              <For each={paginatedFeed()}>{(entry) => <FeedItem entry={entry} now={now} />}</For>
            </ul>

            {/* Pagination controls */}
            <div class="pagination">
              <Button
                variant="ghost"
                disabled={page() === 1}
                onClick={() => setPage((p) => Math.max(1, p - 1))}
              >
                {t('feed.pagination.prev')}
              </Button>
              <span class="pagination-info">
                {t('feed.pagination.page', { current: page(), total: totalPages() })}
              </span>
              <Button
                variant="ghost"
                disabled={page() === totalPages()}
                onClick={() => setPage((p) => Math.min(totalPages(), p + 1))}
              >
                {t('feed.pagination.next')}
              </Button>
              <span class="pagination-info text-fg-muted">
                {t('feed.pagination.showing', paginationInfo())}
              </span>
            </div>
          </Show>
        </Match>
      </Switch>
    </Card>
  );
}
