/**
 * Playwright browser install, extracted from `scripts/setup.ts` so the
 * refusal path is unit-testable without downloading a browser.
 *
 * `bunx playwright` resolves the `playwright` bin from the root devDependency
 * (@playwright/test). The local install is browser-only — it does not touch
 * system packages; CI installs the browser together with its OS libraries
 * (`bunx playwright install --with-deps chromium`). A failed download must not
 * stay a warning: e2e is a gated suite, so a broken browser install refuses
 * here instead of hiding until a developer hits "browser not found".
 */
import { log } from './log.ts';

/** A runner executes one command and returns its exit code. */
export type CommandRunner = (cmd: string[]) => number;

/** The local install: the Chromium build the e2e suite drives, no OS packages. */
export const PLAYWRIGHT_INSTALL_CMD: string[] = ['bunx', 'playwright', 'install', 'chromium'];

/** The refusal shown when the install fails, and how to retry it by hand. */
export const PLAYWRIGHT_INSTALL_FAILURE =
  'Playwright browser install failed — E2E tests cannot run.\n' +
  '  Retry: bunx playwright install --with-deps chromium';

/**
 * Install the Chromium build the e2e suite drives. Returns 0 on success and
 * 1 when the install failed — the caller must abort on a non-zero code.
 */
export function installPlaywrightBrowsers(run: CommandRunner): number {
  const code = run(PLAYWRIGHT_INSTALL_CMD);
  if (code !== 0) {
    log.fail(PLAYWRIGHT_INSTALL_FAILURE);
    return 1;
  }
  return 0;
}
