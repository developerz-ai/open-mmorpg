/**
 * Typed errors, not bare throws. Distinguishing failure kinds lets a component
 * render the right `t()` message: a validation failure is *contract drift* (a
 * bug), not the same as being offline. Never carries a stack, SQL, or internal
 * message from the server — the gateway maps internals to stable codes.
 * → docs/specs/web-client/data-layer
 */

/** The kind of API failure, mapped to a `t('errors.*')` message. */
export type ApiErrorKind = 'network' | 'validation' | 'auth' | 'server' | 'unknown';

/** Base class for every failure the API client surfaces. */
export class ApiError extends Error {
  readonly kind: ApiErrorKind;
  constructor(kind: ApiErrorKind, message: string) {
    super(message);
    this.name = 'ApiError';
    this.kind = kind;
  }
  /** The `t()` key for this failure's user-facing copy. */
  get messageKey(): string {
    return `errors.${this.kind === 'server' ? 'unknown' : this.kind}`;
  }
}

/** Transport failure — offline, DNS, CORS, 5xx. */
export class NetworkError extends ApiError {
  constructor(message: string) {
    super('network', message);
    this.name = 'NetworkError';
  }
}

/** Zod parse failure at the boundary — the server's shape drifted from ours. */
export class ValidationError extends ApiError {
  constructor(message: string) {
    super('validation', message);
    this.name = 'ValidationError';
  }
}

/** 401/403 — the session is missing or expired. */
export class AuthError extends ApiError {
  /** Stable client code from the gateway, e.g. `invalid_credentials`. */
  readonly code?: string;
  constructor(message: string, code?: string) {
    super('auth', message);
    this.name = 'AuthError';
    this.code = code;
  }
}

/** Map any thrown value to its `t('errors.*')` key. */
export function errorKey(err: unknown): string {
  return err instanceof ApiError ? err.messageKey : 'errors.unknown';
}
