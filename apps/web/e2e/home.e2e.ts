import { expect, test } from '@playwright/test';

test.describe('marketing home', () => {
  test('renders the brand, live realm status, and no missing i18n keys', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle('Open-MMORPG');
    await expect(page.getByRole('heading', { name: 'Open-MMORPG', level: 1 })).toBeVisible();

    // Realm status: Zod-validated mock projection, Intl-formatted population.
    await expect(page.getByText('Online', { exact: true })).toBeVisible();
    await expect(page.getByText('1,204 / 100,000 adventurers online')).toBeVisible();

    // A missing key would render ⟦…⟧ — assert none leaked into the DOM.
    await expect(page.locator('body')).not.toContainText('⟦');
  });

  test('nav exposes the feature-flagged surfaces', async ({ page }) => {
    await page.goto('/');
    for (const name of ['Armory', 'Auction house', 'World feed']) {
      await expect(page.getByRole('link', { name })).toBeVisible();
    }
  });
});
