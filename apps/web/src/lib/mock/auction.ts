import { NetworkError } from '../errors.ts';
import type { MockRequest, MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock worldsvc auction house. Seeded listings + a deterministic price series so
 * search, the chart, and buy flows render stably. Buy is idempotent: a repeated
 * idempotency key returns the same recorded outcome (mirrors the worldsvc rule).
 */
interface Row {
  id: string;
  item: string;
  quantity: number;
  buyoutPer: number;
  seller: string;
  endsAt: string;
}

const listings: Row[] = [
  {
    id: 'l1',
    item: 'Thornbite Glaive',
    quantity: 1,
    buyoutPer: 42_000,
    seller: 'Aria',
    endsAt: '2026-07-06T12:00:00.000Z',
  },
  {
    id: 'l2',
    item: 'Cinder Staff',
    quantity: 1,
    buyoutPer: 31_500,
    seller: 'Kael',
    endsAt: '2026-07-06T18:00:00.000Z',
  },
  {
    id: 'l3',
    item: 'Sylvan Herb',
    quantity: 20,
    buyoutPer: 120,
    seller: 'Bryn',
    endsAt: '2026-07-05T20:00:00.000Z',
  },
  {
    id: 'l4',
    item: 'Emberstone',
    quantity: 5,
    buyoutPer: 890,
    seller: 'Cass',
    endsAt: '2026-07-07T09:00:00.000Z',
  },
];

const history: Record<string, { at: string; avgBuyout: number }[]> = {
  'thornbite glaive': [
    { at: '2026-07-01T00:00:00.000Z', avgBuyout: 39_000 },
    { at: '2026-07-02T00:00:00.000Z', avgBuyout: 40_500 },
    { at: '2026-07-03T00:00:00.000Z', avgBuyout: 41_200 },
    { at: '2026-07-04T00:00:00.000Z', avgBuyout: 42_000 },
  ],
};

const purchases = new Map<string, { item: string; price: number; quantity: number }>();

function asBody(body: unknown): { listingId: string; idempotencyKey: string } {
  const b = (body ?? {}) as Partial<Record<'listingId' | 'idempotencyKey', string>>;
  return { listingId: b.listingId ?? '', idempotencyKey: b.idempotencyKey ?? '' };
}

export const auctionRoutes: MockRoute[] = [
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/auction/listings'),
    resolve: ({ query }: MockRequest) => {
      const q = (query.get('q') ?? '').toLowerCase();
      return { listings: q ? listings.filter((l) => l.item.toLowerCase().includes(q)) : listings };
    },
  },
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/auction/history/:item'),
    resolve: ({ params }: MockRequest) => {
      const item = params.item ?? '';
      return { item, points: history[item.toLowerCase()] ?? [] };
    },
  },
  {
    backend: 'worldsvc',
    method: 'POST',
    test: pattern('/auction/buy'),
    resolve: ({ body }: MockRequest) => {
      const { listingId, idempotencyKey } = asBody(body);
      const prior = purchases.get(idempotencyKey);
      if (prior) return prior;
      const row = listings.find((l) => l.id === listingId);
      if (!row) throw new NetworkError(`no listing ${listingId}`);
      const outcome = {
        item: row.item,
        price: row.buyoutPer * row.quantity,
        quantity: row.quantity,
      };
      purchases.set(idempotencyKey, outcome);
      return outcome;
    },
  },
];
