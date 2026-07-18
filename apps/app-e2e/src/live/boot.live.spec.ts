import { expect, liveScenario } from './support/live-scenario';

liveScenario(
  'inspect the no-active-ruleset boundary @live-artifact',
  async ({ page, collector, liveBaseUrl }) => {
    collector.addNonClaim(
      'This deletion-phase scenario does not claim compilation, activation, gameplay execution, persistence, or replay.',
    );

    await page.goto(liveBaseUrl);
    const workspace = page.getByLabel('Rulebench empty workspace');
    await expect(workspace).toBeVisible();
    await expect(
      workspace.getByRole('heading', { name: 'No compiled ruleset active' }),
    ).toBeVisible();
    await collector.milestone('no active ruleset desktop', {
      screenshot: true,
      layerSnapshot: {
        visibleState: await workspace.innerText(),
      },
    });

    await page.setViewportSize({ width: 390, height: 844 });
    await expect(workspace).toBeVisible();
    await collector.milestone('no active ruleset mobile', {
      screenshot: true,
      layerSnapshot: {
        viewport: '390x844',
        visibleState: await workspace.innerText(),
      },
    });
  },
);
