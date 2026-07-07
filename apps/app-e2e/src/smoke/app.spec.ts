import { expect, test } from '@playwright/test';

test('boots the template shell', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'UI Pattern Bootstrap', exact: true })).toBeVisible();
  await expect(page.getByText('Layer skeleton online')).toBeVisible();
  await expect(page.getByText('Session idle')).toBeVisible();
});
