import { expect, test } from '@playwright/test';

test.describe('auction house', () => {
  test('browse, filter, and buy an item (intent → confirmed outcome)', async ({ page }) => {
    await page.goto('/auction');
    await expect(page.getByRole('cell', { name: 'Thornbite Glaive' })).toBeVisible();

    // Filter narrows to the matching listing only.
    await page.getByLabel('Search').fill('staff');
    await expect(page.getByRole('cell', { name: 'Cinder Staff' })).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Thornbite Glaive' })).toHaveCount(0);

    // Buy shows the server-confirmed outcome, never optimistic.
    await page.getByLabel('Search').fill('');
    await page.getByRole('button', { name: 'Buy' }).first().click();
    await expect(page.getByText('Purchased Thornbite Glaive for 42,000 gold.')).toBeVisible();

    // Price history chart appears for the selected item.
    await expect(page.getByRole('img', { name: 'Price history (avg buyout)' })).toBeVisible();
  });
});
