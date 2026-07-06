import { expect, test } from '@playwright/test';

/**
 * Cash shop E2E tests with feature flag DISABLED (default).
 * Tests that the route shows 404 when cash shop is disabled.
 * For tests with the feature enabled, see shop-enabled.e2e.ts
 */

test.describe('cash shop (feature disabled)', () => {
  test('feature flag off shows not found, not a crash', async ({ page }) => {
    // Default config has cashShop disabled (VITE_CASH_SHOP=false)
    await page.goto('/shop');
    await expect(page.getByText('This page does not exist on this realm.')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Return home' })).toBeVisible();
  });
});
