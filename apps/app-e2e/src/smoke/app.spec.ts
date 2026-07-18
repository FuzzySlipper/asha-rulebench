import { expect, test } from '@playwright/test';

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
  await expect(workspace).toContainText('Gameplay execution unavailable');

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
  await expect(workspace).toContainText('Signal Flare');
  await expect(workspace).toContainText('rulebench.signal-flare');
  await expect(workspace).toContainText('catalog.damage.radiant');
  await expect(workspace).toContainText('3 lock edges');
  await expect(workspace).toContainText('operation.damage@1');
  await expect(workspace).toContainText('capability.vitality@1');
  await expect(workspace).toContainText('Source');
  await expect(workspace).toContainText('Semantic');
  await expect(workspace).toContainText('Presentation');

  await workspace
    .getByRole('button', { name: 'Activate accepted artifact' })
    .click();
  await expect(
    workspace.getByRole('heading', { name: 'Compiled ruleset active' }),
  ).toBeVisible();
  await expect(workspace).toContainText('Activation revision 1');
  const activeArtifact = workspace
    .getByText('Active artifact', { exact: true })
    .locator('..')
    .locator('dd');
  const activeArtifactId = await activeArtifact.innerText();

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
  await expect(workspace).toContainText('Definition: rulebench.signal-flare');
  await expect(workspace).toContainText(
    'Source: packages/rulebench-field-manual.ts#signalFlare',
  );
  await expect(
    workspace.getByRole('heading', { name: 'Compiled ruleset active' }),
  ).toBeVisible();
  await expect(workspace).toContainText('Activation revision 1');
  await expect(activeArtifact).toHaveText(activeArtifactId);
  await expect(
    workspace.getByRole('button', { name: /execute|roll|resolve/i }),
  ).toHaveCount(0);

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(workspace).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
});
