import type { Locator, Page } from '@playwright/test';

import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'three independent representative kits play through the Rust authority @live',
  async ({ page, collector }) => {
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

    await page.setViewportSize({ width: 390, height: 844 });
    await openCharacter(workspace, 'Reader');
    const reader = page.getByRole('dialog', { name: 'Reader' });
    await expect(reader).toContainText('Resources');
    await expect(reader).toContainText('Charge');
    await expect(reader).toContainText('Reserve');
    await expect(reader).toContainText('Activation budgets');
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
  const contentPack = playDialog.getByRole('checkbox').first();
  await expect(contentPack).toBeVisible({ timeout: 30_000 });
  await contentPack.check();
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
  await scenarioDialog
    .getByRole('button', { name: 'Validate and start Scenario' })
    .click();
  await expect(scenarioDialog).not.toBeVisible();
}

async function restartScenario(
  page: Page,
  workspace: Locator,
  scenarioName: string,
): Promise<void> {
  await openMenuItem(page, workspace, 'Session', 'Start new Scenario…');
  const scenarioDialog = page.getByRole('dialog', { name: 'Scenario setup' });
  await scenarioDialog
    .getByRole('button', { name: new RegExp(scenarioName) })
    .click();
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
