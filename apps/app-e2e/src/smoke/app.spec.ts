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

  const encounterDialog = page.getByRole('dialog', {
    name: 'Encounter setup',
  });
  const artifactId = (
    await encounterDialog
      .locator('[data-setup-path="$.artifactId"] code')
      .textContent()
  )?.trim();
  expect(artifactId).toBeTruthy();
  if (artifactId === undefined || artifactId.length === 0) return;
  await importEncounterDocument(encounterDialog, artifactId, {
    first: { id: 'hero', label: 'Hero', teamId: 'team.blue', x: 0, y: 0 },
    second: {
      id: 'raider',
      label: 'Raider',
      teamId: 'team.red',
      x: 4,
      y: 0,
    },
    secondVitality: 0,
  });
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(encounterDialog).toBeHidden();

  const turnStatus = workspace.getByRole('region', { name: '3. Turn status' });
  await expect(turnStatus).toContainText('Encounter complete');
  await expect(turnStatus).toContainText('Winning team: team.blue');

  await invokeMenuItem(page, workspace, 'Session', 'Start new encounter…');
  await importEncounterDocument(encounterDialog, 'artifact-mismatch', {
    first: { id: 'hero', label: 'Hero', teamId: 'team.blue', x: 0, y: 0 },
    second: {
      id: 'raider',
      label: 'Raider',
      teamId: 'team.red',
      x: 4,
      y: 0,
    },
    secondVitality: 40,
  });
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  const artifactBinding = encounterDialog.locator(
    '[data-setup-path="$.artifactId"]',
  );
  await expect(artifactBinding).toContainText(
    `expected artifact ${artifactId}`,
  );
  await expect(artifactBinding).toBeFocused();
  await encounterDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(turnStatus).toContainText('Encounter complete');

  await invokeMenuItem(page, workspace, 'Session', 'Start new encounter…');
  await configureEncounter(encounterDialog, {
    first: { id: 'hero', label: 'Hero', teamId: 'team.blue', x: 0, y: 0 },
    second: {
      id: 'raider',
      label: 'Raider',
      teamId: 'team.red',
      x: 4,
      y: 0,
    },
  });

  const battlefield = workspace.getByRole('region', { name: '1. Battlefield' });
  const participants = workspace.getByRole('region', {
    name: '2. Participants',
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
  await expect(participants).toContainText('Hero · team.blue');
  await expect(participants).toContainText('Raider · team.red');
  await expect(participants).toContainText('Current actor');
  await expect(turnStatus).toContainText('hero is acting');
  await expect(turnStatus).toContainText('Round 1 · turn 1');
  await expect(turnStatus).toContainText('Authority revision 0');

  await actionPalette.getByRole('button', { name: /^End turn/ }).click();
  await expect(turnStatus).toContainText('raider is acting');
  await expect(turnStatus).toContainText('Authority revision 1');
  await actionPalette.getByRole('button', { name: /^End turn/ }).click();
  await expect(turnStatus).toContainText('hero is acting');
  await expect(turnStatus).toContainText('Authority revision 2');

  const heroCell = battlefield.getByRole('gridcell', {
    name: /Hero, team.blue.*current actor/,
  });
  await heroCell.focus();
  await heroCell.press('ArrowRight');
  await expect(
    battlefield.getByRole('gridcell', { name: 'Cell 1, 0, empty' }),
  ).toBeFocused();

  await chooseAction(actionPalette, 'Tactical Advance');
  const tacticalTarget = battlefield.getByRole('gridcell', {
    name: /Raider, team.red.*available target/,
  });
  await expect(tacticalTarget).toBeVisible();
  await tacticalTarget.click();
  await actionPalette
    .getByRole('button', { name: /^Use Tactical Advance/ })
    .click();
  await expect(turnStatus).toContainText('Authority revision 3');
  await expect(turnStatus).toContainText('raider is acting');
  await expect(turnStatus).toContainText('Round 2 · turn 4');
  await expect(participants).toContainText('cell (2, 0)');
  await expect(participants).toContainText('exposed -2');

  await chooseAction(
    actionPalette,
    'Arc Lash',
    'rulebench.arc-lash-stormfront',
  );
  await actionPalette.getByRole('button', { name: 'Participant hero' }).click();
  await actionPalette.getByRole('button', { name: /^Use Arc Lash/ }).click();
  await expect(turnStatus).toContainText('Authority revision 4');
  await expect(turnStatus).toContainText('hero is acting');
  await expect(participants).toContainText('focus 2/3');
  await expect(combatLog).toContainText('Automatic roll · 1d20 → 10');
  await expect(combatLog).toContainText('Automatic roll · 1d6 → 3');
  await expect(combatLog).toContainText('Automatic roll · 1d6 → 4');
  await expect(combatLog).toContainText('damageApplied:');

  await chooseAction(actionPalette, 'Wardbreaker Volley');
  await actionPalette
    .getByRole('button', { name: 'Participant raider' })
    .click();
  await actionPalette
    .getByRole('button', { name: /^Use Wardbreaker Volley/ })
    .click();
  await expect(actionPalette).toContainText('Reaction for raider');
  await expect(
    actionPalette.getByRole('button', { name: /^Raise ward/ }),
  ).toBeVisible();
  await expect(actionPalette.locator('.reaction-card')).toBeFocused();
  await actionPalette.getByRole('button', { name: /^Raise ward/ }).click();
  await expect(turnStatus).toContainText('Authority revision 5');
  await expect(turnStatus).toContainText('raider is acting');
  await expect(participants).toContainText('focus 2/3');
  await expect(combatLog).toContainText('reactionResolved:');
  await expect(combatLog).toContainText('Automatic roll');
  const combatLogContents = combatLog.getByRole('group', {
    name: 'Combat log contents',
  });
  const logOverflow = await combatLogContents.evaluate((element) => ({
    clientHeight: element.clientHeight,
    scrollHeight: element.scrollHeight,
  }));
  expect(logOverflow.scrollHeight).toBeGreaterThan(logOverflow.clientHeight);
  await combatLogContents.focus();
  await combatLogContents.press('End');
  await expect
    .poll(() => combatLogContents.evaluate((element) => element.scrollTop))
    .toBeGreaterThan(0);

  const replayDialog = await openReplayDialog(page, workspace);
  const replayRecords = replayDialog.getByRole('list', {
    name: 'Replay records',
  });
  await expect(replayRecords).toContainText('1. control.end-turn by hero');
  await expect(replayRecords).toContainText('2. control.end-turn by raider');
  await expect(replayRecords).toContainText('6. react reaction.raise-ward');
  await expect(replayRecords).toContainText('accepted');
  await replayDialog
    .getByRole('button', { name: 'Restore latest checkpoint' })
    .click();
  await expect(replayDialog).toContainText('checkpointRestored');
  await replayDialog
    .getByRole('button', { name: 'Verify stored replay' })
    .click();
  await expect(replayDialog).toContainText('Rust replay verified 6 record(s)');
  await replayDialog.getByRole('button', { name: 'Close' }).click();

  await invokeMenuItem(page, workspace, 'Session', 'Start new encounter…');
  await configureEncounter(encounterDialog, {
    first: {
      id: 'scout',
      label: 'Scout',
      teamId: 'team.gold',
      x: 0,
      y: 1,
    },
    second: {
      id: 'brute',
      label: 'Brute',
      teamId: 'team.violet',
      x: 4,
      y: 1,
    },
  });
  await expect(participants).toContainText('Scout · team.gold');
  await expect(participants).toContainText('Brute · team.violet');
  await expect(turnStatus).toContainText('scout is acting');
  await expect(turnStatus).toContainText('Authority revision 0');

  await invokeMenuItem(page, workspace, 'Session', 'Start new encounter…');
  await authorEncounterDraft(encounterDialog, {
    first: {
      id: 'pathfinder',
      label: 'Pathfinder',
      teamId: 'team.gold',
      x: 0,
      y: 0,
    },
    second: {
      id: 'sentinel',
      label: 'Sentinel',
      teamId: 'team.violet',
      x: 0,
      y: 0,
    },
  });
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(encounterDialog).toContainText('RPG_SETUP_POSITION_OCCUPIED');
  const draftEditors = encounterDialog.locator('.participant-editor');
  const secondPositionX = draftEditors
    .nth(1)
    .getByRole('spinbutton', { name: 'Position X' });
  await expectDescribedSetupError(encounterDialog, secondPositionX);
  await expect(secondPositionX).toBeFocused();
  await expectUniqueSetupErrorIds(encounterDialog);

  await secondPositionX.fill('4');
  const boardWidth = encounterDialog.locator(
    '[data-setup-path="$.board.width"]',
  );
  const unrelatedVitalityCurrent = draftEditors
    .nth(0)
    .getByRole('group', { name: 'vitality capability 1' })
    .getByRole('spinbutton', { name: 'Current' });
  await boardWidth.fill('0');
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(encounterDialog).toContainText('RPG_SETUP_BOARD_EXTENT_INVALID');
  await expectDescribedSetupError(encounterDialog, boardWidth);
  await expect(boardWidth).toBeFocused();
  await expect(unrelatedVitalityCurrent).not.toHaveAttribute(
    'aria-invalid',
    'true',
  );
  await expectUniqueSetupErrorIds(encounterDialog);

  await boardWidth.fill('5');
  await encounterDialog
    .getByRole('button', { name: 'Add terrain cell' })
    .click();
  const terrainCell = encounterDialog.locator('[aria-label="Terrain cell 1"]');
  await terrainCell.getByRole('button', { name: 'Add traversal' }).click();
  const traversalCapability = terrainCell.getByRole('group', {
    name: 'traversal capability 1',
  });
  const capabilityId = traversalCapability.getByRole('textbox', {
    name: 'Capability ID',
  });
  const capabilityVersion = traversalCapability.getByRole('spinbutton', {
    name: 'Version',
  });
  await capabilityId.fill('');
  await capabilityVersion.fill('0');
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(encounterDialog).toContainText(
    'RPG_SETUP_CELL_CAPABILITY_ID_INVALID',
  );
  await expect(encounterDialog).toContainText(
    'RPG_SETUP_CELL_CAPABILITY_VERSION_INVALID',
  );
  await expectDescribedSetupError(encounterDialog, capabilityId);
  await expectDescribedSetupError(encounterDialog, capabilityVersion);
  await expect(capabilityId).toBeFocused();
  await expectDescribedSetupError(encounterDialog, traversalCapability);
  await expectUniqueSetupErrorIds(encounterDialog);

  await capabilityId.fill('capability.traversal');
  await capabilityVersion.fill('1');
  const firstParticipant = draftEditors.nth(0);
  await firstParticipant
    .getByRole('group', { name: 'vitality capability 1' })
    .getByRole('button', { name: 'Remove capability' })
    .click();
  const addVitality = firstParticipant.getByRole('button', {
    name: 'Add vitality',
  });
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(encounterDialog).toContainText('RPG_SETUP_VITALITY_REQUIRED');
  await expectDescribedSetupError(encounterDialog, addVitality);
  await expect(addVitality).toBeFocused();
  await expectUniqueSetupErrorIds(encounterDialog);
  await encounterDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(participants).toContainText('Scout · team.gold');
  await expect(turnStatus).toContainText('Authority revision 0');

  await invokeMenuItem(page, workspace, 'Session', 'Start new encounter…');
  await encounterDialog
    .getByRole('button', { name: 'Add participant' })
    .click();
  await encounterDialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(encounterDialog).toContainText(
    'RPG_SETUP_PARTICIPANT_ACTION_REQUIRED',
  );
  await encounterDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(participants).toContainText('Scout · team.gold');
  await expect(turnStatus).toContainText('Authority revision 0');

  await chooseAction(
    actionPalette,
    'Arc Lash',
    'rulebench.arc-lash-stormfront',
  );
  await actionPalette
    .getByRole('button', { name: 'Participant brute' })
    .click();
  await actionPalette.getByRole('button', { name: /^Use Arc Lash/ }).click();
  const gameplayFailure = combatLog.getByRole('alert');
  await expect(gameplayFailure).toContainText(
    'Gameplay request could not be completed',
  );
  await expect(gameplayFailure).toContainText('RULESET_RANDOM_TAPE_EXHAUSTED');
  await expect(combatLog).toBeFocused();
  await expect(turnStatus).toContainText('Authority revision 0');

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
  await expect(participants).toContainText('Scout · team.gold');
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

interface EncounterParticipantInput {
  readonly id: string;
  readonly label: string;
  readonly teamId: string;
  readonly x: number;
  readonly y: number;
}

async function importEncounterDocument(
  dialog: Locator,
  artifactId: string,
  setup: {
    readonly first: EncounterParticipantInput;
    readonly second: EncounterParticipantInput;
    readonly secondVitality: number;
  },
): Promise<void> {
  const document = {
    schema: { id: 'asha.rpg.encounter.setup', version: 1 },
    artifactId,
    board: { width: 5, height: 3, cells: [] },
    participants: [
      participantSetup(setup.first, 40),
      participantSetup(setup.second, setup.secondVitality),
    ],
    turn: {
      initiativeOrder: [setup.first.id, setup.second.id],
      currentActorId: setup.first.id,
      round: 1,
      turn: 1,
    },
    randomSource: {
      policyId: 'rulebench.automatic-random',
      policyVersion: 1,
      sourceId: 'rulebench.roll-tape',
      sourceVersion: 1,
    },
  };
  await dialog.locator('input[type="file"]').setInputFiles({
    name: 'review-encounter.setup.json',
    mimeType: 'application/json',
    buffer: Buffer.from(JSON.stringify(document)),
  });
  await expect(dialog).toContainText('Loaded review-encounter.setup.json');
}

function participantSetup(
  participant: EncounterParticipantInput,
  vitality: number,
) {
  return {
    id: participant.id,
    label: participant.label,
    teamId: participant.teamId,
    position: { x: participant.x, y: participant.y },
    definitionIds: [
      'rulebench.tactical-advance',
      'rulebench.arc-lash-stormfront',
      'rulebench.wardbreaker-volley',
    ],
    capabilities: [
      { owner: 'vitality', value: { current: vitality, max: 40 } },
      { owner: 'stat', id: 'power', value: 3 },
      { owner: 'defense', id: 'guard', value: 12 },
      {
        owner: 'resource',
        id: 'focus',
        value: { current: 3, max: 3 },
      },
    ],
  };
}

async function configureEncounter(
  dialog: Locator,
  setup: {
    readonly first: EncounterParticipantInput;
    readonly second: EncounterParticipantInput;
  },
): Promise<void> {
  await authorEncounterDraft(dialog, setup);
  await dialog
    .getByRole('button', { name: 'Validate and start encounter' })
    .click();
  await expect(dialog).toBeHidden();
}

async function authorEncounterDraft(
  dialog: Locator,
  setup: {
    readonly first: EncounterParticipantInput;
    readonly second: EncounterParticipantInput;
  },
): Promise<void> {
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: 'Add participant' }).click();
  await dialog.getByRole('button', { name: 'Add participant' }).click();
  const editors = dialog.locator('.participant-editor');
  await fillParticipant(editors.nth(0), setup.first);
  await fillParticipant(editors.nth(1), setup.second);
  for (const editor of [editors.nth(0), editors.nth(1)]) {
    const definitions = editor.getByRole('checkbox');
    const count = await definitions.count();
    for (let index = 0; index < count; index += 1) {
      await definitions.nth(index).check();
    }
  }
}

