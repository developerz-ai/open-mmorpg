import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import {
  type BuyIntent,
  type BuyResult,
  buyShopItem,
  fetchShopCategories,
  fetchShopItems,
  type ShopItem,
} from '../lib/cashShop.ts';

/** Shop items for a category (cached read, stale-while-revalidate). */
export function useShopItems(category: () => string) {
  return useQuery<ShopItem[]>(() => ({
    queryKey: ['shop', 'items', category()],
    queryFn: () => fetchShopItems(category()),
    staleTime: 30_000,
  }));
}

/** Available categories (cached read). */
export function useShopCategories() {
  return useQuery<string[]>(() => ({
    queryKey: ['shop', 'categories'],
    queryFn: () => fetchShopCategories(),
    staleTime: 300_000,
  }));
}

/** Buy intent mutation. Invalidates listings on the server's confirmed outcome. */
export function useBuyShopItem() {
  const qc = useQueryClient();
  return useMutation<BuyResult, Error, BuyIntent>(() => ({
    mutationFn: (intent: BuyIntent) => buyShopItem(intent),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['shop', 'items'] }),
  }));
}
