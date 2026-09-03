import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const knownMaze = {
  grid: { width: 3, height: 2 },
  walls: { inner: [
    [[0, 0], [1, 0]],
    [[1, 0], [1, 1]],
  ] },
  start: [0, 0],
  goal: [2, 1],
  keys: [[[1, 0], 0]],
  doors: [[[[1, 0], [1, 1]], 0]],
};

test('known maze renders correct boundaries, markers, and keyboard inspection', async ({ page }) => {
  await page.route('**/api/maze/generate', (route) => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ mazeId: 'known-maze', maze: knownMaze }),
  }));
  await page.goto('/');
  await page.getByTestId('generate-button').click();

  const innerWalls = page.getByTestId('maze-inner-walls');
  await expect(innerWalls).toHaveAttribute('d', 'M20 0L20 20M20 20L40 20');
  await expect(page.getByTestId('maze-outer-wall')).toHaveAttribute('width', '60');
  await expect(page.getByTestId('maze-outer-wall')).toHaveAttribute('height', '40');
  await expect(page.getByTestId('maze-key')).toHaveCount(1);
  await expect(page.getByTestId('maze-door')).toHaveCount(1);
  await expect(page.locator('.maze-marker--start')).toHaveCount(1);
  await expect(page.locator('.maze-marker--goal')).toHaveCount(1);

  const viewport = page.getByTestId('maze-viewport');
  await viewport.focus();
  await viewport.press('ArrowRight');
  await expect(page.getByTestId('cell-inspection')).toContainText('Column 2, Row 1');
  await expect(page.getByTestId('cell-inspection')).toContainText('unvisited');
  await viewport.press('+');
  await expect(page.locator('.zoom-readout')).toHaveText('125%');
  await page.getByRole('button', { name: 'Pan maze right' }).click();
  await expect(page.locator('.maze-transform')).toHaveAttribute('transform', 'translate(-40 0) scale(1.25)');
  await page.getByRole('button', { name: 'Fit maze to stage' }).click();
  await expect(page.locator('.maze-transform')).toHaveAttribute('transform', 'translate(0 0) scale(1)');

  const violations = (await new AxeBuilder({ page }).analyze()).violations.filter(({ impact }) => impact === 'critical' || impact === 'serious');
  expect(violations).toEqual([]);
});

test('50 by 50 SVG stays compact and meets the stage render budget', async ({ page }, testInfo) => {
  await page.goto('/');
  await page.getByLabel('Width').fill('50');
  await page.getByLabel('Height').fill('50');
  const startedAt = await page.evaluate(() => performance.now());
  await page.getByTestId('generate-button').click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();
  const renderMs = await page.evaluate((start) => performance.now() - start, startedAt);
  const svgNodes = await page.getByTestId('maze-grid-svg').locator('*').count();
  const wallPathBytes = (await page.getByTestId('maze-inner-walls').getAttribute('d'))?.length ?? 0;
  const heapBytes = await page.evaluate(() => {
    const memory = performance as Performance & { memory?: { usedJSHeapSize: number } };
    return memory.memory?.usedJSHeapSize ?? null;
  });

  expect(renderMs).toBeLessThan(5_000);
  expect(svgNodes).toBeLessThan(30);
  expect(wallPathBytes).toBeGreaterThan(1_000);
  await testInfo.attach('maze-50x50-performance.json', {
    body: Buffer.from(JSON.stringify({ renderMs, svgNodes, wallPathBytes, heapBytes }, null, 2)),
    contentType: 'application/json',
  });
  const screenshotPath = testInfo.outputPath('maze-50x50.png');
  await page.screenshot({ path: screenshotPath, fullPage: true });
  await testInfo.attach('maze-50x50.png', { path: screenshotPath, contentType: 'image/png' });
});

test('live playback can pause, step, change speed, and return to live', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('Width').fill('35');
  await page.getByLabel('Height').fill('35');
  await page.getByTestId('generate-button').click();
  await page.getByTestId('solve-button').click();
  await expect(page.getByTestId('stream-status')).toContainText('completed', { timeout: 30_000 });

  const timeline = page.getByLabel('Live run frame');
  await expect(timeline).toBeVisible();
  await timeline.fill('0');
  await expect(page.getByRole('button', { name: 'Return to Live' })).toBeVisible();
  await page.getByRole('button', { name: 'Next', exact: true }).click();
  await page.getByLabel('Playback speed').selectOption('2');
  await expect(page.getByLabel('Playback speed')).toHaveValue('2');
  await page.getByRole('button', { name: 'Return to Live' }).click();
  await expect(page.getByRole('button', { name: 'Return to Live' })).toBeHidden();
});
