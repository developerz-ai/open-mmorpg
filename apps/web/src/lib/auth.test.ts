import { describe, expect, test } from 'bun:test';
import { authMessageKey, fetchAccount, login, logout, register } from './auth.ts';
import { AuthError } from './errors.ts';
import { sessionToken } from './session.ts';

describe('auth (mock gateway)', () => {
  test('login with the seeded account returns a session and stores the token', async () => {
    const session = await login({ email: 'aria@realm.test', password: 'password123' });
    expect(session.account.displayName).toBe('Aria');
    expect(sessionToken()).toBe(session.token);
  });

  test('login with bad credentials throws a typed AuthError with a stable code', async () => {
    const promise = login({ email: 'aria@realm.test', password: 'wrong' });
    await expect(promise).rejects.toBeInstanceOf(AuthError);
    await promise.catch((err) => expect((err as AuthError).code).toBe('invalid_credentials'));
  });

  test('register creates an account and a session', async () => {
    const session = await register({
      displayName: 'Nova',
      email: 'nova@realm.test',
      password: 'password123',
    });
    expect(session.account.email).toBe('nova@realm.test');
  });

  test('registering a taken email fails with email_taken', async () => {
    const promise = register({
      displayName: 'Dup',
      email: 'aria@realm.test',
      password: 'password123',
    });
    await promise.catch((err) => expect((err as AuthError).code).toBe('email_taken'));
    await expect(promise).rejects.toBeInstanceOf(AuthError);
  });

  test('fetchAccount returns the account projection', async () => {
    const account = await fetchAccount();
    expect(account.email).toContain('@');
  });

  test('logout clears the local token', async () => {
    await login({ email: 'aria@realm.test', password: 'password123' });
    await logout();
    expect(sessionToken()).toBeNull();
  });

  test('authMessageKey maps a coded AuthError to its catalog key', () => {
    expect(authMessageKey(new AuthError('x', 'rate_limited'))).toBe('auth.error.rate_limited');
    expect(authMessageKey(new Error('x'))).toBe('errors.unknown');
  });
});
