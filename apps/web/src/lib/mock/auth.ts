import { AuthError } from '../errors.ts';
import type { MockRequest, MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock gateway auth. A tiny in-memory account store, seeded with one known
 * account so login E2E/tests are deterministic. Simulates the gateway's stable
 * typed error codes by throwing `AuthError` (never a leaked internal message).
 */
interface Stored {
  id: string;
  displayName: string;
  email: string;
  password: string;
  createdAt: string;
}

const accounts = new Map<string, Stored>([
  [
    'aria@realm.test',
    {
      id: 'acc_seed',
      displayName: 'Aria',
      email: 'aria@realm.test',
      password: 'password123',
      createdAt: '2026-01-01T00:00:00.000Z',
    },
  ],
]);

let seq = 0;

function session(a: Stored) {
  return {
    token: `mock.${a.id}`,
    account: { id: a.id, displayName: a.displayName, email: a.email, createdAt: a.createdAt },
  };
}

function asInput(body: unknown): { displayName: string; email: string; password: string } {
  const b = (body ?? {}) as Partial<Record<'displayName' | 'email' | 'password', string>>;
  return {
    displayName: b.displayName ?? 'Adventurer',
    email: b.email ?? '',
    password: b.password ?? '',
  };
}

export const authRoutes: MockRoute[] = [
  {
    backend: 'gateway',
    method: 'POST',
    test: pattern('/auth/register'),
    resolve: ({ body }: MockRequest) => {
      const input = asInput(body);
      if (accounts.has(input.email)) throw new AuthError('email taken', 'email_taken');
      seq += 1;
      const stored: Stored = {
        id: `acc_${seq}`,
        displayName: input.displayName,
        email: input.email,
        password: input.password,
        createdAt: '2026-07-05T12:00:00.000Z',
      };
      accounts.set(stored.email, stored);
      return session(stored);
    },
  },
  {
    backend: 'gateway',
    method: 'POST',
    test: pattern('/auth/login'),
    resolve: ({ body }: MockRequest) => {
      const input = asInput(body);
      const found = accounts.get(input.email);
      if (!found || found.password !== input.password) {
        throw new AuthError('bad credentials', 'invalid_credentials');
      }
      return session(found);
    },
  },
  {
    backend: 'gateway',
    method: 'POST',
    test: pattern('/auth/logout'),
    resolve: () => ({ ok: true }),
  },
  {
    backend: 'gateway',
    method: 'GET',
    test: pattern('/account'),
    resolve: () => {
      const seed = accounts.get('aria@realm.test');
      if (!seed) throw new AuthError('no session', 'invalid_credentials');
      return {
        id: seed.id,
        displayName: seed.displayName,
        email: seed.email,
        createdAt: seed.createdAt,
      };
    },
  },
];
