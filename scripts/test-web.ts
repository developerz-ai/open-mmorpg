#!/usr/bin/env bun
/**
 * bin/test-web — run web unit tests via bun test.
 * Filters to apps/web only, skipping e2e tests.
 */
import { runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';

log.step('web: unit tests');

const code = runInherit(['bun', 'run', '--filter', '@omm/web', 'test']);

if (code === 0) {
  log.ok('web: unit tests passed');
} else {
  log.fail(`web: unit tests failed (exit ${code})`);
  process.exit(1);
}
