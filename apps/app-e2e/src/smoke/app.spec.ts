import { expect, test } from '@playwright/test';

test('boots the rulebench shell', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'ASHA Rulebench', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /Hexing Bolt Hit/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Hexing Bolt Miss/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /Hexing Bolt Self Target Rejected/ })).toBeVisible();
  await expect(page.getByText('Hexing Bolt Hit · roll-stream:17,5')).toBeVisible();
  await expect(page.getByLabel('Scenario board')).toBeVisible();
  await expect(page.getByLabel('Selected action')).toContainText('Hexing Bolt');
  await expect(page.getByLabel('Combatants')).toContainText('Adept');
  await expect(page.getByLabel('DomainEvents timeline')).toContainText('DamageApplied');
  await expect(page.getByLabel('Rule trace')).toContainText('Target legality accepted');
  await expect(page.getByLabel('Final state')).toContainText('Raider is damaged and rattled');

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
