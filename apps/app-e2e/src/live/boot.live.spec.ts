import type { Locator } from '@playwright/test';
import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'inspect explicit compilation and atomic activation @live-artifact',
  async ({ page, collector, liveBaseUrl }) => {
    collector.addNonClaim(
      'This scenario proves one in-memory portable checkpoint/archive and Rust replay path. It does not claim process-restart storage, migration execution, or exhaustive cross-product certification.',
    );

    await page.goto(liveBaseUrl);
    const workspace = page.getByLabel('Rulebench ruleset workspace');
    await expect(workspace).toBeVisible();
    await expect(
      workspace.getByRole('heading', { name: 'No compiled ruleset active' }),
    ).toBeVisible();
    await collector.milestone('explicit compiler starts inactive', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    const menubar = workspace.getByRole('menubar', {
      name: 'Rulebench application menu',
    });
    await menubar.getByRole('menuitem', { name: 'Ruleset' }).click();
    await page
      .getByRole('menu', { name: 'Ruleset' })
      .getByRole('menuitem', { name: 'Load ruleset root…' })
      .click();
    await expect(
      workspace.getByRole('textbox', { name: 'Ruleset root' }),
    ).toBeFocused();
    await selectRulesetRoot(workspace, 'examples/rulesets/field-manual');
    await workspace
      .getByRole('button', { name: 'Load ruleset candidate' })
      .click();
    await expect(
      workspace.getByRole('heading', { name: 'Compiled candidate ready' }),
    ).toBeVisible();
    await expect(workspace).toContainText('Tactical Advance');
    await expect(workspace).toContainText('Arc Lash');
    await expect(workspace).toContainText('Arc Lash: Stormfront');
    await expect(workspace).toContainText('Wardbreaker Volley');
    await expect(workspace).toContainText('Source');
    await expect(workspace).toContainText('Semantic');
    await expect(workspace).toContainText('Presentation');
    await expect(workspace).toContainText('Materialization provenance');
    await expect(workspace).toContainText('rulebench.double-range');
    await expect(workspace).toContainText('rulebench.extend-range');
    await expect(workspace).toContainText('rulebench.stormfront-balance@1.0.0');
    await expect(workspace).toContainText(
      'rulebench.stormfront-presentation@1.0.0',
    );
    await collector.milestone('Rust accepted closed artifact', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    await workspace
      .getByRole('button', { name: 'Activate accepted artifact' })
      .click();
    await expect(
      workspace.getByRole('heading', { name: 'Compiled ruleset active' }),
    ).toBeVisible();
    await expect(
      workspace
        .getByText('Activation revision', { exact: true })
        .locator('..')
        .locator('dd'),
    ).toHaveText('1');
    const arcLashPlan = actionCard(workspace, 'rulebench.arc-lash');
    await expect(arcLashPlan).toContainText('always: attackCheck 1d20');
    await expect(arcLashPlan).toContainText(
      'if check hit and predicate true: formulaDice 2d6',
    );
    await expect(arcLashPlan).toContainText(
      'if check hit and predicate false: formulaDice 1d6',
    );
    await collector.milestone('accepted artifact atomically active', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    await actionCard(workspace, 'rulebench.tactical-advance')
      .getByRole('button', { name: 'Select action' })
      .click();
    await workspace
      .getByRole('button', { name: 'Submit typed intent' })
      .click();
    await expect(workspace).toContainText('Revision 1 · actor hero');
    await expect(workspace).toContainText('Position (2, 0)');

    await actionCard(workspace, 'rulebench.arc-lash')
      .getByRole('button', { name: 'Select action' })
      .click();
    await workspace.getByLabel('Random evidence').fill('10, 3, 4');
    await workspace
      .getByRole('button', { name: 'Submit typed intent' })
      .click();
    await expect(workspace).toContainText('Revision 2 · actor hero');
    await expect(workspace).toContainText('focus 1/2');

    await actionCard(workspace, 'rulebench.wardbreaker-volley')
      .getByRole('button', { name: 'Select action' })
      .click();
    await workspace.getByLabel('Random evidence').fill('');
    await workspace
      .getByRole('button', { name: 'Submit typed intent' })
      .click();
    await expect(workspace).toContainText(
      'Reaction pending: reaction.raise-ward',
    );
    await expect(workspace).toContainText('remains staged at revision 2');
    const archive = page.getByTestId('replay-archive-panel');
    await expect(archive).toContainText('3 stored replay record(s)');
    await expect(archive).toContainText('awaitingReaction reaction.raise-ward');
    await archive.getByTestId('restore-checkpoint').click();
    await expect(archive).toContainText('checkpointRestored');
    await expect(workspace).toContainText(
      'Reaction pending: reaction.raise-ward',
    );
    await workspace.getByLabel('Random evidence').fill('1, 2, 3, 4, 1');
    await workspace.getByRole('button', { name: /Raise ward/ }).click();
    await expect(workspace).toContainText('Revision 3 · actor hero');
    await expect(workspace).toContainText('focus 0/2');
    await expect(workspace).toContainText('Random consumed: 5');
    await expect(workspace).toContainText('reactionResolved:');
    await expect(archive).toContainText('4 stored replay record(s)');
    await expect(archive).toContainText('asha.rpg.session.checkpoint@1');
    await expect(archive).toContainText('awaitingReaction');
    await archive.getByTestId('replay-records').click();
    await expect(archive).toContainText('verified');
    await expect(archive).toContainText('Rust replay verified 4 record(s)');
    await expect(workspace).toContainText('Revision 3 · actor hero');
    await expect(workspace).toContainText('focus 0/2');
    await collector.milestone(
      'checkpoint restores and replay verifies in Rust',
      {
        screenshot: true,
        layerSnapshot: { visibleState: await workspace.innerText() },
      },
    );

    const activeArtifact = workspace
      .getByText('Active artifact', { exact: true })
      .locator('..')
      .locator('dd');
    const activeArtifactId = await activeArtifact.innerText();
    await selectRulesetRoot(workspace, 'examples/rulesets/ember-skirmish');
    await workspace
      .getByRole('button', { name: 'Load ruleset candidate' })
      .click();
    await expect(workspace).toContainText(
      'rulebench.ember-skirmish.demo@1.0.0',
    );
    await expect(workspace).toContainText('Ember Guard');
    await expect(
      workspace
        .getByText('Activation revision', { exact: true })
        .locator('..')
        .locator('dd'),
    ).toHaveText('1');
    await expect(workspace).toContainText('Revision 3 · actor hero');
    await expect(activeArtifact).toHaveText(activeArtifactId);
    const emberCandidateArtifact = workspace
      .getByText('Artifact', { exact: true })
      .locator('..')
      .locator('dd');
    const emberCandidateArtifactId = await emberCandidateArtifact.innerText();
    expect(emberCandidateArtifactId).toContain(
      'rulebench.ember-skirmish.demo@1.0.0',
    );
    await collector.milestone('independent root staged before activation', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    await workspace
      .getByRole('button', { name: 'Activate accepted artifact' })
      .click();
    await expect(activationRevision(workspace)).toHaveText('2');
    await expect(workspace).toContainText('Revision 0 · actor hero');
    await expect(activeArtifact).toHaveText(emberCandidateArtifactId);
    await expect(workspace).toContainText('Ember Guard');

    await menubar.getByRole('menuitem', { name: 'Ruleset' }).click();
    const rulesetMenu = page.getByRole('menu', { name: 'Ruleset' });
    await expect(
      rulesetMenu.getByRole('menuitem', {
        name: 'Switch to examples/rulesets/ember-skirmish',
      }),
    ).toBeVisible();
    await collector.milestone('recent roots visible in the top menu', {
      screenshot: true,
      layerSnapshot: { visibleMenu: await rulesetMenu.innerText() },
    });
    await rulesetMenu
      .getByRole('menuitem', {
        name: 'Switch to examples/rulesets/field-manual',
      })
      .click();
    await expect(workspace).toContainText('rulebench.fresh-start@1.0.0');
    await expect(workspace).toContainText('Tactical Advance');
    await expect(activeArtifact).toHaveText(emberCandidateArtifactId);
    await collector.milestone('top menu switched back to a recent root', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    await workspace
      .getByRole('button', { name: 'Activate accepted artifact' })
      .click();
    await expect(activationRevision(workspace)).toHaveText('3');
    const fieldManualArtifactId = await activeArtifact.innerText();
    await selectRulesetRoot(workspace, 'examples/rulesets/invalid-build');
    await workspace
      .getByRole('button', { name: 'Load ruleset candidate' })
      .click();
    await expect(workspace).toContainText('RULESET_WORKSPACE_BUILD_FAILED');
    await expect(workspace).toContainText('TS2322');
    await expect(workspace).toContainText('ruleset.ts#ruleset');
    await expect(activationRevision(workspace)).toHaveText('3');
    await expect(workspace).toContainText('Revision 0 · actor hero');
    await expect(activeArtifact).toHaveText(fieldManualArtifactId);
    await collector.milestone(
      'invalid TypeScript recompile preserves active artifact',
      {
        screenshot: true,
        layerSnapshot: { visibleState: await workspace.innerText() },
      },
    );

    await page.setViewportSize({ width: 390, height: 844 });
    await expect(workspace).toBeVisible();
    await collector.milestone('active artifact mobile inspection', {
      screenshot: true,
      layerSnapshot: {
        viewport: '390x844',
        visibleState: await workspace.innerText(),
      },
    });
  },
);

function actionCard(workspace: Locator, actionId: string) {
  return workspace
    .locator('li.action-card')
    .filter({ hasText: actionId })
    .filter({ hasNotText: `${actionId}-` });
}

function activationRevision(workspace: Locator): Locator {
  return workspace
    .getByText('Activation revision', { exact: true })
    .locator('..')
    .locator('dd');
}

async function selectRulesetRoot(
  workspace: Locator,
  rulesetRoot: string,
): Promise<void> {
  await workspace
    .getByRole('textbox', { name: 'Ruleset root' })
    .fill(rulesetRoot);
}
