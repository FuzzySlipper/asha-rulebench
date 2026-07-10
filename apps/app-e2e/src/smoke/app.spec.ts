import { expect, test } from '@playwright/test';

test('boots the rulebench shell', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'ASHA Rulebench', exact: true })).toBeVisible();
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

  await expect(page.getByRole('button', { name: 'Hexing Bolt Hit Accepted hit · roll-stream:17,5', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /Hexing Bolt Miss/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Hexing Bolt Self Target Rejected/ })).toBeVisible();

  await page.getByRole('button', { name: /Hexing Bolt Miss/ }).click();

  await expect(page.getByText('Hexing Bolt Miss · roll-stream:2,5')).toBeVisible();
  await expect(page.getByLabel('DomainEvents timeline')).toContainText('AttackRolled');
  await expect(page.getByLabel('Rule trace')).toContainText('Miss branch selected');
  await expect(page.getByLabel('Final state')).toContainText('Attack missed; no authority state changed.');

  await page.getByRole('button', { name: /Hexing Bolt Self Target Rejected/ }).click();

  await expect(page.getByText('Hexing Bolt Self Target Rejected · roll-stream:17,5')).toBeVisible();
  await expect(page.getByLabel('Selected action')).toContainText('Rejected: Target is not hostile.');
  await expect(page.getByLabel('Rule trace')).toContainText('Intent rejected');
  await expect(page.getByLabel('Final state')).toContainText('No authority state changed; intent rejected.');
});
