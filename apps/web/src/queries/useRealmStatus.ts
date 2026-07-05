import { useQuery } from '@tanstack/solid-query';
import { fetchRealmStatus } from '../lib/realm.ts';

/**
 * Server state lives only in TanStack Query — never a hand-rolled fetch cache.
 * Thin routes/components consume this hook; the fetch + validation lives in
 * `lib/realm.ts`.
 */
export function useRealmStatus() {
  return useQuery(() => ({
    queryKey: ['realm', 'status'],
    queryFn: fetchRealmStatus,
    refetchInterval: 30_000,
  }));
}
