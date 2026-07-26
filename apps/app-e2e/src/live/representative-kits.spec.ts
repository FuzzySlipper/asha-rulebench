import type { Locator, Page } from '@playwright/test';
import {
  decodePlayWorkspaceResponse,
  type GameplayForcedMovementOptionDto,
} from '@asha-rulebench/protocol';

import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'four independent representative kits play through the Rust authority @live',
  async ({ page, collector }) => {
    liveScenario.setTimeout(120_000);
    collector.addNonClaim(
      'This proves interactive first-wave kit integration and authority readback; it does not claim complete games, unattended scenarios, animation, content import, or TypeScript rule execution.',
    );
    await page.goto('/');
    const workspace = page.getByLabel('Rulebench interactive combat workspace');
    const history = workspace.getByRole('list', { name: 'Combat history' });

    await activateKit(
      page,
      workspace,
      'tactical-rollover',
      'Measured Crossing',
    );
    await executeParticipantAction(workspace, 'Test Guard', 'Warded Rival');
    await expect(history).toContainText('scalarTestResolved');
    await expect(history).toContainText('base band');
    await expect(history).toContainText('final band');
    await expect(history).toContainText(/hit|surge|miss/);
    await history
      .getByText('final band', { exact: true })
      .last()
      .scrollIntoViewIfNeeded();
    await collector.milestone('tactical scalar bands', {
      screenshot: true,
      layerSnapshot: {
        sourceSet: 'tactical-rollover',
        authorityReadback: ['scalarTestResolved', 'base band', 'final band'],
      },
    });

    await restartScenario(page, workspace, 'Measured Crossing');
    await workspace.getByRole('button', { name: /^Pressure Sweep/ }).click();
    await workspace
      .getByRole('button', { name: /^Area cell-/ })
      .first()
      .click();
    await workspace
      .getByRole('button', { name: /^Use Pressure Sweep/ })
      .click();
    await declineReactionIfOpen(workspace);
    await expect(history).toContainText('areaTargetsDerived');
    await expect(history).toContainText('included cells');
    await expect(history).toContainText('included participants');
    await assertStaleAreaCommandIsAtomic(page);

    await activateKit(page, workspace, 'context-tactics', 'Overlook Crossing');
    await executeParticipantAction(workspace, 'Measured Contact', 'Keeper');
    await chooseFirstReactionIfOpen(workspace);
    await expect(history).toContainText('activationBudgetSpent');
    await expect(history).toContainText('damagePacketApplied');
    await expect(history).toContainText('damage part kinetic');
    await expect(history).toContainText('damage part strain');
    await executeParticipantAction(workspace, 'Center Line', 'Coordinator');
    await expect(history).toContainText('effectApplied');
    await workspace.getByRole('button', { name: /End turn/ }).click();
    await executeParticipantAction(workspace, 'Probe Ward', 'Coordinator');
    await expect(history).toContainText('scalarTestResolved');
    await openCharacter(workspace, 'Coordinator');
    const coordinator = page.getByRole('dialog', { name: 'Coordinator' });
    await expect(coordinator).toContainText('Activation budgets');
    await expect(coordinator).toContainText('Tempo');
    await expect(coordinator).toContainText('Response');
    await coordinator
      .getByText('Activation budgets', { exact: true })
      .scrollIntoViewIfNeeded();
    await collector.milestone('context budgets and typed damage', {
      screenshot: true,
      layerSnapshot: {
        sourceSet: 'context-tactics',
        authorityReadback: [
          'activationBudgetSpent',
          'damagePacketApplied',
          'typed parts',
        ],
      },
    });
    await coordinator.getByRole('button', { name: 'Close' }).click();

    await activateKit(page, workspace, 'multi-axis-pool', 'Signal Crossing');
    await executeParticipantAction(
      workspace,
      'Prime Trailing Signal',
      'Operator',
    );
    await openCharacter(workspace, 'Operator');
    const operator = page.getByRole('dialog', { name: 'Operator' });
    await expect(operator).toContainText('Active effects');
    await expect(operator).toContainText('Trailing Signal');
    await operator.getByRole('button', { name: 'Close' }).click();

    await executeParticipantAction(
      workspace,
      'Cross Signals — Plain Instrument',
      'Reader',
    );
    await expect(history).toContainText('heterogeneousPoolResolved');
    await expect(history).toContainText('typed evidence');
    await expect(history).toContainText('raw axes');
    await expect(history).toContainText('automatic axes');
    await expect(history).toContainText('net axes');
    await expect(history).toContainText('cancellation');
    const ledger = history.getByRole('list', {
      name: 'Authority contribution ledger',
    });
    await expect(ledger).toContainText('Trailing Signal');
    await expect(ledger).toContainText('Plain Instrument');
    await expect(ledger).toContainText('applied');
    await ledger.scrollIntoViewIfNeeded();
    await collector.milestone('multi-axis pool authority ledger', {
      screenshot: true,
      layerSnapshot: {
        sourceSet: 'multi-axis-pool',
        authorityReadback: [
          'typed heterogeneous evidence',
          'pool axes and cancellations',
          'item and active-effect contributions',
        ],
      },
    });

    await activateKit(
      page,
      workspace,
      'ruleweaver-tactics',
      'Crosswind Outpost',
      'Field Shaper',
    );
    await executeParticipantAction(
      workspace,
      'Raise Pressure Field',
      'Field Shaper',
    );
    await expect(history).toContainText('spatialSourceCreated');
    await openCharacter(workspace, 'Field Shaper');
    const fieldShaper = page.getByRole('dialog', { name: 'Field Shaper' });
    await expect(fieldShaper).toContainText('Spatial sources');
    await expect(fieldShaper).toContainText('Pressure Field');
    await expect(fieldShaper).toContainText('Triggers:');
    await fieldShaper.getByRole('button', { name: 'Close' }).click();

    await restartScenario(
      page,
      workspace,
      'Crosswind Outpost',
      'Field Shaper',
    );
    await workspace.getByRole('button', { name: /^Crosswind Sweep/ }).click();
    await workspace
      .getByRole('button', { name: /^Area cell-/ })
      .first()
      .click();
    await workspace
      .getByRole('button', { name: /^Use Crosswind Sweep/ })
      .click();
    await expect(history).toContainText('areaTargetsDerived');
    await expect(history).toContainText('scalarTestResolved');

    await restartScenario(
      page,
      workspace,
      'Crosswind Outpost',
      'Field Shaper',
    );
    await executeParticipantAction(workspace, 'Disrupt', 'Dust Runner');
    await expect(history).toContainText('effectApplied');
    await advanceToActor(page, workspace, 'runner');
    await workspace.getByRole('button', { name: /End turn/ }).click();
    await expect(history).toContainText('effectSaveResolved');

    await restartScenario(
      page,
      workspace,
      'Crosswind Outpost',
      'Pathfinder',
    );
    await executeParticipantAction(workspace, 'Redirect', 'Line Sentry');
    const forcedMovement = workspace.getByRole('group', {
      name: 'Move Line Sentry',
    });
    await expect(forcedMovement).toBeVisible();
    const staleForcedMovementOption = await firstForcedMovementOption(page);
    await forcedMovement
      .getByRole('button', { name: /^Move Line Sentry/ })
      .first()
      .click();
    await expect(history).toContainText('movementTransition');
    await expect(history).toContainText('Slide');
    await assertStaleForcedMovementIsAtomic(
      page,
      staleForcedMovementOption,
    );
    await history
      .getByText(/movementTransition:/)
      .last()
      .scrollIntoViewIfNeeded();
    await collector.milestone('ruleweaver tactics interactive authority', {
      screenshot: true,
      layerSnapshot: {
        sourceSet: 'ruleweaver-tactics',
        authorityReadback: [
          'spatial source lifecycle',
          'per-target area resolution',
          'human-choice forced movement',
        ],
      },
    });

    await page.setViewportSize({ width: 390, height: 844 });
    await openCharacter(workspace, 'Line Sentry');
    const reader = page.getByRole('dialog', { name: 'Line Sentry' });
    await expect(reader).toContainText('Resources');
    await expect(reader).toContainText('Focus');
    await expect(reader).toContainText('Activation budgets');
    await expect(reader).toContainText('Position');
    await collector.milestone('narrow representative character readback', {
      screenshot: true,
    });
    await reader.getByRole('button', { name: 'Close' }).click();
  },
);

