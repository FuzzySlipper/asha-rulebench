import { expect, liveScenario } from './support/live-scenario';

liveScenario('boot live evidence @live', async ({ page, collector, liveBaseUrl }) => {
  const liveSessionId = `live-evidence-${Date.now()}`;
  collector.addNonClaim('This scenario proves one Rulebench fixture can be controlled through the live Rust host; it does not prove arbitrary rulesets, durable sessions, broad accessibility coverage, or performance.');

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
  const liveWorkspace = page.getByRole('region', { name: 'Live combat controls' });
  await expect(liveWorkspace.getByText('asha-rulebench.local-authority.v0')).toBeVisible();
  await liveWorkspace.getByRole('button', { name: 'Hexing Bolt Hit', exact: true }).click();
  await liveWorkspace.getByLabel('Session').fill(liveSessionId);
  await liveWorkspace.getByRole('button', { name: 'Create session' }).click();
  await liveWorkspace.getByRole('button', { name: 'Start', exact: true }).click();
  await liveWorkspace.getByRole('button', { name: 'Hexing Bolt', exact: true }).click();
  await liveWorkspace.getByRole('button', { name: /Raider · 18\/18 HP/ }).click();
  await liveWorkspace.getByRole('button', { name: 'Preflight', exact: true }).click();
  await liveWorkspace.getByRole('button', { name: 'Submit', exact: true }).click();
  await expect(liveWorkspace.getByRole('region', { name: 'Live session state' })).toContainText('Raider9/18 HP · Active');
  await collector.milestone('live rust command rendered', {
    screenshot: true,
    layerSnapshot: {
      session: await liveWorkspace.getByRole('region', { name: 'Live session state' }).innerText(),
      log: await liveWorkspace.getByRole('region', { name: 'Live combat log' }).innerText(),
      audit: await liveWorkspace.getByRole('region', { name: 'Live command audit' }).innerText(),
    },
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await collector.milestone('live rust command mobile', {
    screenshot: true,
    layerSnapshot: {
      viewport: '390x844',
      session: await liveWorkspace.getByRole('region', { name: 'Live session state' }).innerText(),
    },
  });
  await liveWorkspace.getByRole('button', { name: 'End', exact: true }).click();
  await liveWorkspace.getByRole('button', { name: 'Close', exact: true }).click();
  await expect(liveWorkspace.getByRole('region', { name: 'Live session state' })).toHaveCount(0);
  await collector.milestone('shell rendered', {
    screenshot: true,
    layerSnapshot: {
      route: page.url(),
      visibleHeading: await page.getByRole('heading').first().innerText(),
    },
  });
});
