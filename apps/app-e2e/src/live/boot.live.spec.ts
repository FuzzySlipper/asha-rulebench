import { expect, liveScenario } from './support/live-scenario';

liveScenario('boot live evidence @live', async ({ page, collector, liveBaseUrl }) => {
  collector.addNonClaim('This boot scenario proves the Rulebench shell renders; it does not prove live Rust-backed resolution, backend correctness, accessibility coverage, or performance.');

  await page.goto(liveBaseUrl);
  await expect(page.getByRole('heading', { name: 'ASHA Rulebench' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Content Packs' })).toBeVisible();
  await page.getByRole('button', { name: /pack.error@1.0.0/ }).click();
  await expect(page.getByLabel('Selected content pack review')).toContainText('missingContentPackDependency');
  await collector.milestone('content diagnostics rendered', {
    screenshot: true,
    layerSnapshot: {
      selectedPack: await page.getByLabel('Selected content pack review').innerText(),
      validation: await page.getByLabel('Content validation review').innerText(),
    },
  });
  await collector.milestone('shell rendered', {
    screenshot: true,
    layerSnapshot: {
      route: page.url(),
      visibleHeading: await page.getByRole('heading').first().innerText(),
    },
  });
});
