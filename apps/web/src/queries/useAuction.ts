import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import {
  type BuyIntent,
  type BuyResult,
  buyListing,
  fetchListings,
  fetchPriceHistory,
  type Listing,
  type PriceHistory,
} from '../lib/auction.ts';

/** AH listings for a search query (cached read, stale-while-revalidate). */
export function useListings(query: () => string) {
  return useQuery<Listing[]>(() => ({
    queryKey: ['auction', 'listings', query()],
    queryFn: () => fetchListings(query()),
    staleTime: 15_000,
  }));
}

/** Price history for an item (cached read). */
export function usePriceHistory(item: () => string) {
  return useQuery<PriceHistory>(() => ({
    queryKey: ['auction', 'history', item()],
    queryFn: () => fetchPriceHistory(item()),
    enabled: item() !== '',
    staleTime: 60_000,
  }));
}

/** Buy intent mutation. Invalidates listings on the server's confirmed outcome. */
export function useBuyListing() {
  const qc = useQueryClient();
  return useMutation<BuyResult, Error, BuyIntent>(() => ({
    mutationFn: (intent: BuyIntent) => buyListing(intent),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['auction', 'listings'] }),
  }));
}
