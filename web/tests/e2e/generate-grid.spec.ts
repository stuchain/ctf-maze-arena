import { expect, test } from '@playwright/test';

test('generate maze renders grid', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByTestId('maze-grid-empty')).toBeVisible();
  await page.getByTestId('generate-button').click();

  await expect(page.getByTestId('maze-grid')).toBeVisible();
  await expect(page.getByTestId('maze-grid-svg').locator('rect').first()).toBeVisible();
});
