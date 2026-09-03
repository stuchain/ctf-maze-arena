import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

async function expectNoSeriousAccessibilityViolations(page: Page) {
  const results = await new AxeBuilder({ page }).analyze();
  const violations = results.violations.filter(({ impact }) => impact === 'critical' || impact === 'serious');
  expect(violations, violations.map(({ id, nodes }) => `${id}: ${nodes.length} node(s)`).join('\n')).toEqual([]);
}

test('desktop shell is ordered, named, and accessible in dark mode', async ({ page }, testInfo) => {
  await page.addInitScript(() => {
    if (!localStorage.getItem('ctf-maze-theme')) localStorage.setItem('ctf-maze-theme', 'dark');
  });
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Watch the Search Unfold' })).toBeVisible();

  const configuration = await page.locator('#configuration').boundingBox();
  const arena = await page.locator('#arena').boundingBox();
  const inspector = await page.locator('#inspector').boundingBox();
  expect(configuration && arena && inspector).toBeTruthy();
  expect(configuration!.x).toBeLessThan(arena!.x);
  expect(arena!.x).toBeLessThan(inspector!.x);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  await expectNoSeriousAccessibilityViolations(page);
  await testInfo.attach('desktop-dark.png', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
});

test('light theme persists and remains accessible', async ({ page }, testInfo) => {
  await page.addInitScript(() => {
    if (!localStorage.getItem('ctf-maze-theme')) localStorage.setItem('ctf-maze-theme', 'dark');
  });
  await page.goto('/');
  await page.getByTestId('theme-toggle').click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await expectNoSeriousAccessibilityViolations(page);
  await testInfo.attach('desktop-light.png', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
});

test('mobile shell is stage-first, keyboard reachable, and overflow-free', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 320, height: 720 });
  await page.emulateMedia({ reducedMotion: 'reduce', colorScheme: 'dark' });
  await page.goto('/');

  const arena = await page.locator('#arena').boundingBox();
  const configuration = await page.locator('#configuration').boundingBox();
  const inspector = await page.locator('#inspector').boundingBox();
  expect(arena && configuration && inspector).toBeTruthy();
  expect(arena!.y).toBeLessThan(configuration!.y);
  expect(configuration!.y).toBeLessThan(inspector!.y);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);

  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('#main-content')).toBeFocused();
  const duration = await page.getByTestId('theme-toggle').evaluate((element) => getComputedStyle(element).transitionDuration);
  expect(Number.parseFloat(duration)).toBeLessThanOrEqual(0.00001);
  await testInfo.attach('mobile-reduced-motion.png', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
});

test('core workflow remains available at a 200 percent zoom equivalent', async ({ page }) => {
  await page.setViewportSize({ width: 720, height: 500 });
  await page.goto('/');
  await expect(page.getByRole('navigation', { name: 'Workspace sections' })).toBeVisible();
  await expect(page.getByTestId('generate-button')).toBeVisible();
  await expect(page.getByTestId('solver-picker')).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
});

test('tablet shell promotes the inspector without horizontal overflow', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 900, height: 900 });
  await page.goto('/');
  const configuration = await page.locator('#configuration').boundingBox();
  const arena = await page.locator('#arena').boundingBox();
  const inspector = await page.locator('#inspector').boundingBox();
  expect(configuration && arena && inspector).toBeTruthy();
  expect(configuration!.x).toBeLessThan(arena!.x);
  expect(inspector!.y).toBeGreaterThan(arena!.y);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  await testInfo.attach('tablet.png', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
});

test('missing replay has a branded recovery state', async ({ page }) => {
  await page.goto('/replay/not-a-run');
  await expect(page.getByRole('heading', { name: 'That trail has gone cold.' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Return to the arena' })).toBeVisible();
  await expectNoSeriousAccessibilityViolations(page);
});

test('completed replay exposes accessible playback controls', async ({ page, request }) => {
  const api = process.env.NEXT_PUBLIC_API_URL ?? 'http://127.0.0.1:8080';
  const generated = await request.post(`${api}/api/maze/generate`, {
    data: { w: 12, h: 12, seed: 4404, algo: 'KRUSKAL' },
  });
  expect(generated.ok()).toBe(true);
  const { mazeId } = await generated.json() as { mazeId: string };
  const started = await request.post(`${api}/api/solve`, { data: { mazeId, solver: 'BFS' } });
  expect(started.ok()).toBe(true);
  const { runId } = await started.json() as { runId: string };

  await expect.poll(async () => (await request.get(`${api}/api/replay/${runId}`)).status(), { timeout: 20_000 }).toBe(200);
  await page.goto(`/replay/${runId}`);
  await expect(page.getByRole('heading', { name: 'Solve replay' })).toBeVisible();
  await expect(page.getByLabel('Replay frame')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Play', exact: true })).toBeVisible();
  await expect(page.getByTestId('maze-grid')).toBeVisible();
  await expectNoSeriousAccessibilityViolations(page);
});

test('destructive run cancellation uses a keyboard-operable confirmation', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('Width').fill('50');
  await page.getByLabel('Height').fill('50');
  await page.getByTestId('solver-picker').selectOption('DFS');
  await page.getByTestId('generate-button').click();
  await expect(page.getByTestId('maze-grid')).toBeVisible();
  await page.getByTestId('solve-button').click();
  const cancel = page.getByRole('button', { name: 'Cancel Run' });
  await expect(cancel).toBeVisible({ timeout: 20_000 });
  await cancel.click();
  await expect(page.getByRole('dialog', { name: 'Cancel This Solver Run?' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog', { name: 'Cancel This Solver Run?' })).toBeHidden();
});
