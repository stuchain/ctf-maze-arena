import { expect, test } from '@playwright/test';

test('solve flow applies live progress before completing', async ({ page }) => {
  test.slow();
  test.setTimeout(90_000);

  await page.goto('/');

  await page.getByTestId('solver-picker').selectOption('DFS');
  await page.getByLabel('Width').fill('35');
  await page.getByLabel('Height').fill('35');
  await page.getByLabel('Seed').fill('4242');
  await page.getByLabel('Generator').selectOption('KRUSKAL');

  await page.getByTestId('generate-button').click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();

  await page.getByTestId('solve-button').click();
  await expect(page.getByTestId('stream-status')).toContainText('completed', {
    timeout: 60_000,
  });
  await expect(page.getByTestId('stream-status')).toContainText(/sequence [1-9][0-9]*/);
});
