import { expect, test } from 'bun:test';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fromRoot } from './paths.ts';

// Source-contract guard only: actual Linux Chromium runs remain separate CI proof.
// CI runs the BASE web job — four unconditional commands. The platform merge
// gate refuses added run: steps from platform PRs, so browser execution is not
// CI-enforced here; an authorized maintainer PR restores the e2e steps and
// re-extends this guard's job contract in the same change (recipe:
// docs/specs/web-client/testing-dx). This file pins that base job against
// silent mutation, plus the CI-ready Playwright wiring the restored steps rely on.
const EXISTING_WEB_CMDS = [
  'bun run lint',
  'bun run typecheck',
  'bun test',
  'bun run --filter @omm/web build',
];
const REQUIRED_CMDS = EXISTING_WEB_CMDS;

type Fields = Record<string, string>;
interface Step {
  fields: Fields;
  inputs: Fields;
}

/** Deliberately limited to this workflow's block-mapping layout, not general YAML. */
function jobLines(source: string): string[] {
  const lines = source.split('\n');
  const start = lines.indexOf('  web:');
  if (start === -1) return [];
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => /^ {0,2}\S/.test(line));
  return end === -1 ? rest : rest.slice(0, end);
}

function parseSteps(job: string[]): Step[] {
  const steps: Step[] = [];
  let step: Step | undefined;
  let inInputs = false;
  for (const line of job) {
    const first = /^ {6}- (.*)$/.exec(line);
    if (first) {
      step = { fields: {}, inputs: {} };
      steps.push(step);
      inInputs = false;
    }
    if (!step) continue;
    const nested = /^ {10}([\w.-]+):\s*(.*)$/.exec(line);
    if (inInputs && nested) {
      step.inputs[nested[1] ?? ''] = (nested[2] ?? '').trim();
      continue;
    }
    const entry = /^ {8}([\w-]+):\s*(.*)$/.exec(first ? `        ${first[1]}` : line);
    if (!entry) continue;
    const key = entry[1] ?? '';
    step.fields[key] = (entry[2] ?? '').trim();
    inInputs = key === 'with';
  }
  return steps;
}

function configValue(source: string, key: string): string | undefined {
  return new RegExp(`${key}:\\s*'([^']*)'`).exec(source)?.[1];
}

/** The SAME validator checks real wiring and every negative mutation below. */
function validateWiring(workflow: string, config: string): string[] {
  const errors: string[] = [];
  const check = (valid: boolean, message: string) => {
    if (!valid) errors.push(message);
  };
  const job = jobLines(workflow);
  const steps = parseSteps(job);
  const jobText = job.join('\n');
  const jobField = (key: string) => new RegExp(`^ {4}${key}:\\s*(.*)$`, 'm').exec(jobText)?.[1];
  const fatal = (value: string | undefined) => value === undefined || value === 'false';
  check(jobField('runs-on') === 'ubuntu-latest', 'Linux runner for committed Chromium baselines');
  check(jobField('if') === undefined, 'web job must not be conditional');
  check(fatal(jobField('continue-on-error')), 'web job must propagate failure');
  const minutes = Number(jobField('timeout-minutes'));
  check(minutes === 10, 'pinned web job timeout');
  const stepMinutes = steps.reduce(
    (sum, step) => sum + Number(step.fields['timeout-minutes'] ?? 0),
    0,
  );
  check(minutes > stepMinutes, 'job timeout must leave room for existing checks');

  let previous = -1;
  for (const cmd of REQUIRED_CMDS) {
    const matches = steps.filter((step) => step.fields.run === cmd);
    check(matches.length === 1, `exactly one unmodified run: ${cmd}`);
    const step = matches[0];
    if (!step) continue;
    const index = steps.indexOf(step);
    check(index > previous, `step order: ${cmd}`);
    previous = index;
    check(step.fields.if === undefined, `run must not be conditional: ${cmd}`);
    check(fatal(step.fields['continue-on-error']), `run must propagate failure: ${cmd}`);
    check(step.fields.shell === undefined, `run must use default fail-fast shell: ${cmd}`);
  }
  check(!workflow.includes('--update-snapshots'), 'CI must not update snapshots');
  check(
    config.includes("updateSnapshots: process.env.CI ? 'none' : undefined"),
    'CI snapshots read-only; local defaults unchanged',
  );
  check(/workers:\s*2\s*,/.test(config), 'at most two browser workers');
  check(configValue(config, 'screenshot') === 'only-on-failure', 'retain failure screenshots');
  check(configValue(config, 'trace') === 'on-first-retry', 'retain retry traces');
  const outputDir = configValue(config, 'outputDir');
  check(
    !/\boutputDir\s*:/.test(config) || ['test-results', './test-results'].includes(outputDir ?? ''),
    'Playwright output must match upload path',
  );
  check(configValue(config, 'testDir') === './e2e', 'stable spec directory');
  check(configValue(config, 'testMatch') === '**/*.e2e.ts', 'collect all e2e specs');
  return errors;
}

