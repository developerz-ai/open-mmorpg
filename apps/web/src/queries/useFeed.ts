import { useQuery } from '@tanstack/solid-query';
import { type FeedEntry, fetchFeed } from '../lib/feed.ts';

/** The world feed — polled (stale-while-revalidate), no bespoke socket for v1. */
export function useFeed() {
  return useQuery<FeedEntry[]>(() => ({
    queryKey: ['world', 'feed'],
    queryFn: fetchFeed,
    refetchInterval: 30_000,
    retry: 3,
    retryDelay: (attempt) => Math.min(1000 * 2 ** (attempt - 1), 30000),
  }));
}
