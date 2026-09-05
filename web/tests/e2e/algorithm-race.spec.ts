import { expect, test } from '@playwright/test';

test('creates, completes, shares, and reloads a synchronized algorithm race', async ({ page }) => {
  test.setTimeout(90_000);
  await page.goto('/');
  await page.getByLabel('Width').fill('12');
  await page.getByLabel('Height').fill('12');
  await page.getByLabel('Seed').fill('606');
  await page.getByLabel('Generator').selectOption('PRIM');
  await page.getByTestId('generate-button').click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();

  await page.getByRole('button', { name: 'Algorithm Race' }).click();
  await page.getByRole('button', { name: 'Side by Side' }).click();
  await page.getByTestId('race-button').click();
  const race = page.getByTestId('race-experience');
  await expect(race).toBeVisible();
  await expect(race.getByText('Analysis Ready')).toBeVisible({ timeout: 60_000 });
  await expect(race.getByRole('button', { name: 'Zoom in' })).toHaveCount(3);
  await expect(race.getByRole('columnheader', { name: 'Peak frontier' })).toBeVisible();
  await expect(race.getByRole('link', { name: 'Open' })).toHaveCount(3);
  await expect(race.getByText(/BFS and A\* both found the optimal/)).toBeVisible();

  await page.getByRole('button', { name: 'Share Configuration' }).click();
  await expect(page.getByRole('status').filter({ hasText: /Race (link copied|URL is ready)/ })).toBeVisible();
  await expect.poll(() => new URL(page.url()).searchParams.get('seed')).toBe('606');
  await page.reload();
  await page.getByRole('button', { name: 'Algorithm Race' }).click();
  await page.getByRole('button', { name: 'Load Race from URL' }).click();
  await expect(page.getByLabel('Generator')).toHaveValue('PRIM');
  await expect(page.getByTestId('maze-grid')).toBeVisible();
});

test('keys preset enables the key-aware competitor and remains usable on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await page.getByLabel('Maze Features').selectOption('keys');
  await expect(page.getByLabel('Maze Features')).toHaveValue('keys');
  await page.getByTestId('generate-button').click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();
  await page.getByRole('button', { name: 'Algorithm Race' }).click();
  const dp = page.getByLabel(/DP KEYS/);
  await expect(dp).toBeEnabled();
  await dp.check();
  await page.getByTestId('race-button').click();
  const race = page.getByTestId('race-experience');
  await expect(race).toBeVisible();
  await expect(race.getByText('Analysis Ready')).toBeVisible({ timeout: 60_000 });
  await expect(race.locator('tbody tr')).toHaveCount(4);
});
