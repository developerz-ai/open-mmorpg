/**
 * In-memory mock backend — stands in for gateway/worldsvc until the servers are
 * live (docs/mvp/v1/50-web-client.md: "mock until live"). The API client routes
 * to it when `config.useMocks` is set. Each domain registers its routes; the
 * mock returns raw JSON-shaped values that still flow through the real Zod
 * boundary, so a mock that drifts fails the same way a real server would.
 */

/** A backend the client talks to. */
export type Backend = 'gateway' | 'worldsvc';

/** A parsed mock request. `params` come from the route's path pattern. */
export interface MockRequest {
  method: string;
  path: string;
  query: URLSearchParams;
  params: Record<string, string>;
  body: unknown;
}

/** A single mock route. `test` returns path params on match, else `null`. */
export interface MockRoute {
  backend: Backend;
  method: string;
  test(path: string): Record<string, string> | null;
  resolve(req: MockRequest): unknown;
}

const routes: MockRoute[] = [];

/** Register a domain's mock routes (called once at module load). */
export function registerRoutes(...next: MockRoute[]): void {
  routes.push(...next);
}

/** Thrown by the mock for an unroutable request (client maps to NetworkError). */
export class MockNotFound extends Error {}

/**
 * Build a path matcher from an Express-style pattern, e.g. `/armory/char/:name`.
 * Returns `test(path)` → params or null. Pure, no regex injection risk (pattern
 * is developer-authored, not user input).
 */
export function pattern(path: string): (p: string) => Record<string, string> | null {
  const keys: string[] = [];
  const re = new RegExp(
    `^${path.replace(/:[^/]+/g, (m) => {
      keys.push(m.slice(1));
      return '([^/]+)';
    })}$`,
  );
  return (p) => {
    const m = re.exec(p);
    if (!m) return null;
    const out: Record<string, string> = {};
    keys.forEach((k, i) => {
      out[k] = decodeURIComponent(m[i + 1] ?? '');
    });
    return out;
  };
}

/** Route a request through the registry. Throws `MockNotFound` if unmatched. */
export async function handleMock(
  backend: Backend,
  method: string,
  url: string,
  body: unknown,
): Promise<unknown> {
  const parsed = new URL(url, 'http://mock.local');
  const path = parsed.pathname;
  for (const route of routes) {
    if (route.backend !== backend || route.method !== method) continue;
    const params = route.test(path);
    if (!params) continue;
    return route.resolve({ method, path, query: parsed.searchParams, params, body });
  }
  throw new MockNotFound(`${method} ${backend}${path}`);
}
