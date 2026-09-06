import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  ApiError,
  dailyResponseSchema,
  leaderboardResponseSchema,
  profileSchema,
  requestJson,
} from '@/lib/api';
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

describe('community API contracts', () => {
  const entry = {
    rank: 1, tied: true, runId: 'de305d54-75b4-431b-adb2-eb6b9e546014', solver: 'ASTAR',
    cost: 12, ms: 4, visited: 30, displayName: 'Ada', avatarUrl: null,
    acceptedAt: '2026-09-05T12:00:00Z', isPersonal: true,
  };

  it('validates versioned personalized daily challenges', () => {
    expect(dailyResponseSchema.parse({
      challengeId: 'de305d54-75b4-431b-adb2-eb6b9e546014', seed: 42, date: '2026-09-05',
      version: 1, w: 15, h: 15, algo: 'KRUSKAL', featurePreset: 'classic',
      secondsUntilReset: 3600, personalBest: entry, streak: 2, completed: true,
    }).personalBest?.runId).toBe(entry.runId);
  });

  it('validates stable ranks, ties, and personal markers', () => {
    expect(leaderboardResponseSchema.parse([entry])[0]).toMatchObject({ rank: 1, tied: true, isPersonal: true });
  });

  it('validates versioned server awards and personal history', () => {
    const profile = profileSchema.parse({
      providerSubject: 'github:1', displayName: 'Ada', avatarUrl: null,
      totalSubmissions: 1, dailyStreak: 1,
      achievements: [{ key: 'first_finish', version: 1, name: 'First Finish', description: 'Finish.', awardedAt: '2026-09-05T12:00:00Z' }],
      history: [{ ...entry, challengeDate: '2026-09-05' }],
    });
    expect(profile.achievements[0].version).toBe(1);
  });
});
