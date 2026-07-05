import { z } from 'zod';
import { request } from './api.ts';

/**
 * Auction house — **read view + intent submit, nothing else.** Browse/search and
 * price history are Dragonfly-cached reads; buying POSTs an idempotent intent to
 * worldsvc, which runs the single Yugabyte transaction and returns the outcome.
 * The web never mints or moves value. → docs/specs/web-client/auction-house
 */
export const ListingSchema = z.object({
  id: z.string(),
  item: z.string(),
  quantity: z.number().int().positive(),
  buyoutPer: z.number().int().nonnegative(),
  seller: z.string(),
  endsAt: z.string(),
});
export type Listing = z.infer<typeof ListingSchema>;

export const ListingsSchema = z.object({ listings: z.array(ListingSchema) });

export const PricePointSchema = z.object({ at: z.string(), avgBuyout: z.number().nonnegative() });
export type PricePoint = z.infer<typeof PricePointSchema>;

export const PriceHistorySchema = z.object({ item: z.string(), points: z.array(PricePointSchema) });
export type PriceHistory = z.infer<typeof PriceHistorySchema>;

export const BuyResultSchema = z.object({
  item: z.string(),
  price: z.number().int().nonnegative(),
  quantity: z.number().int().positive(),
});
export type BuyResult = z.infer<typeof BuyResultSchema>;

/** Search AH listings (cached read). Empty query returns all. */
export async function fetchListings(query: string): Promise<Listing[]> {
  const path = query ? `/auction/listings?q=${encodeURIComponent(query)}` : '/auction/listings';
  const { listings } = await request({ backend: 'worldsvc', path, schema: ListingsSchema });
  return listings;
}

/** Price history for an item (cached read). */
export function fetchPriceHistory(item: string): Promise<PriceHistory> {
  return request({
    backend: 'worldsvc',
    path: `/auction/history/${encodeURIComponent(item)}`,
    schema: PriceHistorySchema,
  });
}

export interface BuyIntent {
  listingId: string;
  /** Idempotency key — a retried packet can't double-submit; worldsvc enforces. */
  idempotencyKey: string;
}

/** Submit a buy intent. "Bought" is only the server's confirmed outcome. */
export function buyListing(intent: BuyIntent): Promise<BuyResult> {
  return request({
    backend: 'worldsvc',
    path: '/auction/buy',
    method: 'POST',
    body: intent,
    schema: BuyResultSchema,
    auth: true,
  });
}