const workflow = readFileSync(fromRoot('.github/workflows/ci.yml'), 'utf8');
const config = readFileSync(fromRoot('apps/web/playwright.config.ts'), 'utf8');

test('step parsing includes first-line run, uses and name fields', () => {
  const steps = parseSteps([
    '      - run: bun test',
    '      - uses: actions/checkout@v4.4.0',
    '      - name: browser',
    '        run: playwright test',
  ]);
  expect(steps[0]?.fields.run).toBe('bun test');
  expect(steps[1]?.fields.uses).toBe('actions/checkout@v4.4.0');
  expect(steps[2]?.fields.name).toBe('browser');
});

test('real workflow and Playwright config satisfy the wiring contract', () => {
  expect(validateWiring(workflow, config)).toEqual([]);
});

function rejectWorkflow(label: string, mutated: string) {
  test(`wiring rejects ${label}`, () => {
    expect(mutated, 'mutation must apply').not.toBe(workflow);
    expect(validateWiring(mutated, config).length).toBeGreaterThan(0);
  });
}

for (const cmd of REQUIRED_CMDS) {
  const run = `        run: ${cmd}\n`;
  rejectWorkflow(`deleted ${cmd}`, workflow.replace(run, ''));
  for (const field of ['if: false', 'continue-on-error: true', 'shell: bash {0}']) {
    rejectWorkflow(`${field} on ${cmd}`, workflow.replace(run, `        ${field}\n${run}`));
  }
  for (const suffix of [' || true', '; exit 0', ' --update-snapshots', ' -u']) {
    rejectWorkflow(`${suffix} on ${cmd}`, workflow.replace(run, `        run: ${cmd}${suffix}\n`));
  }
}

for (const field of ['if: false', 'continue-on-error: true', 'continue-on-error: ${{ true }}']) {
  rejectWorkflow(`job ${field}`, workflow.replace('  web:\n', `  web:\n    ${field}\n`));
}
for (const value of ['', '0', 'Infinity', '60']) {
  rejectWorkflow(
    `job timeout ${value || 'deleted'}`,
    workflow.replace('    timeout-minutes: 10\n', value ? `    timeout-minutes: ${value}\n` : ''),
  );
}
for (const runner of ['windows-latest', 'macos-latest']) {
  const webJob = jobLines(workflow).join('\n');
  rejectWorkflow(
    `non-Linux web runner ${runner}`,
    workflow.replace(webJob, webJob.replace('ubuntu-latest', runner)),
  );
}
for (const [label, before, after] of [
  ['check order changed', 'bun run lint', 'bun run typecheck'],
] as const) {
  rejectWorkflow(label, workflow.replace(before, after));
}

