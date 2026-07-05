import { describe, expect, test } from 'bun:test';
import { z } from 'zod';
import { request } from './api.ts';
import { NetworkError, ValidationError } from './errors.ts';

describe('api client (mock mode)', () => {
  test('routes to the mock backend and Zod-parses the response', async () => {
    const status = await request({
      backend: 'gateway',
      path: '/realm/status',
      schema: z.object({ name: z.string(), online: z.boolean() }),
    });
    expect(status.online).toBe(true);
  });

  test('a schema mismatch surfaces as a ValidationError (contract drift)', async () => {
    const promise = request({
      backend: 'gateway',
      path: '/realm/status',
      schema: z.object({ notAField: z.string() }),
    });
    await expect(promise).rejects.toBeInstanceOf(ValidationError);
  });

  test('an unroutable path surfaces as a NetworkError', async () => {
    const promise = request({
      backend: 'gateway',
      path: '/does/not/exist',
      schema: z.object({}),
    });
    await expect(promise).rejects.toBeInstanceOf(NetworkError);
  });
});
