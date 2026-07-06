import { describe, expect, test } from 'bun:test';
import {
  buyShopItem,
  fetchPurchaseHistory,
  fetchShopCategories,
  fetchShopItems,
} from './cashShop.ts';

describe('cash shop (mock worldsvc)', () => {
  test('lists all shop items', async () => {
    const all = await fetchShopItems('');
    expect(all.length).toBe(5);
    expect(all.at(0)?.name).toBe('XP Boost (24h)');
  });

  test('filters items by category', async () => {
    const cosmetics = await fetchShopItems('cosmetics');
    expect(cosmetics.length).toBe(1);
    expect(cosmetics.at(0)?.category).toBe('cosmetics');
    expect(cosmetics.at(0)?.name).toBe('Gold Loot Box');
  });

  test('fetches available categories', async () => {
    const cats = await fetchShopCategories();
    expect(cats).toContain('boosts');
    expect(cats).toContain('cosmetics');
    expect(cats).toContain('account');
    expect(cats).toContain('mounts');
  });

  test('a buy intent is idempotent — the same key returns the same outcome', async () => {
    const key = 'test-key-1';
    const first = await buyShopItem({ itemId: 'item1', idempotencyKey: key });
    const second = await buyShopItem({ itemId: 'item1', idempotencyKey: key });
    expect(first).toEqual(second);
    expect(first.item).toBe('XP Boost (24h)');
    expect(first.price).toBe(500);
  });

  test('buy result reflects the item price', async () => {
    const result = await buyShopItem({ itemId: 'item5', idempotencyKey: 'test-key-2' });
    expect(result.item).toBe('Mount: Shadow Stallion');
    expect(result.price).toBe(5000);
  });

  test('fetches purchase history for authenticated account', async () => {
    const history = await fetchPurchaseHistory();
    expect(history.length).toBeGreaterThan(0);
    expect(history.at(0)?.item).toBe('XP Boost (24h)');
    expect(history.at(0)?.price).toBe(500);
    expect(history.at(0)?.currency).toBe('credits');
  });

  test('purchase history entries are sorted by most recent first', async () => {
    const history = await fetchPurchaseHistory();
    expect(history.at(0)?.purchasedAt).toBe('2026-07-05T10:30:00.000Z');
    expect(history.at(1)?.purchasedAt).toBe('2026-07-04T15:20:00.000Z');
  });
});
