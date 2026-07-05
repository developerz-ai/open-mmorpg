import { NetworkError } from '../errors.ts';
import type { MockRequest, MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock worldsvc armory projections. A couple of seeded characters + one guild so
 * armory pages render deterministically. An unknown name is a not-found (thrown
 * as a transport miss; the page shows the not-found copy).
 */
const characters: Record<string, unknown> = {
  aria: {
    name: 'Aria',
    race: 'Sylvan',
    class: 'Warden',
    level: 60,
    itemLevel: 412,
    guild: 'Vanguard',
    achievements: 184,
    gear: [
      { slot: 'Head', item: 'Wardens Crown', itemLevel: 415 },
      { slot: 'Chest', item: 'Bark-Woven Vest', itemLevel: 410 },
      { slot: 'Weapon', item: 'Thornbite Glaive', itemLevel: 418 },
    ],
  },
  kael: {
    name: 'Kael',
    race: 'Emberkin',
    class: 'Pyromancer',
    level: 58,
    itemLevel: 388,
    guild: null,
    achievements: 92,
    gear: [{ slot: 'Weapon', item: 'Cinder Staff', itemLevel: 390 }],
  },
};

const guilds: Record<string, unknown> = {
  vanguard: {
    name: 'Vanguard',
    memberCount: 3,
    members: [
      { name: 'Aria', rank: 'Guildmaster', level: 60 },
      { name: 'Bryn', rank: 'Officer', level: 59 },
      { name: 'Cass', rank: 'Member', level: 55 },
    ],
  },
};

function lookup(store: Record<string, unknown>, name: string, kind: string): unknown {
  const hit = store[name.toLowerCase()];
  if (!hit) throw new NetworkError(`armory ${kind} ${name} not found`);
  return hit;
}

export const armoryRoutes: MockRoute[] = [
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/armory/character/:name'),
    resolve: ({ params }: MockRequest) => lookup(characters, params.name ?? '', 'character'),
  },
  {
    backend: 'worldsvc',
    method: 'GET',
    test: pattern('/armory/guild/:name'),
    resolve: ({ params }: MockRequest) => lookup(guilds, params.name ?? '', 'guild'),
  },
];