async function activateKit(
  page: Page,
  workspace: Locator,
  sourceSetId: string,
  scenarioName: string,
  startingActor?: string,
): Promise<void> {
  await openMenuItem(
    page,
    workspace,
    'Play',
    'Choose Ruleset and Content Packs…',
  );
  const playDialog = page.getByRole('dialog', { name: 'Choose play content' });
  await playDialog
    .getByLabel('Configured source set')
    .selectOption(sourceSetId);
  const contentPacks = playDialog.getByRole('checkbox');
  await expect(contentPacks.first()).toBeVisible({ timeout: 30_000 });
  const contentPackCount = await contentPacks.count();
  for (let index = 0; index < contentPackCount; index += 1) {
    await contentPacks.nth(index).check();
  }
  await expect(playDialog).toContainText('Compatible PlayBundle');
  await playDialog
    .getByRole('button', { name: 'Compile selected PlayBundle' })
    .click();
  await expect(playDialog.getByText('candidate', { exact: true })).toBeVisible({
    timeout: 30_000,
  });
  await playDialog
    .getByRole('button', { name: 'Activate compiled PlayBundle' })
    .click();

  const scenarioDialog = page.getByRole('dialog', { name: 'Scenario setup' });
  await scenarioDialog
    .getByRole('button', { name: new RegExp(scenarioName) })
    .click();
  if (startingActor !== undefined) {
    await scenarioDialog
      .getByRole('combobox', { name: 'Starting actor' })
      .selectOption({ label: startingActor });
  }
  await scenarioDialog
    .getByRole('button', { name: 'Validate and start Scenario' })
    .click();
  await expect(scenarioDialog).not.toBeVisible();
}

