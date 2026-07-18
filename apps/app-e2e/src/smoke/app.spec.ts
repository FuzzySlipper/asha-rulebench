import { expect, test } from '@playwright/test';

test('boots in an honest no-active-ruleset state @gate', async ({ page }) => {
  await page.goto('/');

  const workspace = page.getByLabel('Rulebench empty workspace');
  await expect(workspace).toBeVisible();
  await expect(
    workspace.getByRole('heading', { name: 'No compiled ruleset active' }),
  ).toBeVisible();
  await expect(workspace.getByText('No compiled ruleset active')).toHaveCount(3);
  await expect(workspace).toContainText(
    'Future content enters through the package manifest and compiler boundary',
  );

  const menubar = workspace.getByRole('menubar', {
    name: 'Rulebench application menu',
  });
  await menubar.getByRole('menuitem', { name: 'Ruleset' }).click();
  await expect(
    page
      .getByRole('menu', { name: 'Ruleset' })
      .getByRole('menuitem', { name: 'No compiled ruleset active' }),
  ).toHaveAttribute('aria-disabled', 'true');

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(workspace).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
});
