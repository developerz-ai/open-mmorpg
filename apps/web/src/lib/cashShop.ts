import { z } from 'zod';
import { request } from './api.ts';

/**
 * Cash shop — **read view + intent submit, nothing else.** Featured items,
 * categories, and currency display are Dragonfly-cached reads; purchasing POSTs
 * an idempotent intent to worldsvc, which runs the single Yugabyte transaction
 * and returns the outcome. The web never mints or moves value.
 * → docs/specs/web-client/cash-shop
 */
export const ShopItemSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string().optional(),
  category: z.string(),
  price: z.number().int().nonnegative(),
  currency: z.literal('credits'),
});
export type ShopItem = z.infer<typeof ShopItemSchema>;

export const ShopItemsSchema = z.object({ items: z.array(ShopItemSchema) });

export const CategorySchema = z.string();
export type Category = z.infer<typeof CategorySchema>;

export const CategoriesSchema = z.object({ categories: z.array(CategorySchema) });

export const BuyResultSchema = z.object({
  item: z.string(),
  price: z.number().int().nonnegative(),
});
export type BuyResult = z.infer<typeof BuyResultSchema>;

/** Fetch shop items, optionally filtered by category (cached read). */
export async function fetchShopItems(category: string): Promise<ShopItem[]> {
  const path = category ? `/shop/items?category=${encodeURIComponent(category)}` : '/shop/items';
  const { items } = await request({ backend: 'worldsvc', path, schema: ShopItemsSchema });
  return items;
}

/** Fetch available categories (cached read). */
export async function fetchShopCategories(): Promise<Category[]> {
  const { categories } = await request({
    backend: 'worldsvc',
    path: '/shop/categories',
    schema: CategoriesSchema,
  });
  return categories;
}

export interface BuyIntent {
  itemId: string;
  /** Idempotency key — a retried packet can't double-submit; worldsvc enforces. */
  idempotencyKey: string;
}

/** Submit a shop purchase intent. "Bought" is only the server's confirmed outcome. */
export function buyShopItem(intent: BuyIntent): Promise<BuyResult> {
  return request({
    backend: 'worldsvc',
    path: '/shop/buy',
    method: 'POST',
    body: intent,
    schema: BuyResultSchema,
    auth: true,
  });
}
