/**
 * The API client — the one boundary where network shapes enter the app. Every
 * response is Zod-parsed here (validate, then the inferred type flows into the
 * UI); every failure becomes a typed error. When `config.useMocks` is set it
 * routes to the in-memory mock backend, still through the same Zod parse — so a
 * mock and a real server are indistinguishable to callers.
 * → docs/specs/web-client/data-layer
 */
import type { z } from 'zod';
import { config } from '../config.ts';
import { AuthError, NetworkError, ValidationError } from './errors.ts';
import type { Backend } from './mock/backend.ts';
import { handleMock, MockNotFound } from './mock/index.ts';
import { sessionToken } from './session.ts';

export interface RequestOptions<T> {
  backend: Backend;
  path: string;
  schema: z.ZodType<T>;
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  body?: unknown;
  /** Attach the session token (authed endpoints). */
  auth?: boolean;
}

function baseUrl(backend: Backend): string {
  return backend === 'gateway' ? config.endpoints.gatewayUrl : config.endpoints.worldsvcUrl;
}

/** Parse an unknown payload against the schema, mapping failure to `ValidationError`. */
function validate<T>(schema: z.ZodType<T>, payload: unknown): T {
  const result = schema.safeParse(payload);
  if (!result.success) throw new ValidationError(result.error.message);
  return result.data;
}

/** Perform a validated request. Returns the parsed body or throws a typed error. */
export async function request<T>(opts: RequestOptions<T>): Promise<T> {
  const method = opts.method ?? 'GET';

  if (config.useMocks) {
    try {
      const payload = await handleMock(opts.backend, method, opts.path, opts.body);
      return validate(opts.schema, payload);
    } catch (err) {
      if (err instanceof MockNotFound) throw new NetworkError(err.message);
      throw err;
    }
  }

  const headers: Record<string, string> = {};
  if (opts.body !== undefined) headers['content-type'] = 'application/json';
  if (opts.auth) {
    const token = sessionToken();
    if (token) headers.authorization = `Bearer ${token}`;
  }

  let res: Response;
  try {
    res = await fetch(`${baseUrl(opts.backend)}${opts.path}`, {
      method,
      headers,
      body: opts.body === undefined ? undefined : JSON.stringify(opts.body),
    });
  } catch (err) {
    throw new NetworkError(err instanceof Error ? err.message : 'fetch failed');
  }

  if (res.status === 401 || res.status === 403) {
    throw new AuthError(`auth ${res.status}`, await readCode(res));
  }
  if (!res.ok) throw new NetworkError(`${opts.backend} ${opts.path} → ${res.status}`);

  let json: unknown;
  try {
    json = await res.json();
  } catch {
    throw new ValidationError(`${opts.path} returned non-JSON`);
  }
  return validate(opts.schema, json);
}

/** Best-effort extraction of the gateway's stable `code` from an auth failure. */
async function readCode(res: Response): Promise<string | undefined> {
  try {
    const body = (await res.json()) as { code?: unknown };
    return typeof body.code === 'string' ? body.code : undefined;
  } catch {
    return undefined;
  }
}
