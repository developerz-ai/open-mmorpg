import { NetworkError } from '../errors.ts';
import type { MockRequest, MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock worldsvc cash shop. Seeded items, categories, and deterministic purchase
 * history. Buy is idempotent: a repeated idempotency key returns the same outcome.
 */

const items = [
  {
    id: 'item1',
    name: 'XP Boost (24h)',
    description: 'Double experience gains for 24 hours',
    category: 'boosts',
    price: 500,
    currency: 'credits' as const,
  },
  {
    id: 'item2',
    name: 'Gold Loot Box',
    description: 'Contains a random cosmetic item',
    category: 'cosmetics',
    price: 1000,
    currency: 'credits' as const,
  },
  {
    id: 'item3',
    name: 'Character Slot',
    description: 'Unlock an additional character slot',
    category: 'account',
    price: 2000,
    currency: 'credits' as const,
  },
  {
    id: 'item4',
    name: 'Name Change Token',
    description: 'Change your character name',
    category: 'account',
    price: 1500,
    currency: 'credits' as const,
  },
  {
    id: 'item5',
    name: 'Mount: Shadow Stallion',
    description: 'Exclusive shadow-themed mount',
    category: 'mounts',
    price: 5000,
    currency: 'credits' as const,
  },
];

const categories = ['boosts', 'cosmetics', 'account', 'mounts'];

const mockHistory = [
  {
    id: 'purchase1',
    item: 'XP Boost (24h)',
    price: 500,
    currency: 'credits' as const,
    purchasedAt: '2026-07-05T10:30:00.000Z',
  },
  {
    id: 'purchase2',
    item: 'Gold Loot Box',
    price: 1000,
    currency: 'credits' as const,
    purchasedAt: '2026-07-04T15:20:00.000Z',
  },
];

const purchases = new Map<string, { item: string; price: number }>();

function asBody(body: unknown): { itemId: string; idempotencyKey: string } {
  const b = (body ?? {}) as Partial<Record<'itemId' | 'idempotencyKey', string>>;
  return { itemId: b.itemId ?? '', idempotencyKey: b.idempotencyKey ?? '' };
}

export const shopRoutes: MockRoute[] = [
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/shop/items'),
    resolve: ({ query }: MockRequest) => {
      const category = query.get('category');
      const filtered = category ? items.filter((i) => i.category === category) : items;
      return { items: filtered };
    },
  },
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/shop/categories'),
    resolve: () => ({ categories }),
  },
  {
    backend: 'worldsvc',
    method: 'POST',
    test: pattern('/shop/buy'),
    resolve: ({ body }: MockRequest) => {
      const { itemId, idempotencyKey } = asBody(body);
      const prior = purchases.get(idempotencyKey);
      if (prior) return prior;
      const item = items.find((i) => i.id === itemId);
      if (!item) throw new NetworkError(`no item ${itemId}`);
      const outcome = { item: item.name, price: item.price };
      purchases.set(idempotencyKey, outcome);
      return outcome;
    },
  },
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/shop/purchases'),
    resolve: () => ({ entries: mockHistory }),
  },
];
