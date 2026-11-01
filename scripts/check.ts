#!/usr/bin/env bun
/**
 * bin/check — the local gate. Mirrors the ci.yml rust/web jobs and
 * content-validation.yml. The ci.yml rust-integration job (live database) is
 * CI-only and deliberately not part of this gate.
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
log.dim('Local green is not evidence of a GitHub Actions run — CI is the source of truth.');
