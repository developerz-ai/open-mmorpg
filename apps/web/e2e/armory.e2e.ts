import { expect, test } from '@playwright/test';

test.describe('armory', () => {
  test('search routes to a character sheet projection', async ({ page }) => {
    await page.goto('/armory');
    await page.getByLabel('Character or guild name').fill('Aria');
    await page.getByRole('button', { name: 'Find character' }).click();

    await expect(page).toHaveURL(/\/armory\/character\/Aria$/);
    await expect(page.getByRole('heading', { name: 'Aria', level: 1 })).toBeVisible();
    await expect(page.getByText('Level 60 Sylvan Warden')).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Thornbite Glaive' })).toBeVisible();
  });

  test('an unknown character shows the not-found copy, not a crash', async ({ page }) => {
    await page.goto('/armory/character/Nobody');
    await expect(page.getByText('No such character or guild on this realm.')).toBeVisible();
  });
});