async function expectDescribedSetupError(
  dialog: Locator,
  control: Locator,
): Promise<void> {
  await expect(control).toHaveAttribute('aria-invalid', 'true');
  const describedBy = await control.getAttribute('aria-describedby');
  expect(describedBy).not.toBeNull();
  if (describedBy === null) {
    throw new Error('invalid setup control did not name its diagnostic');
  }
  const description = dialog.locator(`[id="${describedBy}"]`);
  await expect(description).toHaveCount(1);
  await expect(description).toBeVisible();
}

async function expectUniqueSetupErrorIds(dialog: Locator): Promise<void> {
  const ids = await dialog
    .locator('[id^="setup-error-"]')
    .evaluateAll((elements) => elements.map((element) => element.id));
  expect(new Set(ids).size).toBe(ids.length);
}

async function fillParticipant(
  editor: Locator,
  participant: EncounterParticipantInput,
): Promise<void> {
  await editor
    .getByRole('textbox', { name: 'ID', exact: true })
    .fill(participant.id);
  await editor.getByRole('textbox', { name: 'Label' }).fill(participant.label);
  await editor
    .getByRole('textbox', { name: 'Team ID' })
    .fill(participant.teamId);
  await editor
    .getByRole('spinbutton', { name: 'Position X' })
    .fill(participant.x.toString());
  await editor
    .getByRole('spinbutton', { name: 'Position Y' })
    .fill(participant.y.toString());
  const vitality = editor.getByRole('group', {
    name: 'vitality capability 1',
  });
  await vitality.getByRole('spinbutton', { name: 'Current' }).fill('40');
  await vitality.getByRole('spinbutton', { name: 'Maximum' }).fill('40');

  await editor.getByRole('button', { name: 'Add stat' }).click();
  const stat = editor.getByRole('group', { name: 'stat capability 2' });
  await stat.getByRole('textbox', { name: 'ID' }).fill('power');
  await stat.getByRole('spinbutton', { name: 'Value' }).fill('3');

  await editor.getByRole('button', { name: 'Add defense' }).click();
  const defense = editor.getByRole('group', {
    name: 'defense capability 3',
  });
  await defense.getByRole('textbox', { name: 'ID' }).fill('guard');
  await defense.getByRole('spinbutton', { name: 'Value' }).fill('12');

  await editor.getByRole('button', { name: 'Add resource' }).click();
  const resource = editor.getByRole('group', {
    name: 'resource capability 4',
  });
  await resource.getByRole('textbox', { name: 'ID' }).fill('focus');
  await resource.getByRole('spinbutton', { name: 'Current' }).fill('3');
  await resource.getByRole('spinbutton', { name: 'Maximum' }).fill('3');
}

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
