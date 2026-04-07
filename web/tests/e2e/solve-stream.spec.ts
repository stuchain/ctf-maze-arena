import { expect, test } from '@playwright/test';

test('solve flow reaches finished stream state', async ({ page }) => {
  test.slow();
  test.setTimeout(90_000);

  await page.goto('/');

  await page.getByTestId('solver-picker').selectOption('DFS');
  await page.getByLabel('Width:').fill('35');
  await page.getByLabel('Height:').fill('35');
  await page.getByLabel('Seed:').fill('4242');
  await page.getByLabel('Algorithm:').selectOption('KRUSKAL');

  await page.getByTestId('generate-button').click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();

  await page.getByTestId('solve-button').click();
  await expect(page.getByTestId('stream-status')).toContainText('stream: finished', {
    timeout: 60_000,
  });
});
