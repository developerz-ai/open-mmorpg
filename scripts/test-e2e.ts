#!/usr/bin/env bun
/**
 * bin/test-e2e — run web E2E tests via Playwright.
 * Filters to apps/web only.
 */
import { runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';

log.step('web: e2e tests');

const code = runInherit(['bun', 'run', '--filter', '@omm/web', 'test:e2e']);

if (code === 0) {
  log.ok('web: e2e tests passed');
} else {
  log.fail(`web: e2e tests failed (exit ${code})`);
  process.exit(1);
}
