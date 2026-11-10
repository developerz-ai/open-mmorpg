import { expect, test } from 'bun:test';
import { Glob } from 'bun';
import { fromRoot } from './paths.ts';
import { WEB_TEST_CMD, WEB_TEST_DIRS } from './web-tests.ts';

function testFilesIn(dir: string): string[] {
  const glob = new Glob('**/*.test.{ts,tsx}');
  return [...glob.scanSync({ cwd: fromRoot(dir) })].filter((f) => !f.includes('node_modules'));
}

test('WEB_TEST_CMD is one bun test invocation over all three web dirs', () => {
  expect(WEB_TEST_CMD).toEqual([
    'bun',
    'test',
    './apps/web/',
    './packages/ui/',
    './packages/i18n/',
  ]);
});

test.each([...WEB_TEST_DIRS])('%s contains at least one test file (no vacuous pass)', (dir) => {
  expect(testFilesIn(dir).length).toBeGreaterThan(0);
});

test.each(['check-web.ts', 'test-web.ts'])('%s uses shared WEB_TEST_CMD', async (script) => {
  const src = await Bun.file(fromRoot('scripts', script)).text();
  expect(src).toContain("from './lib/web-tests.ts'");
  expect(src).toContain('WEB_TEST_CMD');
  expect(src).not.toContain("'--filter', '@omm/web', 'test'");
});
