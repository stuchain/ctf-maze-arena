import { describe, expect, it } from 'vitest';
import { connectingState, reduceServerMessage, replayStates } from '@/lib/realtime';

const base = { protocolVersion: 1, runId: 'run-1' };

describe('realtime protocol reducer', () => {
  it('applies snapshots and deltas without duplicating resumed sequences', () => {
    let state = connectingState('run-1');
    state = reduceServerMessage(state, {
      ...base, type: 'snapshot', sequence: 1,
      state: { step: 1, frontier: [[1, 0]], visited: [[0, 0]], current: [0, 0] },
    });
    state = reduceServerMessage(state, {
      ...base, type: 'delta', sequence: 2,
      delta: { step: 2, frontierAdded: [[2, 0]], frontierRemoved: [[1, 0]], visitedAdded: [[1, 0]], current: [1, 0] },
    });
    const resumedDuplicate = reduceServerMessage(state, {
      ...base, type: 'delta', sequence: 2,
      delta: { step: 2, frontierAdded: [[9, 9]], frontierRemoved: [], visitedAdded: [[9, 9]] },
    });
    expect(resumedDuplicate.sequence).toBe(2);
    expect(resumedDuplicate.visual).toEqual({ step: 2, frontier: [[2, 0]], visited: [[0, 0], [1, 0]], current: [1, 0] });
  });

  it('surfaces distinct terminal states', () => {
    const running = { ...connectingState('run-1'), status: 'live' as const };
    expect(reduceServerMessage(running, { ...base, type: 'cancelled', sequence: 1 }).status).toBe('cancelled');
    expect(reduceServerMessage(running, { ...base, type: 'failed', sequence: 1, message: 'Safe failure' }).error).toBe('Safe failure');
    expect(reduceServerMessage(running, { ...base, type: 'completed', sequence: 1, path: [[0, 0]], stats: { visited: 1, cost: 0, ms: 1 } }).status).toBe('completed');
  });

  it('reconstructs deterministic replay states', () => {
    const states = replayStates([
      { type: 'snapshot', sequence: 1, state: { step: 1, frontier: [[1, 0]], visited: [[0, 0]] } },
      { type: 'delta', sequence: 2, delta: { step: 2, frontierAdded: [], frontierRemoved: [[1, 0]], visitedAdded: [[1, 0]], current: [1, 0] } },
    ]);
    expect(states).toHaveLength(2);
    expect(states[1]).toEqual({ step: 2, frontier: [], visited: [[0, 0], [1, 0]], current: [1, 0] });
  });
});
