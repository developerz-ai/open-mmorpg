import { expect, spyOn, test } from 'bun:test';
import { fromRoot } from './paths.ts';
import { installPlaywrightBrowsers } from './playwright-browsers.ts';

/** Run `fn` with console.error / console.warn captured, then restore them. */
function captureConsole<T>(fn: () => T): { value: T; errors: string[]; warnings: string[] } {
  const errors: string[] = [];
  const warnings: string[] = [];
  const errorSpy = spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
    errors.push(args.map(String).join(' '));
  });
  const warnSpy = spyOn(console, 'warn').mockImplementation((...args: unknown[]) => {
    warnings.push(args.map(String).join(' '));
  });
  try {
    return { value: fn(), errors, warnings };
  } finally {
    errorSpy.mockRestore();
    warnSpy.mockRestore();
  }
}

const INSTALL_CMD = ['bunx', 'playwright', 'install', 'chromium'];

test.each([1, 2, 127, 137, 255])(
  'a browser install exiting %i refuses: normalized exit code 1 and the E2E failure message',
  (exitCode) => {
    const ran: string[][] = [];
    const { value, errors, warnings } = captureConsole(() =>
      installPlaywrightBrowsers((cmd) => {
        ran.push(cmd);
        return exitCode;
      }),
    );

    expect(value).toBe(1);
    expect(ran).toEqual([INSTALL_CMD]);
    expect(errors.join('\n')).toContain('E2E tests cannot run');
    expect(errors.join('\n')).toContain('bunx playwright install --with-deps chromium');
    // The refusal is an error, not a warning: a warn-and-continue setup hides a
    // broken browser install until someone runs the e2e suite.
    expect(warnings).toEqual([]);
  },
);

test('a successful browser install returns 0 and logs no failure', () => {
  const ran: string[][] = [];
  const { value, errors, warnings } = captureConsole(() =>
    installPlaywrightBrowsers((cmd) => {
      ran.push(cmd);
      return 0;
    }),
  );

  expect(value).toBe(0);
  expect(ran).toEqual([INSTALL_CMD]);
  expect(errors).toEqual([]);
  expect(warnings).toEqual([]);
});

test('setup aborts on a failed browser install instead of warning and continuing', async () => {
  const src = await Bun.file(fromRoot('scripts', 'setup.ts')).text();
  const block = src.split('Installing Playwright browsers')[1] ?? '';

  expect(block).toContain('installPlaywrightBrowsers(runInherit)');
  expect(block).toMatch(/process\.exit\(browserInstall\)/);
  expect(block).not.toContain('log.warn');
  // The install command now lives in the helper, not inlined in the script.
  expect(src).not.toContain("'bunx', 'playwright', 'install'");
});
