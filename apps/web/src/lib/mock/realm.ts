import type { MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock gateway realm status. Deterministic seed (fixed population) so screenshot
 * and E2E diffs stay stable — mirrors the gateway's real `RealmStatus` struct.
 */
export const realmRoutes: MockRoute[] = [
  {
    backend: 'gateway',
    method: 'GET',
    test: pattern('/realm/status'),
    resolve: () => ({
      name: 'open-mmorpg',
      online: true,
      population: 1204,
      capacity: 100_000,
    }),
  },
];
