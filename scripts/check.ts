#!/usr/bin/env bun
/**
 * bin/check — the local gate. Mirrors the ci.yml rust/web jobs and
 * content-validation.yml; the web job is the base four (lint, typecheck,
 * bun test, build). The ci.yml rust-integration job (live database) is
 * CI-only and deliberately not part of this gate.
 *
 * E2E is CI-ready but NOT CI-enforced: the web job has no Playwright steps
 * (a platform PR cannot add run: steps). Run the browser suites locally —
 * bin/test-e2e (default mode) plus bun run --filter @omm/web test:e2e:shop
 * (cash-shop enabled); bin/setup installs Chromium. CI enforcement awaits
 * an authorized workflow change.
 *
 * Local green is NOT evidence of a GitHub Actions run — CI is the source of
 * truth for merge status.
 *
 * Flags: --rust-only, --web-only.
 */
import { parseArgs } from './lib/args.ts';
import { have, runInherit } from './lib/cmd.ts';
import { buildSteps, runSteps } from './lib/gate.ts';
import { log } from './lib/log.ts';

const { flags } = parseArgs(Bun.argv.slice(2));
const webOnly = flags['web-only'] === true;
const rustOnly = flags['rust-only'] === true;

const steps = buildSteps({ rustOnly, webOnly, haveNextest: have('cargo-nextest') });
const { failed, total } = runSteps(steps, runInherit);

if (failed > 0) {
  log.fail(`${failed}/${total} checks failed — local gate only, not a GitHub Actions run`);
  process.exit(1);
}
log.ok(`all ${total} checks passed`);
log.dim('Local green excludes browsers and is not GitHub CI proof — CI is the source of truth.');
