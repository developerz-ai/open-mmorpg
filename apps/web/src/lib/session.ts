/**
 * The single secret the web holds: an opaque session token minted by the
 * gateway. We store it (and nothing else — no password, no hash, no account
 * truth) and attach it to authed requests. The gateway is the sole authority;
 * logout revokes server-side. → docs/specs/web-client/account-auth
 */
const STORAGE_KEY = 'omm.session';

let token: string | null = read();

function read(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(STORAGE_KEY);
}

/** The current session token, or `null` when logged out. */
export function sessionToken(): string | null {
  return token;
}

/** Persist a freshly-minted token (called only after a gateway login/register). */
export function setSessionToken(next: string): void {
  token = next;
  if (typeof localStorage !== 'undefined') localStorage.setItem(STORAGE_KEY, next);
}

/** Drop the token locally (pair with a server-side revoke on logout). */
export function clearSessionToken(): void {
  token = null;
  if (typeof localStorage !== 'undefined') localStorage.removeItem(STORAGE_KEY);
}
