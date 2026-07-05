import { z } from 'zod';
import { request } from './api.ts';
import { AuthError, errorKey } from './errors.ts';
import { clearSessionToken, setSessionToken } from './session.ts';

/**
 * Account & auth boundary. **Server-authoritative:** the gateway owns passwords,
 * sessions, and 2FA; the web submits intents (register/login) and stores only
 * the opaque session token. Every response is Zod-parsed; a bad shape is a typed
 * error, not a render. → docs/specs/web-client/account-auth
 */
export const AccountSchema = z.object({
  id: z.string(),
  displayName: z.string(),
  email: z.string().email(),
  createdAt: z.string(), // ISO-8601; formatted for display via `Intl`
});
export type Account = z.infer<typeof AccountSchema>;

export const SessionSchema = z.object({ token: z.string(), account: AccountSchema });
export type Session = z.infer<typeof SessionSchema>;

export interface RegisterInput {
  displayName: string;
  email: string;
  password: string;
}
export interface LoginInput {
  email: string;
  password: string;
}

/** Register, persisting the returned session token. */
export async function register(input: RegisterInput): Promise<Session> {
  const session = await request({
    backend: 'gateway',
    path: '/auth/register',
    method: 'POST',
    body: input,
    schema: SessionSchema,
  });
  setSessionToken(session.token);
  return session;
}

/** Log in, persisting the returned session token. */
export async function login(input: LoginInput): Promise<Session> {
  const session = await request({
    backend: 'gateway',
    path: '/auth/login',
    method: 'POST',
    body: input,
    schema: SessionSchema,
  });
  setSessionToken(session.token);
  return session;
}

/** Fetch the current account projection (session-bearing). */
export function fetchAccount(): Promise<Account> {
  return request({ backend: 'gateway', path: '/account', schema: AccountSchema, auth: true });
}

/**
 * The `t()` key for an auth failure — a stable gateway code (`invalid_credentials`)
 * maps to `auth.error.<code>`; anything else falls back to the generic kinds.
 */
export function authMessageKey(err: unknown): string {
  if (err instanceof AuthError && err.code) return `auth.error.${err.code}`;
  return errorKey(err);
}

/** Revoke server-side, then drop the local token. */
export async function logout(): Promise<void> {
  await request({
    backend: 'gateway',
    path: '/auth/logout',
    method: 'POST',
    schema: z.object({ ok: z.literal(true) }),
    auth: true,
  });
  clearSessionToken();
}
