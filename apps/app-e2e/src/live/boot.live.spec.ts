import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'inspect explicit compilation and atomic activation @live-artifact',
  async ({ page, collector, liveBaseUrl }) => {
    collector.addNonClaim(
      'This compiler scenario does not claim gameplay execution, persistent sessions, migration, replay, or exhaustive cross-product certification.',
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
    await expect(workspace).toContainText('Signal Flare');
    await expect(workspace).toContainText('rulebench.signal-flare');
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
    await expect(
      workspace.getByRole('heading', { name: 'Compiled ruleset active' }),
    ).toBeVisible();
    await expect(workspace).toContainText('Activation revision 1');
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
