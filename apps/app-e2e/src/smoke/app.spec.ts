import { expect, test } from '@playwright/test';

test('boots the rulebench shell', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'ASHA Rulebench', exact: true })).toBeVisible();
  await expect(page.getByText('Rule workbench shell online')).toBeVisible();
  await expect(page.getByText('Hexing Bolt Opening')).toBeVisible();
});
