import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

test('selects a Ruleset and Content Packs before activating a PlayBundle @gate', async ({
  page,
}) => {
  await page.goto('/');

  const workspace = page.getByLabel('Rulebench interactive combat workspace');
  await expect(workspace).toBeVisible();
  await expect(workspace).toContainText('No active session');
  await expect(workspace).toContainText('No PlayBundle active');

  await invokeMenuItem(
    page,
    workspace,
    'Play',
    'Choose Ruleset and Content Packs…',
  );
  const playDialog = page.getByRole('dialog', {
    name: 'Choose play content',
  });
  await expect(playDialog).toBeVisible();

  await playDialog
    .getByRole('combobox', { name: 'Configured source set' })
    .selectOption({ label: 'Rulebench minimal contract fixture' });

  await expect(playDialog).toContainText('rulebench.minimal@1.0.0');
  await expect(playDialog).toContainText('rulebench.minimal.content@1.0.0');
  await expect(playDialog).not.toContainText('Compatible PlayBundle');

  await playDialog
    .getByRole('checkbox', { name: /rulebench\.minimal\.content/ })
    .check();
  await expect(playDialog).toContainText('Compatible PlayBundle');
  await expect(playDialog).toContainText('rulebench.minimal.play@1.0.0');

  await playDialog
    .getByRole('button', { name: 'Compile selected PlayBundle' })
    .click();
  await expect(playDialog).toContainText('candidate');
  await expect(
    playDialog.getByRole('button', {
      name: 'Activate compiled PlayBundle',
    }),
  ).toBeEnabled();

  await playDialog
    .getByRole('button', { name: 'Activate compiled PlayBundle' })
    .click();

  const scenarioDialog = page.getByRole('dialog', { name: 'Scenario setup' });
  await expect(scenarioDialog).toBeVisible();
  await expect(scenarioDialog).toContainText('PlayBundle binding');
  await expect(scenarioDialog).toContainText('Choose an explicit JSON setup');
  await scenarioDialog.getByRole('button', { name: 'Cancel' }).click();

  await expect(workspace).toContainText('PlayBundle active');
  await expect(workspace).toContainText('No active session');

  await page.setViewportSize({ width: 390, height: 844 });
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
});

async function invokeMenuItem(
  page: Page,
  workspace: Locator,
  menuName: string,
  itemName: string,
): Promise<void> {
  const menubar = workspace.getByRole('menubar', {
    name: 'Rulebench application menu',
  });
  await menubar.getByRole('menuitem', { name: menuName, exact: true }).click();
  await page
    .getByRole('menu', { name: menuName })
    .getByRole('menuitem', { name: itemName, exact: true })
    .click();
}
