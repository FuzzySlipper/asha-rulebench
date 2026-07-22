import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

test('loads peer roots and opens participant details @gate', async ({
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
    .selectOption({ label: 'Rulebench split-source demo' });

  await expect(playDialog).toContainText('rulebench.independent@1.0.0');
  await expect(playDialog).toContainText('rulebench.independent.content@1.0.0');
  await expect(playDialog).toContainText('Independent Ruleset:');
  await expect(playDialog).toContainText('Independent Content Pack:');
  await expect(playDialog).toContainText('Independent PlayBundle:');
  await expect(playDialog).toContainText('Independent Scenario:');
  await expect(playDialog).not.toContainText('Compatible PlayBundle');

  await playDialog
    .getByRole('checkbox', { name: /rulebench\.independent\.content/ })
    .check();
  await expect(playDialog).toContainText('Compatible PlayBundle');
  await expect(playDialog).toContainText('rulebench.independent.play@1.0.0');

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
  const scenarioExample = scenarioDialog.getByRole('button', {
    name: /Independent source scenario/,
  });
  await expect(scenarioExample).toBeVisible();
  await expect(scenarioDialog).toContainText('Choose an explicit JSON setup');
  await scenarioExample.click();
  await expect(scenarioExample).toHaveAttribute('aria-pressed', 'true');
  await expect(scenarioDialog).toContainText(
    'Selected setup: Independent source scenario',
  );
  await scenarioDialog
    .getByRole('button', { name: 'Validate and start Scenario' })
    .click();
  await expect(scenarioDialog).not.toBeVisible();

  await expect(workspace).toContainText('Live Rust authority session');
  const action = workspace.getByRole('button', { name: /Catch Breath/ });
  const actionFontSize = await action.evaluate((element) =>
    Number.parseFloat(getComputedStyle(element).fontSize),
  );
  expect(actionFontSize).toBeLessThan(14);
  const actionDescription = workspace.getByText('Recover one hit point.', {
    exact: true,
  });
  await expect(actionDescription).toHaveCount(1);
  await action.click();
  await expect(
    workspace.getByRole('heading', { name: 'Catch Breath' }),
  ).toBeVisible();
  await expect(actionDescription).toHaveCount(1);

  const participants = workspace.getByRole('list', {
    name: 'Session participants',
  });
  await expect(participants).toContainText('Demo Hero');
  await expect(participants).toContainText('Faction allies');
  await expect(participants).toContainText('HP 10/10');
  await expect(participants).not.toContainText('demo-hero');
  await expect(participants).not.toContainText('Current actor');
  await expect(participants).not.toContainText('power 3');

  const hero = participants.getByRole('button', {
    name: 'View Demo Hero character, faction allies, hit points 10/10',
  });
  await hero.click();
  const characterDialog = page.getByRole('dialog', { name: 'Demo Hero' });
  await expect(characterDialog).toBeVisible();
  const closeCharacterDialog = characterDialog.getByRole('button', {
    name: 'Close',
  });
  await expect(closeCharacterDialog).toBeFocused();
  await expect(characterDialog).toContainText('demo-hero');
  await expect(characterDialog).toContainText('Current actor');
  await expect(characterDialog).toContainText('(0, 0)');
  await expect(
    characterDialog.getByRole('heading', { name: 'Stats' }),
  ).toBeVisible();
  await expect(
    characterDialog.getByRole('heading', { name: 'Defenses' }),
  ).toBeVisible();
  await expect(
    characterDialog.getByRole('heading', { name: 'Resources' }),
  ).toBeVisible();
  await expect(
    characterDialog.getByRole('heading', { name: 'Modifiers' }),
  ).toBeVisible();
  await closeCharacterDialog.click();
  await expect(characterDialog).not.toBeVisible();
  await expect(hero).toBeFocused();

  await page.setViewportSize({ width: 390, height: 844 });
  await hero.click();
  await expect(characterDialog).toBeVisible();
  await expect(closeCharacterDialog).toBeFocused();
  await closeCharacterDialog.click();
  await expect(hero).toBeFocused();
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
