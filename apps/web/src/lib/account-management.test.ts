import { afterAll, beforeEach, describe, expect, test } from 'bun:test';
import { changePassword, fetchAccount, login, register, updateProfile } from './auth.ts';
import { AuthError } from './errors.ts';
import { resetAuthMock } from './mock/auth.ts';

/**
 * Account-management flows against the mock gateway. The mock ignores the bearer
 * token, so the seeded account (`aria@realm.test` / `password123`) is the implicit
 * current user. `resetAuthMock` isolates each test — these endpoints mutate state.
 */
describe('account management (mock gateway)', () => {
  beforeEach(() => resetAuthMock());
  afterAll(() => resetAuthMock());

  test('updateProfile changes the display name and returns the projection', async () => {
    const account = await updateProfile({ displayName: 'Aria Renamed' });
    expect(account.displayName).toBe('Aria Renamed');
    expect((await fetchAccount()).displayName).toBe('Aria Renamed');
  });

  test('updateProfile rejects a taken email with email_taken', async () => {
    await register({ displayName: 'Nova', email: 'nova@realm.test', password: 'password123' });
    const promise = updateProfile({ email: 'nova@realm.test' });
    await expect(promise).rejects.toBeInstanceOf(AuthError);
    await promise.catch((err) => expect((err as AuthError).code).toBe('email_taken'));
  });

  test('updateProfile re-keys on email change and login uses the new email', async () => {
    await updateProfile({ email: 'aria2@realm.test' });
    const session = await login({ email: 'aria2@realm.test', password: 'password123' });
    expect(session.account.email).toBe('aria2@realm.test');
  });

  test('changePassword succeeds with the correct current password', async () => {
    await expect(
      changePassword({ currentPassword: 'password123', newPassword: 'newpass123' }),
    ).resolves.toBeUndefined();
    await expect(
      login({ email: 'aria@realm.test', password: 'password123' }),
    ).rejects.toBeInstanceOf(AuthError);
    const session = await login({ email: 'aria@realm.test', password: 'newpass123' });
    expect(session.account.displayName).toBe('Aria');
  });

  test('changePassword rejects a wrong current password with wrong_password', async () => {
    const promise = changePassword({ currentPassword: 'nope', newPassword: 'newpass123' });
    await expect(promise).rejects.toBeInstanceOf(AuthError);
    await promise.catch((err) => expect((err as AuthError).code).toBe('wrong_password'));
  });

  test('changePassword rejects a short new password with password_too_short', async () => {
    const promise = changePassword({ currentPassword: 'password123', newPassword: 'short' });
    await expect(promise).rejects.toBeInstanceOf(AuthError);
    await promise.catch((err) => expect((err as AuthError).code).toBe('password_too_short'));
  });
});
