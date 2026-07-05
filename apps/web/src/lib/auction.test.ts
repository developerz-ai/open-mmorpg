import { describe, expect, test } from 'bun:test';
import { buyListing, fetchListings, fetchPriceHistory } from './auction.ts';

describe('auction house (mock worldsvc)', () => {
  test('lists all listings for an empty query', async () => {
    const all = await fetchListings('');
    expect(all.length).toBeGreaterThan(1);
  });

  test('filters listings by a search query', async () => {
    const results = await fetchListings('staff');
    expect(results.every((l) => l.item.toLowerCase().includes('staff'))).toBe(true);
    expect(results.length).toBe(1);
  });

  test('fetches price history points for an item', async () => {
    const history = await fetchPriceHistory('Thornbite Glaive');
    expect(history.points.length).toBeGreaterThan(0);
    expect(history.points.at(-1)?.avgBuyout).toBe(42_000);
  });

  test('a buy intent is idempotent — the same key returns the same outcome', async () => {
    const key = 'test-key-1';
    const first = await buyListing({ listingId: 'l1', idempotencyKey: key });
    const second = await buyListing({ listingId: 'l1', idempotencyKey: key });
    expect(first).toEqual(second);
    expect(first.item).toBe('Thornbite Glaive');
  });

  test('buy result reflects quantity × per-unit buyout', async () => {
    const result = await buyListing({ listingId: 'l3', idempotencyKey: 'test-key-2' });
    expect(result.quantity).toBe(20);
    expect(result.price).toBe(20 * 120);
  });
});
