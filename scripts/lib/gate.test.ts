import { expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { buildSteps, runSteps, type Step } from './gate.ts';
import { fromRoot } from './paths.ts';

const FULL = { rustOnly: false, webOnly: false, haveNextest: true };

test('rust test step pins nextest with --all-features (was missing it)', () => {
  const steps = buildSteps(FULL);
  expect(steps.find((s) => s.name === 'rust: test')?.cmd).toEqual([
    'cargo',
    'nextest',
    'run',
    '--locked',
    '--workspace',
    '--all-features',
  ]);
});

test('rust test fallback keeps cargo test with --all-features', () => {
  const steps = buildSteps({ ...FULL, haveNextest: false });
  expect(steps.find((s) => s.name === 'rust: test')?.cmd).toEqual([
    'cargo',
    'test',
    '--locked',
    '--workspace',
    '--all-features',
  ]);
});

test('rust clippy and fmt steps match the ci.yml rust job', () => {
  const steps = buildSteps(FULL);
  expect(steps.find((s) => s.name === 'rust: clippy')?.cmd).toEqual([
    'cargo',
    'clippy',
    '--locked',
    '--all-targets',
    '--all-features',
    '--',
    '-D',
    'warnings',
  ]);
  expect(steps.find((s) => s.name === 'rust: fmt')?.cmd).toEqual([
    'cargo',
    'fmt',
    '--all',
    '--check',
  ]);
});

test('default gate includes the content lane and web build', () => {
  const steps = buildSteps(FULL);
  expect(steps.find((s) => s.name === 'content: validate')?.cmd).toEqual([
    'bun',
    'scripts/validate-content.ts',
  ]);
  expect(steps.find((s) => s.name === 'content: schema bench compile')?.cmd).toEqual([
    'cargo',
    'bench',
    '--locked',
    '--package',
    'omm-content-schema',
    '--no-run',
  ]);
  expect(steps.find((s) => s.name === 'web: build')?.cmd).toEqual([
    'bun',
    'run',
    '--filter',
    '@omm/web',
    'build',
  ]);
});

test('--rust-only emits the rust lane only (content and web dropped)', () => {
  const names = buildSteps({ ...FULL, rustOnly: true }).map((s) => s.name);
  expect(names).toEqual(['rust: fmt', 'rust: clippy', 'rust: test']);
});

test('--web-only emits the web lane only (content lane never runs solo)', () => {
  const names = buildSteps({ ...FULL, webOnly: true }).map((s) => s.name);
  expect(names).toEqual(['web: lint', 'web: typecheck', 'web: test', 'web: build']);
});

test('default gate runs rust, content, then web in lane order', () => {
  const names = buildSteps(FULL).map((s) => s.name);
  expect(names).toEqual([
    'rust: fmt',
    'rust: clippy',
    'rust: test',
    'content: validate',
    'content: schema bench compile',
    'web: lint',
    'web: typecheck',
    'web: test',
    'web: build',
  ]);
});

/**
 * Cargo subcommands that resolve the dependency graph. Each of these must be
 * invoked with `--locked` so a manifest/lockfile drift fails the check instead
 * of silently rewriting Cargo.lock.
 *
 * `fmt` is deliberately excluded: it does not resolve dependencies and cargo
 * rejects `--locked` for it.
 */
const DEPENDENCY_RESOLVING_CARGO = [
  'build',
  'bench',
  'check',
  'clippy',
  'doc',
  'fetch',
  'install',
  'nextest',
  'run',
  'test',
  'tree',
  'update',
] as const;

/**
 * Extract every `cargo <subcommand> ...` invocation from a CI file's text.
 * Handles both YAML `run:` lines and Dockerfile `RUN` lines. Returns the
 * argument list of each invocation found.
 */
function cargoInvocations(source: string): Array<[string, ...string[]]> {
  const invocations: Array<[string, ...string[]]> = [];
  const lines = source.split('\n');
  for (const line of lines) {
    const match = /\bcargo\s+([a-z-]+)((?:\s+[^\s|&;]+)*)/.exec(line);
    if (!match) continue;
    const sub = match[1];
    if (!sub) continue;
    const argText = match[2]?.trim() ?? '';
    const args = argText ? argText.split(/\s+/) : [];
    invocations.push([sub, ...args]);
  }
  return invocations;
}

test('CI cargo commands are locked against the committed Cargo.lock', () => {
  const files = [
    '.github/workflows/ci.yml',
    '.github/workflows/content-validation.yml',
    'docker/rust.Dockerfile',
  ];

  for (const file of files) {
    const source = readFileSync(fromRoot(file), 'utf8');
    const invocations = cargoInvocations(source);
    const resolving = invocations.filter(([sub]) =>
      (DEPENDENCY_RESOLVING_CARGO as readonly string[]).includes(sub),
    );

    // Guard against a vacuous pass: each file must actually contain at least
    // one dependency-resolving cargo command for this assertion to mean
    // anything.
    expect(resolving.length, `${file} has no cargo commands to check`).toBeGreaterThan(0);

    for (const invocation of resolving) {
      expect(invocation, `${file}: cargo ${invocation.join(' ')}`).toContain('--locked');
    }
  }
});

test('cargo fmt invocations never carry --locked (cargo rejects the flag)', () => {
  const files = [
    '.github/workflows/ci.yml',
    '.github/workflows/content-validation.yml',
    'docker/rust.Dockerfile',
  ];

  for (const file of files) {
    const source = readFileSync(fromRoot(file), 'utf8');
    for (const invocation of cargoInvocations(source)) {
      if (invocation[0] !== 'fmt') continue;
      expect(invocation, `${file}: cargo ${invocation.join(' ')}`).not.toContain('--locked');
    }
  }
});

test('runSteps counts a failing step and keeps running the rest', () => {
  const ran: string[] = [];
  const steps: Step[] = [
    { name: 'pass', cmd: ['true'] },
    { name: 'boom', cmd: ['false'] },
    { name: 'after', cmd: ['true'] },
  ];
  const { failed, total } = runSteps(steps, (cmd) => {
    ran.push(cmd[0] ?? '');
    return cmd[0] === 'false' ? 2 : 0;
  });
  expect(failed).toBe(1);
  expect(total).toBe(3);
  expect(ran).toEqual(['true', 'false', 'true']);
});