for (const [label, before, after] of [
  ['snapshot updates enabled', "'none' : undefined", "'all' : undefined"],
  ['missing baselines regenerated', "'none' : undefined", "'missing' : undefined"],
  ['local snapshot default changed', "'none' : undefined", "'none' : 'none'"],
  ['workers uncapped', 'workers: 2,', ''],
  ['artifact output redirected', 'testDir:', "outputDir: 'elsewhere', testDir:"],
  ['screenshots disabled', "screenshot: 'only-on-failure'", "screenshot: 'off'"],
  ['traces disabled', "trace: 'on-first-retry'", "trace: 'off'"],
] as const) {
  test(`wiring rejects ${label}`, () => {
    const mutated = config.replace(before, after);
    expect(mutated, 'mutation must apply').not.toBe(config);
    expect(validateWiring(workflow, mutated).length).toBeGreaterThan(0);
  });
}

test('all committed specs remain discoverable under the pinned glob', () => {
  const dir = fromRoot('apps/web/e2e');
  const specs = readdirSync(dir, { recursive: true })
    .map(String)
    .filter((entry) => entry.endsWith('.e2e.ts') && statSync(join(dir, entry)).isFile());
  expect(specs.length).toBeGreaterThanOrEqual(9);
  expect(specs).toContain('shop-enabled.e2e.ts');
  expect(configValue(config, 'testMatch')).toBe('**/*.e2e.ts');
});

const expectedScripts = {
  'test:e2e': 'playwright test',
  'test:e2e:shop': 'CASH_SHOP_ENABLED=true playwright test e2e/shop-enabled.e2e.ts',
};
const { scripts }: { scripts: Record<string, unknown> } = JSON.parse(
  readFileSync(fromRoot('apps/web/package.json'), 'utf8'),
);
const validScripts = (candidate: Record<string, unknown>) =>
  Object.entries(expectedScripts).every(([key, command]) => candidate[key] === command);

test('package scripts invoke Playwright in default and shop-enabled modes', () => {
  expect(validScripts(scripts)).toBe(true);
});
for (const key of Object.keys(expectedScripts)) {
  for (const command of [
    undefined,
    'bun test',
    'playwright test || true',
    'playwright test -u',
    'playwright test e2e/shop-enabled.e2e.ts',
  ]) {
    test(`script mapping rejects ${key} = ${command}`, () => {
      expect(validScripts({ ...scripts, [key]: command })).toBe(false);
    });
  }
}

const visualNames = [
  'armory-character',
  'armory-not-found',
  'armory-search',
  'auction-browse',
  'auction-chart',
  'auction-filtered',
  'feed',
  'home',
].map((name) => `${name}.png`);
const visualSource = readFileSync(fromRoot('apps/web/e2e/visual.e2e.ts'), 'utf8');
const expectations = [...visualSource.matchAll(/\.toHaveScreenshot\('([^']+)'/g)]
  .map((match) => match[1])
  .sort();
const snapshotDir = fromRoot('apps/web/e2e/visual.e2e.ts-snapshots');
const baselines = readdirSync(snapshotDir).filter((name) =>
  statSync(join(snapshotDir, name)).isFile(),
);
const hasBaselines = (files: string[]) =>
  visualNames.every((name) => files.includes(name.replace('.png', '-chromium-linux.png')));

test('all eight named visual expectations have Chromium/Linux baseline files (not pixel proof)', () => {
  expect(expectations).toEqual(visualNames);
  expect(hasBaselines(baselines)).toBe(true);
});
for (const name of visualNames) {
  test(`baseline inventory rejects missing or wrong-platform ${name}`, () => {
    const filename = name.replace('.png', '-chromium-linux.png');
    expect(hasBaselines(baselines.filter((file) => file !== filename))).toBe(false);
    const wrongPlatform = baselines.map((file) =>
      file === filename ? file.replace('-linux', '-darwin') : file,
    );
    expect(hasBaselines(wrongPlatform)).toBe(false);
  });
}
