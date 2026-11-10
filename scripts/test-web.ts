#!/usr/bin/env bun
/**
 * bin/test-web — run web unit tests via bun test.
 * One invocation covers apps/web, packages/ui and packages/i18n (see lib/web-tests.ts).
 */
import { runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';
import { WEB_TEST_CMD } from './lib/web-tests.ts';

log.step('web: unit tests (apps/web, packages/ui, packages/i18n)');

const code = runInherit(WEB_TEST_CMD);

if (code === 0) {
  log.ok('web: unit tests passed');
} else {
  log.fail(`web: unit tests failed (exit ${code})`);
  process.exit(1);
}
