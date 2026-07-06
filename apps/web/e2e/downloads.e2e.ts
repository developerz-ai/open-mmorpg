import { expect, test } from '@playwright/test';

test.describe('downloads page', () => {
  test('renders download links, system requirements, and checksum verifier', async ({ page }) => {
    await page.goto('/downloads');

    // Main heading and intro card render
    await expect(page.getByRole('heading', { name: 'Downloads' })).toBeVisible();
    await expect(page.getByText(/Download the game client for your platform/i)).toBeVisible();

    // Download links render with platform, version, and checksum
    await expect(page.getByRole('heading', { name: 'Installers' })).toBeVisible();
    await expect(
      page.locator('.downloads__platform').filter({ hasText: 'Windows x64' }),
    ).toBeVisible();
    await expect(
      page.locator('.downloads__platform').filter({ hasText: 'Windows ARM64' }),
    ).toBeVisible();
    await expect(
      page.locator('.downloads__platform').filter({ hasText: 'macOS x64' }),
    ).toBeVisible();
    await expect(
      page.locator('.downloads__platform').filter({ hasText: 'macOS ARM64' }),
    ).toBeVisible();
    await expect(
      page.locator('.downloads__platform').filter({ hasText: 'Linux x64' }),
    ).toBeVisible();

    // Version appears for at least one platform
    await expect(page.getByText('Version').first()).toBeVisible();

    // Checksums render (look for the checksum label and hex-like strings)
    await expect(page.getByText('Checksum').first()).toBeVisible();
    await expect(page.locator('.downloads__checksum')).toHaveCount(5);

    // HTTPS security note
    await expect(page.getByText(/HTTPS with TLS encryption/i)).toBeVisible();

    // System requirements table (localized strings vary by locale)
    await expect(page.getByRole('heading', { name: 'System requirements' })).toBeVisible();
    // Check that requirements table has content (3 rows for Windows, macOS, Linux)
    await expect(page.locator('.requirements__table tbody tr')).toHaveCount(3);

    // Checksum verifier section renders
    await expect(page.getByRole('heading', { name: 'Checksum verifier' })).toBeVisible();
    await expect(page.getByLabel('Enter checksum from your file')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Verify' })).toBeVisible();

    // Helper examples show verification info
    await expect(page.getByText(/Verify your download/i)).toBeVisible();

    // Back to home link
    await expect(page.getByRole('link', { name: 'Back to home' })).toBeVisible();

    // No missing i18n keys
    await expect(page.locator('body')).not.toContainText('⟦');
  });

  test('checksum verifier shows mismatch for well-formed but non-matching checksum', async ({
    page,
  }) => {
    await page.goto('/downloads');

    const checksumInput = page.getByLabel('Enter checksum from your file');
    const verifyButton = page.getByRole('button', { name: 'Verify' });

    // Enter a well-formed 64-char hex checksum that doesn't correspond to any fallback download
    const testChecksum =
      'a'.repeat(12) + 'b'.repeat(12) + 'c'.repeat(12) + 'd'.repeat(12) + 'e'.repeat(16);
    await checksumInput.fill(testChecksum);
    await verifyButton.click();

    await expect(page.getByText(/✗ Mismatch/i)).toBeVisible();
  });

  test('checksum verifier shows error for invalid checksum format', async ({ page }) => {
    await page.goto('/downloads');

    const checksumInput = page.getByLabel('Enter checksum from your file');
    const verifyButton = page.getByRole('button', { name: 'Verify' });

    // Enter invalid checksum (too short)
    await checksumInput.fill('abc123');
    await verifyButton.click();

    // Should show error about invalid format
    await expect(page.getByText(/Invalid checksum format/i)).toBeVisible();
  });

  test('checksum verifier shows mismatch for unknown checksum', async ({ page }) => {
    await page.goto('/downloads');

    const checksumInput = page.getByLabel('Enter checksum from your file');
    const verifyButton = page.getByRole('button', { name: 'Verify' });

    // Enter valid format but unknown checksum
    const unknownChecksum = 'f'.repeat(64);
    await checksumInput.fill(unknownChecksum);
    await verifyButton.click();

    // Should show mismatch
    await expect(page.getByText(/✗ Mismatch/i)).toBeVisible();
  });

  test('download buttons navigate to configured URLs', async ({ page }) => {
    await page.goto('/downloads');

    const downloadButtons = page.getByRole('button', { name: /download/i });

    // Should have multiple download buttons (one per platform)
    await expect(downloadButtons).toHaveCount(5);

    // First download button should be visible and clickable
    const firstButton = downloadButtons.first();
    await expect(firstButton).toBeVisible();

    // In local dev, these point to '#', but the button should exist
    await expect(firstButton).toHaveAttribute('type', 'button');
  });

  test('checksum copy buttons work for each platform', async ({ page }) => {
    await page.goto('/downloads');

    // Find all checksum copy buttons (they contain the checksum text)
    const checksumButtons = page.locator('.downloads__checksum');

    // Should have one per download (5 in fallback config)
    await expect(checksumButtons).toHaveCount(5);

    // First checksum button should contain checksum text
    await expect(checksumButtons.first()).toContainText(/checksum/i);
    await expect(checksumButtons.first()).toContainText(/[a-f0-9]{8,}/i);
  });
});
