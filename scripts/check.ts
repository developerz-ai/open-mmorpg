#!/usr/bin/env bun
/**
 * bin/check — the gate. Same commands locally, in the hook, and in CI.
 * Rust: fmt-check + clippy -D warnings + tests. Web: biome + tsc + bun test.
 *
 * Flags: --rust-only, --web-only.
 */
import { parseArgs } from './lib/args.ts';
import { have, runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';

interface Step {
  name: string;
  cmd: string[];
}

const { flags } = parseArgs(Bun.argv.slice(2));
const webOnly = flags['web-only'] === true;
const rustOnly = flags['rust-only'] === true;

const rustTest = have('cargo-nextest')
  ? ['cargo', 'nextest', 'run', '--workspace']
  : ['cargo', 'test', '--workspace', '--all-features'];

const rustSteps: Step[] = [
  { name: 'rust: fmt', cmd: ['cargo', 'fmt', '--all', '--check'] },
  {
    name: 'rust: clippy',
    cmd: ['cargo', 'clippy', '--all-targets', '--all-features', '--', '-D', 'warnings'],
  },
  { name: 'rust: test', cmd: rustTest },
];

const webSteps: Step[] = [
  { name: 'web: lint', cmd: ['bun', 'run', 'lint'] },
  { name: 'web: typecheck', cmd: ['bun', 'run', 'typecheck'] },
  { name: 'web: test', cmd: ['bun', 'test'] },
];

const steps = [...(webOnly ? [] : rustSteps), ...(rustOnly ? [] : webSteps)];

let failed = 0;
for (const step of steps) {
  log.step(step.name);
  const code = runInherit(step.cmd);
  if (code === 0) {
    log.ok(step.name);
  } else {
    log.fail(`${step.name} (exit ${code})`);
    failed++;
  }
}

if (failed > 0) {
  log.fail(`${failed}/${steps.length} checks failed`);
  process.exit(1);
}
log.ok(`all ${steps.length} checks passed`);
