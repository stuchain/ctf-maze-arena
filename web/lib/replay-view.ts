import { z } from 'zod';
import { cellSchema } from '@/lib/maze-view';
import { PROTOCOL_VERSION } from '@/lib/realtime';

export const MAX_REPLAY_EVENTS = 2048;
const boundedInteger = z.number().int().nonnegative().safe();
const replayCells = z.array(cellSchema).max(10_000);
const visualStateSchema = z.object({
  step: boundedInteger,
  frontier: replayCells,
  visited: replayCells,
  current: cellSchema.nullish(),
});
const visualDeltaSchema = z.object({
  step: boundedInteger,
  frontierAdded: replayCells,
  frontierRemoved: replayCells,
  visitedAdded: replayCells,
  current: cellSchema.nullish(),
});
const replayEventSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('snapshot'), sequence: boundedInteger, state: visualStateSchema }),
  z.object({ type: z.literal('delta'), sequence: boundedInteger, delta: visualDeltaSchema }),
]);

const replaySchema = z.object({
  mazeId: z.string().min(1),
  protocolVersion: z.literal(PROTOCOL_VERSION),
  solver: z.string().min(1).max(32),
  seed: boundedInteger,
  events: z.array(replayEventSchema).max(MAX_REPLAY_EVENTS),
  path: z.array(cellSchema).max(10_000),
  stats: z.object({
    visited: boundedInteger,
    cost: boundedInteger,
    ms: boundedInteger,
  }),
});

export type ReplayViewModel = z.infer<typeof replaySchema>;

export function parseReplayView(value: unknown): ReplayViewModel {
  const parsed = replaySchema.safeParse(value);
  if (!parsed.success) throw new Error('The replay payload is invalid or uses an unsupported protocol.');
  return parsed.data;
}
