import { expect, test } from 'bun:test';
import { RealmStatusSchema } from './realm.ts';

test('RealmStatusSchema accepts a valid payload', () => {
  const parsed = RealmStatusSchema.parse({
    name: 'open-mmorpg',
    online: true,
    population: 0,
    capacity: 100_000,
  });
  expect(parsed.capacity).toBe(100_000);
});

test('RealmStatusSchema rejects a bad payload at the boundary', () => {
  expect(() =>
    RealmStatusSchema.parse({ name: 'x', online: 'yes', population: -1, capacity: 0 }),
  ).toThrow();
});
