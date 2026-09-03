import { z } from 'zod';

const errorPayloadSchema = z.object({
  error: z.union([
    z.string(),
    z.object({
      code: z.string().optional(),
      message: z.string().optional(),
    }),
  ]).optional(),
  message: z.string().optional(),
  code: z.string().optional(),
});

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly code: string,
    readonly requestId: string | null = null,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function readJson(response: Response): Promise<unknown> {
  try {
    return await response.json();
  } catch {
    return null;
  }
}

function readError(payload: unknown, fallback: string) {
  const parsed = errorPayloadSchema.safeParse(payload);
  if (!parsed.success) return { message: fallback, code: 'request_failed' };

  const error = parsed.data.error;
  if (typeof error === 'string') {
    return { message: error, code: parsed.data.code ?? 'request_failed' };
  }
  return {
    message: error?.message ?? parsed.data.message ?? fallback,
    code: error?.code ?? parsed.data.code ?? 'request_failed',
  };
}

export async function requestJson<TSchema extends z.ZodType>(
  url: string,
  schema: TSchema,
  init?: RequestInit,
): Promise<z.output<TSchema>> {
  const response = await fetch(url, init);
  const payload = await readJson(response);
  const requestId = response.headers.get('x-request-id');

  if (!response.ok) {
    const details = readError(payload, response.statusText || 'Request failed');
    throw new ApiError(details.message, response.status, details.code, requestId);
  }

  const parsed = schema.safeParse(payload);
  if (!parsed.success) {
    throw new ApiError('The server returned an invalid response.', 502, 'invalid_response', requestId);
  }
  return parsed.data;
}

export function toErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error && error.message ? error.message : fallback;
}

export const generateResponseSchema = z.object({
  mazeId: z.string().min(1),
  maze: z.unknown(),
});

export const dailyResponseSchema = z.object({
  seed: z.coerce.number().int().nonnegative(),
  date: z.string(),
  w: z.coerce.number().int().positive(),
  h: z.coerce.number().int().positive(),
});

export const solveResponseSchema = z.object({ runId: z.string().min(1) });

export const leaderboardSubmitResponseSchema = z.object({
  accepted: z.literal(true),
  duplicate: z.boolean(),
});

export const tokenResponseSchema = z.object({
  token: z.string().min(1),
  tokenType: z.literal('Bearer'),
  expiresAt: z.number().int().positive(),
});

export const leaderboardEntrySchema = z.object({
  runId: z.string(),
  solver: z.string(),
  cost: z.number().nonnegative(),
  ms: z.number().nonnegative(),
  visited: z.number().nonnegative(),
  displayName: z.string().nullable().optional(),
  avatarUrl: z.string().url().nullable().optional(),
});

export const leaderboardResponseSchema = z.array(leaderboardEntrySchema);
export type LeaderboardEntry = z.infer<typeof leaderboardEntrySchema>;
