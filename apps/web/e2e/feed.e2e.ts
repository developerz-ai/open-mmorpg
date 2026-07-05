import { expect, test } from '@playwright/test';

test.describe('world feed', () => {
  test('renders living-world events and degrades unknown variants gracefully', async ({ page }) => {
    await page.goto('/feed');
    await expect(page.getByText('Vanguard felled Emberdrake.')).toBeVisible();
    await expect(page.getByText('The Hollow King has awoken in Ashfen.')).toBeVisible();
    await expect(page.getByText('The Covenant seized control of Ironreach.')).toBeVisible();
    // The unknown `meteor_shower` variant must not crash the stream.
    await expect(page.getByText('Something stirs in the world.')).toBeVisible();
  });
});
