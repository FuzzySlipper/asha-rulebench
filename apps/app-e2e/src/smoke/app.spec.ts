import { expect, test } from '@playwright/test';

test('boots the rulebench shell', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { name: 'ASHA Rulebench', exact: true })).toBeVisible();
  await expect(page.getByText('Hexing Bolt Opening')).toBeVisible();
  await expect(page.getByLabel('Scenario board')).toBeVisible();
  await expect(page.getByLabel('Selected action')).toContainText('Hexing Bolt');
  await expect(page.getByLabel('Combatants')).toContainText('Adept');
  await expect(page.getByLabel('DomainEvents timeline')).toContainText('DamageApplied');
  await expect(page.getByLabel('Rule trace')).toContainText('Target legality accepted');
  await expect(page.getByLabel('Final state')).toContainText('Raider is damaged and rattled');
});
