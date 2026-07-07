import { expect, liveScenario } from './support/live-scenario';

liveScenario('boot live evidence @live', async ({ page, collector, liveBaseUrl }) => {
  collector.addNonClaim('This boot scenario proves the shell renders; it does not prove product workflows, backend correctness, accessibility coverage, or performance.');

  await page.goto(liveBaseUrl);
  await expect(page.getByRole('heading', { name: 'UI Pattern Bootstrap' })).toBeVisible();
  await collector.milestone('shell rendered', {
    screenshot: true,
    layerSnapshot: {
      route: page.url(),
      visibleHeading: await page.getByRole('heading').first().innerText(),
    },
  });
});
