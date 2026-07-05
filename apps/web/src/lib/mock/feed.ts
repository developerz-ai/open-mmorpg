import type { MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock worldsvc world feed. A deterministic seed covering each event class, plus
 * one deliberately-unknown variant to exercise graceful degradation. Fixed
 * timestamps keep the rendered relative times stable under a frozen clock.
 */
const feed: unknown[] = [
  {
    id: 'e1',
    kind: 'boss_kill',
    actor: 'Vanguard',
    target: 'Emberdrake',
    at: '2026-07-05T11:30:00.000Z',
  },
  {
    id: 'e2',
    kind: 'world_boss_spawn',
    target: 'The Hollow King',
    zone: 'Ashfen',
    at: '2026-07-05T11:00:00.000Z',
  },
  {
    id: 'e3',
    kind: 'faction_shift',
    actor: 'The Covenant',
    zone: 'Ironreach',
    at: '2026-07-05T10:15:00.000Z',
  },
  { id: 'e4', kind: 'bounty_posted', target: 'Kael', at: '2026-07-05T09:45:00.000Z' },
  {
    id: 'e5',
    kind: 'milestone',
    message: 'The realm reached 100,000 souls.',
    at: '2026-07-05T08:00:00.000Z',
  },
  // Unknown-to-us variant (e.g. a newer server) — must degrade, not crash.
  { id: 'e6', kind: 'meteor_shower', intensity: 9, at: '2026-07-05T07:30:00.000Z' },
];

export const feedRoutes: MockRoute[] = [
  { backend: 'worldsvc', method: 'GET', test: pattern('/world/feed'), resolve: () => feed },
];