async function restartScenario(
  page: Page,
  workspace: Locator,
  scenarioName: string,
  startingActor?: string,
): Promise<void> {
  await openMenuItem(page, workspace, 'Session', 'Start new Scenario…');
  const scenarioDialog = page.getByRole('dialog', { name: 'Scenario setup' });
  await scenarioDialog
    .getByRole('button', { name: new RegExp(scenarioName) })
    .click();
  if (startingActor !== undefined) {
    await scenarioDialog
      .getByRole('combobox', { name: 'Starting actor' })
      .selectOption({ label: startingActor });
  }
  await scenarioDialog
    .getByRole('button', { name: 'Validate and start Scenario' })
    .click();
  await expect(scenarioDialog).not.toBeVisible();
}

async function executeParticipantAction(
  workspace: Locator,
  actionName: string,
  targetName: string,
): Promise<void> {
  await workspace
    .getByRole('button', { name: new RegExp(`^${escapePattern(actionName)}`) })
    .click();
  await workspace
    .getByRole('button', {
      name: new RegExp(`Target ${escapePattern(targetName)}`),
    })
    .click();
  await workspace
    .getByRole('button', {
      name: new RegExp(`^Use ${escapePattern(actionName)}`),
    })
    .click();
}

async function openCharacter(
  workspace: Locator,
  participantName: string,
): Promise<void> {
  await workspace
    .getByRole('list', { name: 'Session participants' })
    .getByRole('button', {
      name: new RegExp(`View ${escapePattern(participantName)} character`),
    })
    .click();
}

async function advanceToActor(
  page: Page,
  workspace: Locator,
  actorId: string,
): Promise<void> {
  for (let step = 0; step < 8; step += 1) {
    const response = decodePlayWorkspaceResponse(
      await (await page.request.get('/api/play')).json(),
    );
    if (response.gameplay?.actorId === actorId) return;
    const previousActorId = response.gameplay?.actorId;
    await workspace.getByRole('button', { name: /End turn/ }).click();
    await expect
      .poll(async () => {
        const next = decodePlayWorkspaceResponse(
          await (await page.request.get('/api/play')).json(),
        );
        return next.gameplay?.actorId;
      })
      .not.toBe(previousActorId);
  }
  throw new Error(`did not advance to actor ${actorId}`);
}

async function firstForcedMovementOption(
  page: Page,
): Promise<GameplayForcedMovementOptionDto> {
  const response = decodePlayWorkspaceResponse(
    await (await page.request.get('/api/play')).json(),
  );
  const option = response.gameplay?.pendingForcedMovement?.options[0];
  expect(option).toBeDefined();
  if (option === undefined) {
    throw new Error('authority did not expose a forced-movement option');
  }
  return option;
}

async function chooseFirstReactionIfOpen(workspace: Locator): Promise<void> {
  const response = workspace
    .getByRole('button', { name: /· reduce \d+/ })
    .first();
  const opened = await response
    .waitFor({ state: 'visible', timeout: 5_000 })
    .then(() => true)
    .catch(() => false);
  if (opened) await response.click();
}

