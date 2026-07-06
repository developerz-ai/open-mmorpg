import { AuthError } from '../errors.ts';
import type { MockRequest, MockRoute } from './backend.ts';
import { pattern } from './backend.ts';

/**
 * Mock gateway auth. A tiny in-memory account store, seeded with one known
 * account so login E2E/tests are deterministic. Simulates the gateway's stable
 * typed error codes by throwing `AuthError` (never a leaked internal message).
 *
 * The mock ignores the bearer token, so the seeded account is the implicit
 * "current user" for the account-management endpoints — a deliberate
 * simplification while the gateway is "mock until live".
 */
interface Stored {
  id: string;
  displayName: string;
  email: string;
  password: string;
  createdAt: string;
}

/** The seeded account — deterministic login/E2E fixture (`password123`). */
function seedAccount(): Stored {
  return {
    id: 'acc_seed',
    displayName: 'Aria',
    email: 'aria@realm.test',
    password: 'password123',
    createdAt: '2026-01-01T00:00:00.000Z',
  };
}

const SEED_EMAIL = 'aria@realm.test';
const accounts = new Map<string, Stored>([[SEED_EMAIL, seedAccount()]]);
let seq = 0;

/**
 * Restore the seed account — test-only. Account-management endpoints mutate the
 * store; this keeps the suite deterministic regardless of test order.
 */
export function resetAuthMock(): void {
  accounts.clear();
  accounts.set(SEED_EMAIL, seedAccount());
  seq = 0;
}

/** The public account projection — the shape `AccountSchema` parses. */
function projection(a: Stored) {
  return { id: a.id, displayName: a.displayName, email: a.email, createdAt: a.createdAt };
}

function session(a: Stored) {
  return { token: `mock.${a.id}`, account: projection(a) };
}

/** The mock's notion of the logged-in account (the seed). Throws if absent. */
function currentAccount(): Stored {
  const seed = accounts.get(SEED_EMAIL);
  if (!seed) throw new AuthError('no session', 'invalid_credentials');
  return seed;
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
    resolve: () => projection(currentAccount()),
  },
  {
    backend: 'gateway',
    method: 'PUT',
    test: pattern('/account'),
    resolve: ({ body }: MockRequest) => {
      const b = (body ?? {}) as Partial<{ displayName: string; email: string }>;
      const acc = currentAccount();
      if (b.email !== undefined && b.email !== acc.email && accounts.has(b.email)) {
        throw new AuthError('email taken', 'email_taken');
      }
      if (typeof b.displayName === 'string' && b.displayName.trim().length > 0) {
        acc.displayName = b.displayName.trim();
      }
      if (b.email !== undefined && b.email !== acc.email) {
        accounts.delete(acc.email);
        acc.email = b.email;
        accounts.set(acc.email, acc);
      }
      return projection(acc);
    },
  },
  {
    backend: 'gateway',
    method: 'POST',
    test: pattern('/account/password'),
    resolve: ({ body }: MockRequest) => {
      const b = (body ?? {}) as Partial<{ currentPassword: string; newPassword: string }>;
      const acc = currentAccount();
      if (!b.currentPassword || b.currentPassword !== acc.password) {
        throw new AuthError('wrong current password', 'wrong_password');
      }
      if (!b.newPassword || b.newPassword.length < 8) {
        throw new AuthError('password too short', 'password_too_short');
      }
      acc.password = b.newPassword;
      return { ok: true as const };
    },
  },
];
