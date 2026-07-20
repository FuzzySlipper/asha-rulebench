import type { Locator, Page } from '@playwright/test';
import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'play through the interaction-first combat shell @live-artifact',
  async ({ page, collector, liveBaseUrl }) => {
    collector.addNonClaim(
      'This proves the current fixed hero and raider bootstrap encounter. General participant setup, turn sequencing, and authored board metadata remain deferred campaign work.',
    );
    collector.addNonClaim(
      'System rolls are intentionally unpredictable. Replay proves the exact values actually recorded; it does not regenerate entropy.',
    );

    await page.setViewportSize({ width: 1440, height: 1100 });
    await page.goto(liveBaseUrl);
    const workspace = page.getByLabel('Rulebench interactive combat workspace');
    await expect(workspace).toBeVisible();
    await expect(
      workspace.getByRole('region', { name: '1. Battlefield' }),
    ).toContainText('No active encounter');
    await collector.milestone('interactive shell starts explicitly inactive', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    const rulesetDialog = await openRulesetDialog(page, workspace);
    await expect(
      rulesetDialog.getByRole('option', {
        name: 'Field Manual',
        exact: true,
      }),
    ).toBeAttached();
    await collector.milestone(
      'configured rulesets are selectable without typing paths',
      {
        screenshot: true,
        layerSnapshot: { rulesetSetup: await rulesetDialog.innerText() },
      },
    );
    await rulesetDialog
      .getByRole('combobox', { name: 'Configured ruleset' })
      .selectOption({ label: 'Field Manual' });
    await rulesetDialog
      .getByRole('button', { name: 'Load ruleset candidate' })
      .click();
    await expect(rulesetDialog).toContainText('Rust validation accepted');

    await rulesetDialog.getByRole('button', { name: 'Close' }).click();
    await invokeMenuItem(
      page,
      workspace,
      'Ruleset',
      'Artifact and provenance…',
    );
    const artifactDialog = page.getByRole('dialog', {
      name: 'Artifact and provenance',
    });
    await expect(artifactDialog).toContainText('rulebench.fresh-start@1.0.0');
    await expect(artifactDialog).toContainText('Arc Lash: Stormfront');
    await expect(artifactDialog).toContainText('Materialization provenance');
    await collector.milestone(
      'compiled artifact remains inspectable but secondary',
      {
        screenshot: true,
        layerSnapshot: { artifact: await artifactDialog.innerText() },
      },
    );
    await artifactDialog.getByRole('button', { name: 'Close' }).click();

    await openRulesetDialog(page, workspace);
    await rulesetDialog
      .getByRole('button', { name: 'Activate accepted artifact' })
      .click();

    const battlefield = workspace.getByRole('region', {
      name: '1. Battlefield',
    });
    const participants = workspace.getByRole('region', {
      name: '2. Participants',
    });
    const turnStatus = workspace.getByRole('region', {
      name: '3. Turn status',
    });
    const actionPalette = workspace.getByRole('region', {
      name: '4. Action palette',
    });
    const combatLog = workspace.getByRole('region', { name: '5. Combat log' });
    const applicationMenu = workspace.getByRole('region', {
      name: '0. Application menu',
    });

    const menuBounds = await applicationMenu.boundingBox();
    const battlefieldBounds = await battlefield.boundingBox();
    expect(menuBounds).not.toBeNull();
    expect(battlefieldBounds).not.toBeNull();
    if (menuBounds !== null && battlefieldBounds !== null) {
      expect(
        battlefieldBounds.y - (menuBounds.y + menuBounds.height),
      ).toBeLessThanOrEqual(2);
    }

    await expect(
      battlefield.getByRole('grid', { name: 'Combat grid' }),
    ).toBeVisible();
    await expect(participants).toContainText('hero · ally');
    await expect(participants).toContainText('raider · enemy');
    await expect(turnStatus).toContainText('Authority revision 0');
    await collector.milestone(
      'desktop combat workspace is the primary surface',
      {
        screenshot: true,
        layerSnapshot: { visibleState: await workspace.innerText() },
      },
    );

    await chooseAction(actionPalette, 'Tactical Advance');
    await expect(
      battlefield.getByRole('gridcell', {
        name: /raider, enemy.*available target/,
      }),
    ).toBeVisible();
    await actionPalette
      .getByRole('button', { name: /^Use Tactical Advance/ })
      .click();
    await expect(turnStatus).toContainText('Authority revision 1');
    await expect(participants).toContainText('cell (2, 0)');

    await chooseAction(
      actionPalette,
      'Arc Lash',
      'rulebench.arc-lash-stormfront',
    );
    await actionPalette.getByRole('button', { name: /^Use Arc Lash/ }).click();
    await expect(turnStatus).toContainText('Authority revision 2');
    await expect(participants).toContainText('focus 1/2');
    await expect(combatLog).toContainText('Automatic roll · 1d20 →');
    await collector.milestone(
      'automatic system roll returned visible authority evidence',
      {
        screenshot: true,
        layerSnapshot: { outcome: await combatLog.innerText() },
      },
    );

    await chooseAction(actionPalette, 'Wardbreaker Volley');
    await actionPalette
      .getByRole('button', { name: /^Use Wardbreaker Volley/ })
      .click();
    await expect(actionPalette).toContainText('Reaction for raider');
    await expect(actionPalette.locator('.reaction-card')).toBeFocused();
    await actionPalette.getByRole('button', { name: /^Raise ward/ }).click();
    await expect(turnStatus).toContainText('Authority revision 3');
    await expect(combatLog).toContainText('reactionResolved:');
    const combatLogContents = combatLog.getByRole('group', {
      name: 'Combat log contents',
    });
    const logOverflow = await combatLogContents.evaluate((element) => ({
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
    }));
    expect(logOverflow.scrollHeight).toBeGreaterThan(logOverflow.clientHeight);

    const replayDialog = await openReplayDialog(page, workspace);
    await expect(replayDialog).toContainText('4. react reaction.raise-ward');
    await expect(replayDialog).toContainText('Recorded roll:');
    await replayDialog
      .getByRole('button', { name: 'Verify stored replay' })
      .click();
    await expect(replayDialog).toContainText(
      'Rust replay verified 4 record(s)',
    );
    await collector.milestone('recorded system rolls replay exactly in Rust', {
      screenshot: true,
      layerSnapshot: { replay: await replayDialog.innerText() },
    });
    await replayDialog.getByRole('button', { name: 'Close' }).click();

    await page.setViewportSize({ width: 390, height: 844 });
    await expect(workspace).toBeVisible();
    await expect(battlefield).toBeVisible();
    await expect(actionPalette).toBeVisible();
    const dimensions = await page.evaluate(() => ({
      body: document.body.scrollWidth,
      viewport: document.documentElement.clientWidth,
    }));
    expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
    await collector.milestone('narrow combat workspace remains operable', {
      screenshot: true,
      layerSnapshot: {
        viewport: '390x844',
        visibleState: await workspace.innerText(),
      },
    });
  },
);

async function openRulesetDialog(
  page: Page,
  workspace: Locator,
): Promise<Locator> {
  await invokeMenuItem(page, workspace, 'Ruleset', 'Load or switch ruleset…');
  const dialog = page.getByRole('dialog', { name: 'Ruleset setup' });
  await expect(dialog).toBeVisible();
  return dialog;
}

async function openReplayDialog(
  page: Page,
  workspace: Locator,
): Promise<Locator> {
  await invokeMenuItem(page, workspace, 'Session', 'Replay and checkpoints…');
  const dialog = page.getByRole('dialog', {
    name: 'Replay and checkpoint tools',
  });
  await expect(dialog).toBeVisible();
  return dialog;
}

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

async function chooseAction(
  actionPalette: Locator,
  actionName: string,
  actionId?: string,
): Promise<void> {
  const choice = actionPalette
    .locator('button.action-choice:not(:disabled)')
    .filter({ hasText: actionName });
  await (
    actionId === undefined ? choice : choice.filter({ hasText: actionId })
  ).click();
  await expect(
    actionPalette.getByRole('heading', { name: actionName }),
  ).toBeVisible();
}
