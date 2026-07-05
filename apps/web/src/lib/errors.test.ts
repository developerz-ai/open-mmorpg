import { describe, expect, test } from 'bun:test';
import { ApiError, AuthError, errorKey, NetworkError, ValidationError } from './errors.ts';

describe('typed errors', () => {
  test('each kind maps to its t() key', () => {
    expect(new NetworkError('x').messageKey).toBe('errors.network');
    expect(new ValidationError('x').messageKey).toBe('errors.validation');
    expect(new AuthError('x').messageKey).toBe('errors.auth');
    expect(new ApiError('server', 'x').messageKey).toBe('errors.unknown');
  });

  test('errorKey falls back to unknown for non-ApiErrors', () => {
    expect(errorKey(new Error('boom'))).toBe('errors.unknown');
    expect(errorKey(new NetworkError('x'))).toBe('errors.network');
  });

  test('AuthError carries the gateway stable code', () => {
    expect(new AuthError('bad', 'invalid_credentials').code).toBe('invalid_credentials');
  });
});
