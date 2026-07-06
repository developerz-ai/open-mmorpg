import { expect, test } from '@playwright/test';

/**
 * Cash shop E2E tests with feature flag ENABLED.
 * Run with: CASH_SHOP_ENABLED=true bun run test:e2e:shop
 * These tests are skipped by default in the standard test run.
 */

test.describe('cash shop (feature enabled)', () => {
  test.skip(!process.env.CASH_SHOP_ENABLED, 'Skipping - cash shop feature disabled');

  test('browse categories and items', async ({ page }) => {
    await page.goto('/shop');

    // Page heading
    await expect(page.getByRole('heading', { name: 'Cash Shop' })).toBeVisible();

    // Category filters appear
    await expect(page.getByRole('button', { name: 'All' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Boosts' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Cosmetics' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Account' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Mounts' })).toBeVisible();

    // Items table renders - check for table structure
    await expect(page.locator('table')).toBeVisible();
    await expect(page.locator('th:has-text("Item")')).toBeVisible();
    await expect(page.locator('th:has-text("Category")')).toBeVisible();
    await expect(page.locator('th:has-text("Price")')).toBeVisible();

    // Sample items from mocked data
    await expect(page.getByText('XP Boost (24h)')).toBeVisible();
    await expect(page.getByText('Mount: Shadow Stallion')).toBeVisible();
  });

  test('filter by category narrows items', async ({ page }) => {
    await page.goto('/shop');

    // Click "Boosts" category
    await page.getByRole('button', { name: 'Boosts' }).click();

    // Only boost items visible
    await expect(page.getByText('XP Boost (24h)')).toBeVisible();

    // Mount should not be visible
    await expect(page.getByText('Mount: Shadow Stallion')).not.toBeVisible();

    // Click "All" to reset
    await page.getByRole('button', { name: 'All' }).click();
    await expect(page.getByText('Mount: Shadow Stallion')).toBeVisible();
  });

  test('purchase intent shows confirmed outcome', async ({ page }) => {
    await page.goto('/shop');

    // Click first Buy button
    await page.getByRole('button', { name: 'Buy' }).first().click();

    // Success message appears with server-confirmed outcome
    await expect(page.getByText(/Purchased .+ for \d+ credits\./)).toBeVisible();
  });
});
