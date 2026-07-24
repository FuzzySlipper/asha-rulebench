import type { Locator, Page } from '@playwright/test';

import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'Starter Skirmish exposes item-bound setup and gameplay @live',
  async ({ page, collector }) => {
    collector.addNonClaim(
      'This scenario covers setup, bound action choice, execution, and authority readback; it does not claim loot, inventory transfer, or drag-and-drop equipment.',
    );
    await page.goto('/');
    const workspace = page.getByLabel('Rulebench interactive combat workspace');

    await openMenuItem(
      page,
      workspace,
      'Play',
      'Choose Ruleset and Content Packs…',
    );
    const playDialog = page.getByRole('dialog', {
      name: 'Choose play content',
    });
    await playDialog
      .getByLabel('Configured source set')
      .selectOption('asha-d20-fantasy');
    await playDialog.getByRole('checkbox', { name: /Starter/ }).check();
    await expect(playDialog).toContainText('Compatible PlayBundle');
    await playDialog
      .getByRole('button', { name: 'Compile selected PlayBundle' })
      .click();
    await expect(
      playDialog.getByText('candidate', { exact: true }),
    ).toBeVisible({ timeout: 30_000 });
    await playDialog
      .getByRole('button', { name: 'Activate compiled PlayBundle' })
      .click();

    const scenarioDialog = page.getByRole('dialog', {
      name: 'Scenario setup',
    });
    await scenarioDialog
      .getByRole('button', { name: /Starter Skirmish/ })
      .click();
    const fighterLongsword = scenarioDialog.locator(
      '[data-setup-path="$.participants[0].items[1].id"]',
    );
    const fighterMainHand = scenarioDialog.locator(
      '[data-setup-path="$.participants[0].equipment[0].slotId"]',
    );
    await expect(fighterLongsword).toHaveValue('fighter-longsword');
    await expect(fighterMainHand).toHaveValue('hand.main');

    await fighterMainHand.fill('not-a-slot');
    await scenarioDialog
      .getByRole('button', { name: 'Validate and start Scenario' })
      .click();
    await expect(fighterMainHand).toHaveAttribute('aria-invalid', 'true');
    await expect(fighterMainHand).toBeFocused();
    await expect(scenarioDialog).toContainText(
      'RPG_SCENARIO_EQUIPMENT_SLOT_NOT_ALLOWED',
    );

    await scenarioDialog
      .getByRole('button', { name: /Starter Skirmish/ })
      .click();
    await scenarioDialog
      .getByRole('button', { name: 'Validate and start Scenario' })
      .click();
    await expect(scenarioDialog).not.toBeVisible();

    const actionList = workspace.getByRole('list', {
      name: 'Available actions',
    });
    await expect(
      actionList.getByRole('button', {
        name: /Basic Attack — Shortsword/,
      }),
    ).toBeVisible();
    await expect(
      actionList.getByRole('button', { name: /^Basic Attack$/ }),
    ).toHaveCount(0);
    await collector.milestone('shortsword action available', {
      screenshot: true,
    });

    await executeBoundAction(
      page,
      workspace,
      'Basic Attack — Shortsword',
      'Fighter',
    );
    await declineReactionIfOpen(workspace);

    const history = workspace.getByRole('list', {
      name: 'Combat history',
    });
    await expect(history).toContainText('Item: Shortsword');
    await expect(history).toContainText('skeleton-shortsword');
    await expect(
      actionList.getByRole('button', {
        name: /Basic Attack — Battleaxe/,
      }),
    ).toBeVisible();
    await expect(
      actionList.getByRole('button', {
        name: /Basic Attack — Longsword/,
      }),
    ).toBeVisible();

    const fighter = workspace
      .getByRole('list', { name: 'Session participants' })
      .getByRole('button', { name: /View Fighter character/ });
    await fighter.click();
    const characterDialog = page.getByRole('dialog', { name: 'Fighter' });
    await expect(characterDialog).toContainText('Inventory');
    await expect(characterDialog).toContainText('Longsword');
    await expect(characterDialog).toContainText('Battleaxe');
    await expect(characterDialog).toContainText('Equipment');
    await expect(characterDialog).toContainText('hand.main');
    await collector.milestone('character inventory and equipment', {
      screenshot: true,
    });
    await characterDialog.getByRole('button', { name: 'Close' }).click();

    await executeBoundAction(
      page,
      workspace,
      'Basic Attack — Longsword',
      'Skeleton',
    );
    await declineReactionIfOpen(workspace);
    await expect(history).toContainText('Item: Longsword');
    await expect(history).toContainText('fighter-longsword');

    await workspace.getByRole('button', { name: /End turn/ }).click();
    await expect(
      actionList.getByRole('button', {
        name: /Basic Attack — Scimitar/,
      }),
    ).toBeVisible();
    await executeBoundAction(
      page,
      workspace,
      'Basic Attack — Scimitar',
      'Fighter',
    );
    await declineReactionIfOpen(workspace);
    await expect(history).toContainText('Item: Scimitar');
    await expect(history).toContainText('goblin-scimitar');
    await collector.milestone('three item-bound attacks executed', {
      screenshot: true,
      layerSnapshot: {
        actions: ['Shortsword', 'Longsword', 'Scimitar'],
        source: 'Rust authority readback',
      },
    });

    await page.setViewportSize({ width: 390, height: 844 });
    await fighter.click();
    await expect(characterDialog).toBeVisible();
    await expect(characterDialog).toContainText('Inventory');
    await collector.milestone('narrow character inventory', {
      screenshot: true,
    });
  },
);

async function executeBoundAction(
  page: Page,
  workspace: Locator,
  actionName: string,
  targetName: string,
): Promise<void> {
  await workspace.getByRole('button', { name: new RegExp(actionName) }).click();
  await workspace
    .getByRole('button', { name: new RegExp(`Target ${targetName}`) })
    .click();
  const execute = workspace.getByRole('button', {
    name: `Use ${actionName}`,
  });
  await execute.focus();
  await page.keyboard.press('Space');
  await expect(execute).not.toBeVisible();
}

async function declineReactionIfOpen(workspace: Locator): Promise<void> {
  const decline = workspace.getByRole('button', {
    name: 'Decline reaction',
  });
  if (await decline.isVisible().catch(() => false)) {
    await decline.click();
  }
}

async function openMenuItem(
  page: Page,
  workspace: Locator,
  menuName: string,
  itemName: string,
): Promise<void> {
  await workspace
    .getByRole('menuitem', { name: menuName, exact: true })
    .click();
  await page
    .getByRole('menu', { name: menuName })
    .getByRole('menuitem', { name: itemName, exact: true })
    .click();
}
