#!/usr/bin/env bun
/**
 * bin/check-web — run all web quality gates.
 * Typecheck → Biome check → Unit tests.
 */
import { runInherit } from './lib/cmd.ts';
import { log } from './lib/log.ts';

interface Step {
  name: string;
  cmd: string[];
}

const steps: Step[] = [
  { name: 'web: typecheck', cmd: ['bun', 'run', '--filter', '@omm/web', 'typecheck'] },
  { name: 'web: lint', cmd: ['bun', 'run', 'lint'] },
  { name: 'web: test', cmd: ['bun', 'run', '--filter', '@omm/web', 'test'] },
];

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
