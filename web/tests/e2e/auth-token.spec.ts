import { test, expect } from '@playwright/test';

test('token route returns 401 when signed out', async ({ request }) => {
  const response = await request.get('/api/token');
  expect(response.status()).toBe(401);
  const body = await response.json();
  if (typeof body.error === 'string') {
    expect(body.error.toLowerCase()).toContain('unauthorized');
  } else {
    expect(body.error?.code).toBe('UNAUTHORIZED');
  }
});
