import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('anonymous daily play and community conversion remain accessible', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Your Arena Profile' })).toBeVisible();
  await expect(page.getByText(/Anonymous play stays fully available/)).toBeVisible();
  await expect(page.getByText('Persistent identity is disabled in this environment.')).toBeVisible();
  await expect(page.getByLabel('View')).toBeDisabled();
  await expect(page.getByText(/Browser-only progress is marked legacy/)).toBeVisible();

  await page.getByRole('button', { name: 'Load Daily' }).click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();
  await expect(page.getByText(/left$/)).toBeVisible();
  await expect(page.getByText('Not completed · 0 days streak')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Daily', exact: true })).toBeEnabled();

  await page.getByTestId('solve-button').click();
  await expect(page.getByRole('button', { name: 'Share Result Replay' })).toBeVisible({ timeout: 30_000 });
  await expect(page.getByRole('button', { name: 'Sign In for Future Ranked Runs' })).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(({ impact }) => impact === 'critical' || impact === 'serious')).toEqual([]);
});
