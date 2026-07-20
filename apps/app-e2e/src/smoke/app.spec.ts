import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

test('plays an encounter through the interaction-first authority loop @gate', async ({
  page,
}) => {
  await page.goto('/');

  const workspace = page.getByLabel('Rulebench interactive combat workspace');
  await expect(workspace).toBeVisible();
  await expect(
    workspace.getByRole('region', { name: '1. Battlefield' }),
  ).toContainText('No active encounter');

  const rulesetDialog = await openRulesetDialog(page, workspace);
  await selectConfiguredRuleset(rulesetDialog, 'Field Manual');
  await rulesetDialog
    .getByRole('button', { name: 'Load ruleset candidate' })
    .click();
  await expect(rulesetDialog).toContainText('Rust validation accepted');
  await expect(rulesetDialog).toContainText('candidate');
  await rulesetDialog.getByRole('button', { name: 'Close' }).click();

  await openArtifactDialog(page, workspace);
  const artifactDialog = page.getByRole('dialog', {
    name: 'Artifact and provenance',
  });
  await expect(artifactDialog).toContainText('rulebench.fresh-start@1.0.0');
  await expect(artifactDialog).toContainText('Tactical Advance');
  await expect(artifactDialog).toContainText('Arc Lash: Stormfront');
  await expect(artifactDialog).toContainText('Wardbreaker Volley');
  await expect(artifactDialog).toContainText('Fingerprint planes');
  await expect(artifactDialog).toContainText('Materialization provenance');
  await artifactDialog.getByRole('button', { name: 'Close' }).click();

  await openRulesetDialog(page, workspace);
  await rulesetDialog
    .getByRole('button', { name: 'Activate accepted artifact' })
    .click();
  await expect(rulesetDialog).toBeHidden();

  const battlefield = workspace.getByRole('region', { name: '1. Battlefield' });
  const participants = workspace.getByRole('region', {
    name: '2. Participants',
  });
  const turnStatus = workspace.getByRole('region', { name: '3. Turn status' });
  const actionPalette = workspace.getByRole('region', {
    name: '4. Action palette',
  });
  const combatLog = workspace.getByRole('region', { name: '5. Combat log' });

  await expect(
    battlefield.getByRole('grid', { name: 'Combat grid' }),
  ).toBeVisible();
  await expect(participants).toContainText('hero · ally');
  await expect(participants).toContainText('raider · enemy');
  await expect(participants).toContainText('Current actor');
  await expect(turnStatus).toContainText('hero is acting');
  await expect(turnStatus).toContainText('Authority revision 0');

  const heroCell = battlefield.getByRole('gridcell', {
    name: /hero, ally.*current actor/,
  });
  await heroCell.focus();
  await heroCell.press('ArrowRight');
  await expect(
    battlefield.getByRole('gridcell', { name: 'Cell 1, 0, empty' }),
  ).toBeFocused();

  await chooseAction(actionPalette, 'Tactical Advance');
  const tacticalTarget = battlefield.getByRole('gridcell', {
    name: /raider, enemy.*available target/,
  });
  await expect(tacticalTarget).toBeVisible();
  await tacticalTarget.click();
  await actionPalette
    .getByRole('button', { name: /^Use Tactical Advance/ })
    .click();
  await expect(turnStatus).toContainText('Authority revision 1');
  await expect(participants).toContainText('cell (2, 0)');
  await expect(participants).toContainText('exposed -2');

  await chooseAction(
    actionPalette,
    'Arc Lash',
    'rulebench.arc-lash-stormfront',
  );
  await actionPalette.getByRole('button', { name: /^Use Arc Lash/ }).click();
  await expect(turnStatus).toContainText('Authority revision 2');
  await expect(participants).toContainText('focus 1/2');
  await expect(combatLog).toContainText('Automatic roll · 1d20 → 10');
  await expect(combatLog).toContainText('Automatic roll · 1d6 → 3');
  await expect(combatLog).toContainText('Automatic roll · 1d6 → 4');
  await expect(combatLog).toContainText('damageApplied:');

  await chooseAction(actionPalette, 'Wardbreaker Volley');
  await actionPalette
    .getByRole('button', { name: /^Use Wardbreaker Volley/ })
    .click();
  await expect(actionPalette).toContainText('Reaction for raider');
  await expect(
    actionPalette.getByRole('button', { name: /^Raise ward/ }),
  ).toBeVisible();
  await expect(actionPalette.locator('.reaction-card')).toBeFocused();
  await actionPalette.getByRole('button', { name: /^Raise ward/ }).click();
  await expect(turnStatus).toContainText('Authority revision 3');
  await expect(participants).toContainText('focus 0/2');
  await expect(combatLog).toContainText('reactionResolved:');
  await expect(combatLog).toContainText('Automatic roll');

  const replayDialog = await openReplayDialog(page, workspace);
  const replayRecords = replayDialog.getByRole('list', {
    name: 'Replay records',
  });
  await expect(replayRecords).toContainText('4. react reaction.raise-ward');
  await expect(replayRecords).toContainText('accepted');
  await replayDialog
    .getByRole('button', { name: 'Restore latest checkpoint' })
    .click();
  await expect(replayDialog).toContainText('checkpointRestored');
  await replayDialog
    .getByRole('button', { name: 'Verify stored replay' })
    .click();
  await expect(replayDialog).toContainText('Rust replay verified 4 record(s)');
  await replayDialog.getByRole('button', { name: 'Close' }).click();

  const nextRulesetDialog = await openRulesetDialog(page, workspace);
  await selectCustomRulesetRoot(
    nextRulesetDialog,
    'examples/rulesets/invalid-build',
  );
  await nextRulesetDialog
    .getByRole('button', { name: 'Load ruleset candidate' })
    .click();
  await expect(nextRulesetDialog).toContainText(
    'RULESET_WORKSPACE_BUILD_FAILED',
  );
  await expect(nextRulesetDialog).toContainText('TS2322');
  await expect(nextRulesetDialog).toContainText('active');
  await nextRulesetDialog.getByRole('button', { name: 'Close' }).click();
  await expect(turnStatus).toContainText('Authority revision 3');

  const exhaustedTapeDialog = await openRulesetDialog(page, workspace);
  await selectConfiguredRuleset(exhaustedTapeDialog, 'Field Manual');
  await exhaustedTapeDialog
    .getByRole('button', { name: 'Load ruleset candidate' })
    .click();
  await expect(exhaustedTapeDialog).toContainText('Rust validation accepted');
  await exhaustedTapeDialog
    .getByRole('button', { name: 'Activate accepted artifact' })
    .click();
  await expect(exhaustedTapeDialog).toBeHidden();
  await expect(turnStatus).toContainText('Authority revision 0');

  await chooseAction(
    actionPalette,
    'Arc Lash',
    'rulebench.arc-lash-stormfront',
  );
  await actionPalette.getByRole('button', { name: /^Use Arc Lash/ }).click();
  const gameplayFailure = combatLog.getByRole('alert');
  await expect(gameplayFailure).toContainText(
    'Gameplay request could not be completed',
  );
  await expect(gameplayFailure).toContainText('RULESET_RANDOM_TAPE_EXHAUSTED');
  await expect(combatLog).toBeFocused();
  await expect(turnStatus).toContainText('Authority revision 0');

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(workspace).toBeVisible();
  await expect(actionPalette).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
});

async function openRulesetDialog(
  page: Page,
  workspace: Locator,
): Promise<Locator> {
  await invokeMenuItem(page, workspace, 'Ruleset', 'Load or switch ruleset…');
  const dialog = page.getByRole('dialog', { name: 'Ruleset setup' });
  await expect(dialog).toBeVisible();
  await expect(
    dialog.getByRole('combobox', { name: 'Configured ruleset' }),
  ).toBeVisible();
  return dialog;
}

async function openArtifactDialog(
  page: Page,
  workspace: Locator,
): Promise<void> {
  await invokeMenuItem(page, workspace, 'Ruleset', 'Artifact and provenance…');
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

async function selectConfiguredRuleset(
  dialog: Locator,
  label: string,
): Promise<void> {
  await dialog
    .getByRole('combobox', { name: 'Configured ruleset' })
    .selectOption({ label });
}

async function selectCustomRulesetRoot(
  dialog: Locator,
  rulesetRoot: string,
): Promise<void> {
  await dialog
    .getByRole('textbox', { name: 'Custom ruleset root' })
    .fill(rulesetRoot);
}
