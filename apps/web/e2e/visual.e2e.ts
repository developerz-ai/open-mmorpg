import { expect, test } from '@playwright/test';

/**
 * Visual regression tests for the Open-MMORPG operator portal.
 * Screenshots are deterministic due to:
 * - Dark-only theme (CSS variables)
 * - Fixed viewport (1280x800)
 * - Reduced motion disabled
 * - UTC timezone
 * → playwright.config.ts
 *
 * Run: `bun run test:e2e visual.e2e.ts`
 * Update snapshots: `bun playwright test --update-snapshots=changed visual.e2e.ts`
 */

test.describe('visual regression', () => {
  test('home page - brand, realm status, and navigation', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveScreenshot('home.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('armory - search interface', async ({ page }) => {
    await page.goto('/armory');
    // Wait for search interface to render
    await expect(page.getByLabel('Character or guild name')).toBeVisible();
    await expect(page).toHaveScreenshot('armory-search.png', {
      maxDiffPixels: 100,
    });
  });

  test('armory - character sheet projection', async ({ page }) => {
    await page.goto('/armory');
    await page.getByLabel('Character or guild name').fill('Aria');
    await page.getByRole('button', { name: 'Find character' }).click();

    // Wait for character sheet to load
    await expect(page.getByRole('heading', { name: 'Aria', level: 1 })).toBeVisible();
    await expect(page).toHaveScreenshot('armory-character.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('auction house - browse listings', async ({ page }) => {
    await page.goto('/auction');
    // Wait for listings to render
    await expect(page.getByRole('cell', { name: 'Thornbite Glaive' })).toBeVisible();
    await expect(page).toHaveScreenshot('auction-browse.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('auction house - filtered search', async ({ page }) => {
    await page.goto('/auction');
    await page.getByLabel('Search').fill('staff');
    // Wait for filter to apply
    await expect(page.getByRole('cell', { name: 'Cinder Staff' })).toBeVisible();
    await expect(page).toHaveScreenshot('auction-filtered.png', {
      maxDiffPixels: 100,
    });
  });

  test('auction house - price history chart', async ({ page }) => {
    await page.goto('/auction');
    // Wait for listings to render
    await expect(page.getByRole('cell', { name: 'Thornbite Glaive' })).toBeVisible();
    // Click an item to select it and show price history
    await page.getByRole('cell', { name: 'Thornbite Glaive' }).click();
    // Wait for price chart card to appear with the chart
    await expect(page.getByText('Price history (avg buyout) — Thornbite Glaive')).toBeVisible();
    await expect(page).toHaveScreenshot('auction-chart.png', {
      maxDiffPixels: 100,
    });
  });

  test('world feed - living-world events stream', async ({ page }) => {
    await page.goto('/feed');
    // Wait for feed items to render
    await expect(page.getByText('Vanguard felled Emberdrake.')).toBeVisible();
    await expect(page).toHaveScreenshot('feed.png', {
      fullPage: true,
      maxDiffPixels: 100,
    });
  });

  test('armory - not found state', async ({ page }) => {
    await page.goto('/armory/character/Nobody');
    await expect(page.getByText('No such character or guild on this realm.')).toBeVisible();
    await expect(page).toHaveScreenshot('armory-not-found.png', {
      maxDiffPixels: 100,
    });
  });
});
