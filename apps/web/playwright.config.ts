import { defineConfig, devices } from '@playwright/test';

/**
 * E2E config. The app is served in **mock mode** (`VITE_USE_MOCKS=true`) so flows
 * run against the deterministic in-memory backend — no live gateway needed. The
 * dark-only theme + reduced-motion + a fixed timezone/viewport make renders and
 * screenshots deterministic (an AI agent can author and verify these).
 * → docs/specs/web-client/testing-dx
 */
const PORT = 4173;
const cashShopEnabled = process.env.CASH_SHOP_ENABLED === 'true' ? 'VITE_CASH_SHOP=true' : '';

export default defineConfig({
  testDir: './e2e',
  // `.e2e.ts` (not `.test`/`.spec`) so `bun test` never picks these up.
  testMatch: '**/*.e2e.ts',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'line' : 'list',
  use: {
    baseURL: `http://localhost:${PORT}`,
    timezoneId: 'UTC',
    viewport: { width: 1280, height: 800 },
    reducedMotion: 'reduce',
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: `VITE_USE_MOCKS=true ${cashShopEnabled} bun run dev --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
