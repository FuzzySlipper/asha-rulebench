import { expect, test } from '@playwright/test';
import type { Locator } from '@playwright/test';

test('compiles, inspects, and atomically activates the explicit ruleset @gate', async ({
  page,
}) => {
  await page.goto('/');

  const workspace = page.getByLabel('Rulebench ruleset workspace');
  await expect(workspace).toBeVisible();
  await expect(
    workspace.getByRole('heading', { name: 'No compiled ruleset active' }),
  ).toBeVisible();
  await expect(workspace.getByText('none', { exact: true })).toBeVisible();
  await expect(workspace).toContainText('Gameplay session: inactive');

  const menubar = workspace.getByRole('menubar', {
    name: 'Rulebench application menu',
  });
  await menubar.getByRole('menuitem', { name: 'Ruleset' }).click();
  await expect(
    page
      .getByRole('menu', { name: 'Ruleset' })
      .getByRole('menuitem', { name: 'Explicit compiler workspace' }),
  ).toHaveAttribute('aria-disabled', 'true');

  await workspace
    .getByRole('button', { name: 'Compile explicit manifest' })
    .click();
  await expect(
    workspace.getByRole('heading', { name: 'Compiled candidate ready' }),
  ).toBeVisible();
  await expect(workspace).toContainText('Rust validation accepted');
  await expect(workspace).toContainText('rulebench.fresh-start@1.0.0:fnv1a64:');
  await expect(workspace).toContainText('Tactical Advance');
  await expect(workspace).toContainText('Arc Lash');
  await expect(workspace).toContainText('Arc Lash: Stormfront');
  await expect(workspace).toContainText('Wardbreaker Volley');
  await expect(workspace).toContainText('catalog.damage.storm');
  await expect(workspace).toContainText('6 lock edges');
  await expect(workspace).toContainText('operation.damage@1');
  await expect(workspace).toContainText('capability.vitality@1');
  await expect(workspace).toContainText('operation.openReaction@1');
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
  await expect(workspace).toContainText('semantic · targets.maximumRange');
  await expect(workspace).toContainText('presentation · label');

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
  await expect(workspace).toContainText('Revision 0 · actor hero');
  await expect(workspace).toContainText('Candidates: none at this revision');
  const arcLashPlan = actionCard(workspace, 'rulebench.arc-lash');
  await expect(arcLashPlan).toContainText('always: attackCheck 1d20');
  await expect(arcLashPlan).toContainText(
    'if check hit and predicate true: formulaDice 2d6',
  );
  await expect(arcLashPlan).toContainText(
    'if check hit and predicate false: formulaDice 1d6',
  );
  await expect(arcLashPlan).not.toContainText(
    'attackCheck 1d20, formulaDice 2d6, formulaDice 1d6',
  );

  await actionCard(workspace, 'rulebench.tactical-advance')
    .getByRole('button', { name: 'Select action' })
    .click();
  await workspace.getByRole('button', { name: 'Submit typed intent' }).click();
  await expect(workspace).toContainText('Revision 1 · actor hero');
  await expect(workspace).toContainText('hero · ally');
  await expect(workspace).toContainText('Position (2, 0)');
  await expect(workspace).toContainText('exposed -2 (2 turns, guard-penalty)');

  await actionCard(workspace, 'rulebench.arc-lash')
    .getByRole('button', { name: 'Select action' })
    .click();
  await workspace.getByLabel('Random evidence').fill('10, 3, 4');
  await workspace.getByRole('button', { name: 'Submit typed intent' }).click();
  await expect(workspace).toContainText('Revision 2 · actor hero');
  await expect(workspace).toContainText('focus 1/2');
  await expect(workspace).toContainText('Random consumed: 3');
  await expect(workspace).toContainText('damageApplied:');

  await actionCard(workspace, 'rulebench.wardbreaker-volley')
    .getByRole('button', { name: 'Select action' })
    .click();
  await workspace.getByLabel('Random evidence').fill('');
  await workspace.getByRole('button', { name: 'Submit typed intent' }).click();
  await expect(workspace).toContainText(
    'Reaction pending: reaction.raise-ward',
  );
  await expect(workspace).toContainText('remains staged at revision 2');
  await expect(workspace).toContainText('focus 1/2');
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
  await archive.getByTestId('replay-records').click();
  await expect(archive).toContainText('Rust replay verified 4 record(s)');
  await expect(workspace).toContainText('Revision 3 · actor hero');
  await expect(workspace).toContainText('focus 0/2');
  const activeArtifact = workspace
    .getByText('Active artifact', { exact: true })
    .locator('..')
    .locator('dd');
  const activeArtifactId = await activeArtifact.innerText();

  await workspace
    .getByRole('button', { name: 'Use Field manual 1.1 candidate' })
    .click();
  await workspace
    .getByRole('button', { name: 'Compile explicit manifest' })
    .click();
  await expect(workspace).toContainText('Pre-activation upgrade impact');
  await expect(workspace).toContainText(
    'Candidate compared with active runtime truth',
  );
  await expect(workspace).toContainText('rulebench.field-manual: 1.0.0');
  await expect(workspace).toContainText('rulebench.arc-lash-stormfront');
  await expect(workspace).toContainText('changed derived descendant');
  await expect(workspace).toContainText(
    'primary base identity or fingerprint changed',
  );
  await expect(
    workspace
      .getByText('Activation revision', { exact: true })
      .locator('..')
      .locator('dd'),
  ).toHaveText('1');
  await expect(workspace).toContainText('Revision 3 · actor hero');
  await expect(activeArtifact).toHaveText(activeArtifactId);
  const candidateArtifact = workspace
    .getByText('Artifact', { exact: true })
    .locator('..')
    .locator('dd');
  const candidateArtifactId = await candidateArtifact.innerText();
  expect(candidateArtifactId).toContain('rulebench.fresh-start@1.1.0');

  await workspace
    .getByRole('button', { name: 'Activate accepted artifact' })
    .click();
  await expect(workspace).toContainText('Activation revision 2');
  await expect(workspace).toContainText('Revision 0 · actor hero');
  await expect(activeArtifact).toHaveText(candidateArtifactId);

  await workspace
    .getByRole('button', { name: 'Use Invalid missing support' })
    .click();
  await expect(workspace).toContainText(
    'Selected source: Invalid missing support',
  );
  await workspace
    .getByRole('button', { name: 'Compile explicit manifest' })
    .click();
  await expect(workspace).toContainText('RULESET_DEFINITION_REFERENCE_MISSING');
  await expect(workspace).toContainText('Package: rulebench.field-manual');
  await expect(workspace).toContainText(
    'Definition: rulebench.arc-lash',
  );
  await expect(workspace).toContainText(
    'Source: packages/rulebench-field-manual.ts#rulebench.arc-lash',
  );
  await expect(
    workspace.getByRole('heading', { name: 'Compiled ruleset active' }),
  ).toBeVisible();
  await expect(workspace).toContainText('Activation revision 2');
  await expect(workspace).toContainText('Revision 0 · actor hero');
  await expect(activeArtifact).toHaveText(candidateArtifactId);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(workspace).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
});

function actionCard(workspace: Locator, actionId: string) {
  return workspace
    .locator('li.action-card')
    .filter({ hasText: actionId })
    .filter({ hasNotText: `${actionId}-` });
}
