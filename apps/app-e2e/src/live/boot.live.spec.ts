import type { Locator } from '@playwright/test';
import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'inspect explicit compilation and atomic activation @live-artifact',
  async ({ page, collector, liveBaseUrl }) => {
    collector.addNonClaim(
      'This scenario proves one fresh gameplay session and does not claim persistence across activation, replay, derivation, migration, or exhaustive cross-product certification.',
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

    await workspace
      .getByRole('button', { name: 'Compile explicit manifest' })
      .click();
    await expect(
      workspace.getByRole('heading', { name: 'Compiled candidate ready' }),
    ).toBeVisible();
    await expect(workspace).toContainText('Tactical Advance');
    await expect(workspace).toContainText('Arc Lash');
    await expect(workspace).toContainText('Wardbreaker Volley');
    await expect(workspace).toContainText('Source');
    await expect(workspace).toContainText('Semantic');
    await expect(workspace).toContainText('Presentation');
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
    await expect(workspace).toContainText('Activation revision 1');
    await collector.milestone('accepted artifact atomically active', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    await actionCard(workspace, 'Tactical Advance')
      .getByRole('button', { name: 'Select action' })
      .click();
    await workspace
      .getByRole('button', { name: 'Submit typed intent' })
      .click();
    await expect(workspace).toContainText('Revision 1 · actor hero');
    await expect(workspace).toContainText('Position (2, 0)');

    await actionCard(workspace, 'Arc Lash')
      .getByRole('button', { name: 'Select action' })
      .click();
    await workspace.getByLabel('Random evidence').fill('10, 3, 4');
    await workspace
      .getByRole('button', { name: 'Submit typed intent' })
      .click();
    await expect(workspace).toContainText('Revision 2 · actor hero');
    await expect(workspace).toContainText('focus 1/2');

    await actionCard(workspace, 'Wardbreaker Volley')
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
    await workspace.getByLabel('Random evidence').fill('1, 2, 3, 4, 1');
    await workspace.getByRole('button', { name: /Raise ward/ }).click();
    await expect(workspace).toContainText('Revision 3 · actor hero');
    await expect(workspace).toContainText('focus 0/2');
    await expect(workspace).toContainText('Random consumed: 5');
    await expect(workspace).toContainText('reactionResolved:');
    await collector.milestone('three authority commands persist and react', {
      screenshot: true,
      layerSnapshot: { visibleState: await workspace.innerText() },
    });

    const activeArtifact = workspace
      .getByText('Active artifact', { exact: true })
      .locator('..')
      .locator('dd');
    const activeArtifactId = await activeArtifact.innerText();
    await workspace
      .getByRole('button', { name: 'Use Invalid missing support' })
      .click();
    await workspace
      .getByRole('button', { name: 'Compile explicit manifest' })
      .click();
    await expect(workspace).toContainText(
      'RULESET_DEFINITION_REFERENCE_MISSING',
    );
    await expect(workspace).toContainText('Package: rulebench.field-manual');
    await expect(workspace).toContainText('Definition: rulebench.arc-lash');
    await expect(workspace).toContainText(
      'Source: packages/rulebench-field-manual.ts#rulebench.arc-lash',
    );
    await expect(
      workspace.getByRole('heading', { name: 'Compiled ruleset active' }),
    ).toBeVisible();
    await expect(workspace).toContainText('Activation revision 1');
    await expect(workspace).toContainText('Revision 3 · actor hero');
    await expect(activeArtifact).toHaveText(activeArtifactId);
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

function actionCard(workspace: Locator, name: string) {
  return workspace.getByText(name, { exact: true }).locator('..');
}
