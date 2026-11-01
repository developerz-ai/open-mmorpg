import { expect, test } from 'bun:test';
import { buildSteps, runSteps, type Step } from './gate.ts';

const FULL = { rustOnly: false, webOnly: false, haveNextest: true };

test('rust test step pins nextest with --all-features (was missing it)', () => {
  const steps = buildSteps(FULL);
  expect(steps.find((s) => s.name === 'rust: test')?.cmd).toEqual([
    'cargo',
    'nextest',
    'run',
    '--workspace',
    '--all-features',
  ]);
});

test('rust test fallback keeps cargo test with --all-features', () => {
  const steps = buildSteps({ ...FULL, haveNextest: false });
  expect(steps.find((s) => s.name === 'rust: test')?.cmd).toEqual([
    'cargo',
    'test',
    '--workspace',
    '--all-features',
  ]);
});

test('rust clippy and fmt steps match the ci.yml rust job', () => {
  const steps = buildSteps(FULL);
  expect(steps.find((s) => s.name === 'rust: clippy')?.cmd).toEqual([
    'cargo',
    'clippy',
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