async function declineReactionIfOpen(workspace: Locator): Promise<void> {
  const decline = workspace.getByRole('button', { name: 'Decline reaction' });
  const opened = await decline
    .waitFor({ state: 'visible', timeout: 2_000 })
    .then(() => true)
    .catch(() => false);
  if (opened) await decline.click();
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

function escapePattern(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

async function assertStaleAreaCommandIsAtomic(page: Page): Promise<void> {
  const beforeResponse = await page.request.get('/api/play');
  expect(beforeResponse.ok()).toBe(true);
  const before = decodePlayWorkspaceResponse(await beforeResponse.json());
  const beforeGameplay = before.gameplay;
  expect(beforeGameplay).not.toBeNull();
  if (beforeGameplay === null) return;

  const areaAction = beforeGameplay.actions.find(
    (action) => action.label === 'Pressure Sweep',
  );
  expect(areaAction).toBeDefined();
  const anchorCellId = areaAction?.options.areaIds[0];
  expect(anchorCellId).toBeDefined();
  expect(beforeGameplay.stateRevision).toBeGreaterThan(0);
  if (areaAction === undefined || anchorCellId === undefined) return;

  const staleResponse = await page.request.post('/api/session/command', {
    data: {
      expectedRevision: beforeGameplay.stateRevision - 1,
      actionId: areaAction.definitionId,
      actorId: beforeGameplay.actorId,
      targetIds: [anchorCellId],
      itemBinding: areaAction.itemBinding,
    },
  });
  expect(staleResponse.ok()).toBe(true);
  const rejected = decodePlayWorkspaceResponse(await staleResponse.json());
  expect(rejected.ok).toBe(false);
  expect(rejected.gameplay?.lastResult).toEqual(
    expect.objectContaining({
      status: 'rejected',
      code: 'RPG_AREA_OPTION_STALE',
    }),
  );

  const afterResponse = await page.request.get('/api/play');
  expect(afterResponse.ok()).toBe(true);
  const after = decodePlayWorkspaceResponse(await afterResponse.json());
  const afterGameplay = after.gameplay;
  expect(afterGameplay).not.toBeNull();
  if (afterGameplay === null) return;

  expect(afterGameplay.stateRevision).toBe(beforeGameplay.stateRevision);
  expect(afterGameplay.acceptedRandomValues).toBe(
    beforeGameplay.acceptedRandomValues,
  );
  expect(afterGameplay.log).toHaveLength(beforeGameplay.log.length);
  expect(afterGameplay.archive.stateRevision).toBe(
    beforeGameplay.archive.stateRevision,
  );
  expect(afterGameplay.archive.acceptedRandomPosition).toBe(
    beforeGameplay.archive.acceptedRandomPosition,
  );
  expect(afterGameplay.archive.stateHash).toBe(beforeGameplay.archive.stateHash);
  expect(afterGameplay.archive.replayEntries).toHaveLength(
    beforeGameplay.archive.replayEntries.length,
  );
}

async function assertStaleForcedMovementIsAtomic(
  page: Page,
  option: GameplayForcedMovementOptionDto,
): Promise<void> {
  const beforeResponse = await page.request.get('/api/play');
  expect(beforeResponse.ok()).toBe(true);
  const before = decodePlayWorkspaceResponse(await beforeResponse.json());
  const beforeGameplay = before.gameplay;
  expect(beforeGameplay).not.toBeNull();
  if (beforeGameplay === null) return;

  const staleResponse = await page.request.post(
    '/api/session/forced-movement',
    { data: { option } },
  );
  expect(staleResponse.ok()).toBe(true);
  const rejected = decodePlayWorkspaceResponse(await staleResponse.json());
  expect(rejected.ok).toBe(false);
  expect(rejected.gameplay?.lastResult).toEqual(
    expect.objectContaining({
      status: 'rejected',
      code: 'RPG_FORCED_MOVEMENT_PENDING_ABSENT',
    }),
  );

  const afterResponse = await page.request.get('/api/play');
  expect(afterResponse.ok()).toBe(true);
  const after = decodePlayWorkspaceResponse(await afterResponse.json());
  const afterGameplay = after.gameplay;
  expect(afterGameplay).not.toBeNull();
  if (afterGameplay === null) return;

  expect(afterGameplay.stateRevision).toBe(beforeGameplay.stateRevision);
  expect(afterGameplay.acceptedRandomValues).toBe(
    beforeGameplay.acceptedRandomValues,
  );
  expect(afterGameplay.log).toHaveLength(beforeGameplay.log.length);
  expect(afterGameplay.archive.stateRevision).toBe(
    beforeGameplay.archive.stateRevision,
  );
  expect(afterGameplay.archive.acceptedRandomPosition).toBe(
    beforeGameplay.archive.acceptedRandomPosition,
  );
  expect(afterGameplay.archive.stateHash).toBe(beforeGameplay.archive.stateHash);
  expect(afterGameplay.archive.replayEntries).toHaveLength(
    beforeGameplay.archive.replayEntries.length,
  );
}
