import { expect, test } from '@playwright/test';

test('boots the rulebench shell', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByLabel('Rulebench panel layout')).toBeVisible();
  for (const label of [
    '1. Combat grid',
    '2. Initiative',
    '3. Application menu',
    '4. Turn status',
    '5. Evidence log',
    '6. Available actions',
    '7. Active units',
  ]) {
    await expect(page.getByRole('region', { name: label })).toBeVisible();
  }
  await page.getByRole('tab', { name: 'DomainEvents' }).click();
  await expect(page.getByRole('tabpanel')).toContainText('Accepted DomainEvents');

  await expect(page.getByRole('heading', { name: 'ASHA Rulebench', exact: true })).toBeVisible();
  const menubar = page.getByRole('menubar', { name: 'Rulebench application menu' });
  await menubar.getByRole('menuitem', { name: 'File' }).click();
  await page.getByRole('menu', { name: 'File' }).getByRole('menuitem', { name: 'Content packs' }).click();
  const contentDialog = page.getByRole('dialog', { name: 'Content packs' });
  await expect(contentDialog.getByRole('heading', { name: 'Content Packs', exact: true })).toBeVisible();
  await expect(page.getByLabel('Selected content pack review')).toContainText('pack.valid@1.0.0');
  await expect(page.getByLabel('Selected content pack review')).toContainText('fnv1a64.rulebench-content-pack.v0');
  await expect(page.getByLabel('Content validation review')).toContainText('Hexing Bolt Hit');
  await page.getByRole('button', { name: /pack.warning@1.0.0/ }).click();
  await expect(page.getByLabel('Selected content pack review')).toContainText('duplicateContentTagCanonicalized');
  await page.getByRole('button', { name: /pack.error@1.0.0/ }).click();
  await expect(page.getByLabel('Selected content pack review')).toContainText('missingContentPackDependency');
  await contentDialog.getByRole('button', { name: 'Close' }).click();

  await menubar.getByRole('menuitem', { name: 'Scenario' }).click();
  await page.getByRole('menu', { name: 'Scenario' }).getByRole('menuitem', { name: 'Scenario cases' }).click();
  await expect(page.getByRole('heading', { name: 'Combat Session' })).toBeVisible();
  await expect(page.getByRole('button', { name: '1 · Adept hits Raider Accepted hit', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /Adept misses Raider/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Adept targets themself/ })).toBeVisible();
  await expect(page.getByLabel('Combat session step')).toContainText('1 · Adept hits Raider');
  await expect(page.getByLabel('Combat log')).toContainText('DamageApplied');
  await expect(page.getByLabel('Step state review')).toContainText('Before');
  await expect(page.getByLabel('Step state review')).toContainText('Raider · 9/18 HP · rattled');
  await expect(page.getByText('Adept hits Raider · roll-stream:17,5')).toBeVisible();
  await expect(page.getByLabel('DomainEvents timeline')).toContainText('DamageApplied');
  await expect(page.getByLabel('Final state')).toContainText('Raider is damaged and rattled');

  await page.getByRole('region', { name: 'Combat session' }).getByRole('button', { name: 'Next' }).first().click();

  await expect(page.getByLabel('Combat session step')).toContainText('2 · Adept misses Raider');
  await expect(page.getByText('Adept misses Raider · roll-stream:2,5')).toBeVisible();
  await expect(page.getByLabel('DomainEvents timeline')).toContainText('AttackRolled');
  await expect(page.getByLabel('Rule trace')).toContainText('Miss branch selected');
  await expect(page.getByLabel('Final state')).toContainText('Attack missed; no authority state changed.');
  await expect(page.getByLabel('Step state review')).toContainText('Raider · 9/18 HP · rattled');

  await page.getByRole('button', { name: /Adept targets themself/ }).click();

  await expect(page.getByLabel('Combat session step')).toContainText('3 · Adept targets themself');
  await expect(page.getByText('Adept targets themself · roll-stream:17,5')).toBeVisible();
  await expect(page.getByLabel('Combat log')).toContainText('No accepted DomainEvents');
  await expect(page.getByLabel('Selected action')).toContainText('Rejected: Target is not hostile.');
  await expect(page.getByLabel('Rule trace')).toContainText('Intent rejected');
  await expect(page.getByLabel('Final state')).toContainText('No authority state changed; intent rejected.');
  await expect(page.getByLabel('Step state review')).toContainText('Raider · 9/18 HP · rattled');

  await page.getByRole('region', { name: 'Combat session' }).getByRole('button', { name: 'Previous' }).first().click();

  await expect(page.getByLabel('Combat session step')).toContainText('2 · Adept misses Raider');

  const scenarioCatalog = page.getByRole('region', { name: 'Scenario catalog' });
  await expect(scenarioCatalog.getByRole('button', { name: 'Hexing Bolt Hit Accepted hit · roll-stream:17,5', exact: true })).toBeVisible();
  await expect(scenarioCatalog.getByRole('button', { name: /Hexing Bolt Miss/ })).toBeVisible();
  await expect(scenarioCatalog.getByRole('button', { name: /Hexing Bolt Self Target Rejected/ })).toBeVisible();

  await scenarioCatalog.getByRole('button', { name: /Hexing Bolt Miss/ }).click();

  await expect(page.getByText('Hexing Bolt Miss · roll-stream:2,5')).toBeVisible();
  await expect(page.getByLabel('DomainEvents timeline')).toContainText('AttackRolled');
  await expect(page.getByLabel('Rule trace')).toContainText('Miss branch selected');
  await expect(page.getByLabel('Final state')).toContainText('Attack missed; no authority state changed.');

  await scenarioCatalog.getByRole('button', { name: /Hexing Bolt Self Target Rejected/ }).click();

  await expect(page.getByText('Hexing Bolt Self Target Rejected · roll-stream:17,5')).toBeVisible();
  await expect(page.getByLabel('Selected action')).toContainText('Rejected: Target is not hostile.');
  await expect(page.getByLabel('Rule trace')).toContainText('Intent rejected');
  await expect(page.getByLabel('Final state')).toContainText('No authority state changed; intent rejected.');
});
