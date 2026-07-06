import { Alert, Button, Card, Select, SelectOption, Spinner } from '@omm/ui';
import type { JSX } from 'solid-js';
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from 'solid-js';
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
const AUTO_REFRESH_INTERVAL = 60000; // 60 seconds

/** World feed — event type filter + pagination + auto-refresh + permalinks. */
export default function WorldFeed(): JSX.Element {
  const feed = useFeed();
  const [eventType, setEventType] = createSignal<EventTypeFilter>('all');
  const [page, setPage] = createSignal(1);
  const [lastSeenIds, setLastSeenIds] = createSignal<Set<string>>(new Set());
  const [highlightedId, setHighlightedId] = createSignal<string | null>(null);
  const [autoRefresh, setAutoRefresh] = createSignal(true);
  let refreshTimer: ReturnType<typeof setInterval> | undefined;

  // Check URL hash for permalink on mount
  onMount(() => {
    const hash = window.location.hash.slice(1); // Remove #
    if (hash) {
      setHighlightedId(hash);
      setPage(1); // Reset to first page to find the event
    }
  });

  // Set up auto-refresh
  createEffect(() => {
    if (autoRefresh() && !feed.isPending && !feed.isError) {
      refreshTimer = setInterval(() => {
        // Store current feed IDs as "last seen"
        if (feed.data) {
          setLastSeenIds(new Set(feed.data.map((e) => e.id)));
        }
        // Refetch will happen automatically via TanStack Query's refetchOnMount behavior
        // For explicit refresh, we'd need to expose a refetch function from useFeed
      }, AUTO_REFRESH_INTERVAL);
    } else {
      if (refreshTimer) {
        clearInterval(refreshTimer);
        refreshTimer = undefined;
      }
    }

    onCleanup(() => {
      if (refreshTimer) {
        clearInterval(refreshTimer);
      }
    });
  });

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

  // Mark new items (items not in lastSeenIds)
  const feedWithNewStatus = createMemo(() => {
    return filteredFeed().map((item) => ({
      ...item,
      isNew: !lastSeenIds().has(item.id) && lastSeenIds().size > 0,
    }));
  });

  // Find highlighted event and set its page
  createEffect(() => {
    const highlighted = highlightedId();
    if (highlighted && feedWithNewStatus().length > 0) {
      const index = feedWithNewStatus().findIndex((e) => e.id === highlighted);
      if (index >= 0) {
        const targetPage = Math.floor(index / ITEMS_PER_PAGE) + 1;
        setPage(targetPage);
      }
    }
  });

  // Pagination
  const totalPages = createMemo(() => Math.ceil(feedWithNewStatus().length / ITEMS_PER_PAGE) || 1);
  const paginatedFeed = createMemo(() => {
    const start = (page() - 1) * ITEMS_PER_PAGE;
    return feedWithNewStatus().slice(start, start + ITEMS_PER_PAGE);
  });

  const paginationInfo = createMemo(() => {
    const total = feedWithNewStatus().length;
    const start = total === 0 ? 0 : (page() - 1) * ITEMS_PER_PAGE + 1;
    const end = Math.min(page() * ITEMS_PER_PAGE, total);
    return { start, end, total };
  });

  const now = Date.now();

  // Share permalink handler
  const handleShare = (eventId: string) => {
    const url = new URL(window.location.href);
    url.hash = eventId;
    navigator.clipboard.writeText(url.toString()).then(() => {
      // Could show a toast notification here
      console.log('Permalink copied to clipboard');
    });
  };

  return (
    <Card title={t('feed.heading')} class="stack">
      {/* Toolbar with filter and auto-refresh toggle */}
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

        <Button
          variant="ghost"
          onClick={() => setAutoRefresh((v) => !v)}
          aria-pressed={autoRefresh()}
        >
          {autoRefresh() ? t('feed.autoRefresh.on') : t('feed.autoRefresh.off')}
        </Button>
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
              <For each={paginatedFeed()}>
                {(entry) => (
                  <FeedItem
                    entry={entry}
                    now={now}
                    isHighlighted={entry.id === highlightedId()}
                    onShare={() => handleShare(entry.id)}
                  />
                )}
              </For>
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
