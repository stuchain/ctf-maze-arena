import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, requestJson } from '@/lib/api';
import { z } from 'zod';

describe('requestJson', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('returns validated response data', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ ok: true }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    )));

    await expect(requestJson('/test', z.object({ ok: z.literal(true) })))
      .resolves.toEqual({ ok: true });
  });

  it('normalizes API errors and preserves the request ID', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ error: { code: 'invalid_input', message: 'Invalid input' } }),
      { status: 400, headers: { 'x-request-id': 'request-123' } },
    )));

    const error = await requestJson('/test', z.object({ ok: z.boolean() }))
      .catch((cause: unknown) => cause);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({
      message: 'Invalid input',
      status: 400,
      code: 'invalid_input',
      requestId: 'request-123',
    });
  });

  it('rejects successful responses that violate the runtime schema', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ ok: 'yes' }),
      { status: 200 },
    )));

    await expect(requestJson('/test', z.object({ ok: z.boolean() })))
      .rejects.toMatchObject({ code: 'invalid_response', status: 502 });
  });
});
