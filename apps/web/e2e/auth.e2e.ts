import { expect, test } from '@playwright/test';

test.describe('account & auth', () => {
  test('log in with the seeded account reaches the account projection', async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel('Email').fill('aria@realm.test');
    await page.getByLabel('Password').fill('password123');
    await page.getByRole('button', { name: 'Log in' }).click();

    await expect(page).toHaveURL(/\/account$/);
    await expect(page.getByText('Aria', { exact: true })).toBeVisible();
    await expect(page.getByText('aria@realm.test')).toBeVisible();
  });

  test('bad credentials surface a typed error, not a crash', async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel('Email').fill('aria@realm.test');
    await page.getByLabel('Password').fill('wrong-password');
    await page.getByRole('button', { name: 'Log in' }).click();

    await expect(page.getByText('Email or password is incorrect.')).toBeVisible();
  });
});
