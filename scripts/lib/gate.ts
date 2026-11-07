/**
 * Gate step assembly and runner, extracted from `scripts/check.ts` so tests
 * can verify the lane composition and failure propagation without spawning
 * processes. Lane order: rust → content → web; the content lane runs only in
 * the full default gate (never under --rust-only or --web-only).
 */
import { log } from './log.ts';

/** One gate check: a display name plus the exact command to run. */
export interface Step {
  name: string;
  cmd: string[];
}

export interface GateOptions {
  /** --rust-only: emit the rust lane only. */
  rustOnly: boolean;
  /** --web-only: emit the web lane only. */
  webOnly: boolean;
  /** Whether `cargo-nextest` is on PATH (falls back to `cargo test`). */
  haveNextest: boolean;
}

/** Assemble the gate's steps for the given flag state. Pure — no I/O. */
export function buildSteps(opts: GateOptions): Step[] {
  const rustSteps: Step[] = [
    { name: 'rust: fmt', cmd: ['cargo', 'fmt', '--all', '--check'] },
    {
      name: 'rust: clippy',
      cmd: [
        'cargo',
        'clippy',
        '--locked',
        '--all-targets',
        '--all-features',
        '--',
        '-D',
        'warnings',
      ],
    },
    {
      name: 'rust: test',
      cmd: opts.haveNextest
        ? ['cargo', 'nextest', 'run', '--locked', '--workspace', '--all-features']
        : ['cargo', 'test', '--locked', '--workspace', '--all-features'],
    },
  ];

  const contentSteps: Step[] = [
    { name: 'content: validate', cmd: ['bun', 'scripts/validate-content.ts'] },
    {
      name: 'content: schema bench compile',
      cmd: ['cargo', 'bench', '--locked', '--package', 'omm-content-schema', '--no-run'],
    },
  ];

  const webSteps: Step[] = [
    { name: 'web: lint', cmd: ['bun', 'run', 'lint'] },
    { name: 'web: typecheck', cmd: ['bun', 'run', 'typecheck'] },
    { name: 'web: test', cmd: ['bun', 'test'] },
    { name: 'web: build', cmd: ['bun', 'run', '--filter', '@omm/web', 'build'] },
  ];

  return [
    ...(opts.webOnly ? [] : rustSteps),
    ...(opts.rustOnly || opts.webOnly ? [] : contentSteps),
    ...(opts.rustOnly ? [] : webSteps),
  ];
}

/** A runner executes one step's command and returns its exit code. */
export type Runner = (cmd: string[]) => number;

/**
 * Run every step in order, logging pass/fail per step. Never throws: a
 * non-zero exit is counted and the remaining steps still run.
 */
export function runSteps(steps: Step[], run: Runner): { failed: number; total: number } {
  let failed = 0;
  for (const step of steps) {
    log.step(step.name);
    const code = run(step.cmd);
    if (code === 0) {
      log.ok(step.name);
    } else {
      log.fail(`${step.name} (exit ${code})`);
      failed++;
    }
  }
  return { failed, total: steps.length };
}
