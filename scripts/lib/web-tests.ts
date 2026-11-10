/**
 * Single source of truth for the web unit-test invocation, shared by
 * bin/test-web and bin/check-web.
 *
 * Runs from the repo root so the root bunfig.toml preload
 * (./scripts/test-setup.ts) applies. The `./` prefix makes bun treat each
 * argument as a path filter rather than a test-name filter.
 */
export const WEB_TEST_DIRS = ['./apps/web/', './packages/ui/', './packages/i18n/'] as const;

export const WEB_TEST_CMD: string[] = ['bun', 'test', ...WEB_TEST_DIRS];
