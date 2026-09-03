import { describe, expect, it } from 'vitest';
import { MAX_REPLAY_EVENTS, parseReplayView } from '@/lib/replay-view';
import { appendBoundedFrame } from '@/hooks/useSolveStream';
import type { VisualState } from '@/lib/realtime';

const validReplay = {
  mazeId: 'maze-1', protocolVersion: 1, solver: 'BFS', seed: 42,
  events: [], path: [[0, 0]], stats: { visited: 1, cost: 0, ms: 1 },
};

describe('replay view model and bounded history', () => {
  it('accepts the supported replay protocol', () => {
    expect(parseReplayView(validReplay)).toMatchObject({ solver: 'BFS', path: [[0, 0]] });
  });

  it('rejects unsupported protocols and oversized histories', () => {
    expect(() => parseReplayView({ ...validReplay, protocolVersion: 2 })).toThrow(/unsupported protocol/);
    expect(() => parseReplayView({ ...validReplay, events: Array.from({ length: MAX_REPLAY_EVENTS + 1 }) })).toThrow(/invalid/);
    expect(() => parseReplayView({ ...validReplay, events: [{ type: 'snapshot', sequence: 1, state: { step: -1 } }] })).toThrow(/invalid/);
  });

  it('retains only the configured live frame window', () => {
    let frames: VisualState[] = [];
    for (let step = 1; step <= 6; step += 1) frames = appendBoundedFrame(frames, { step, frontier: [], visited: [] }, 3);
    expect(frames.map(({ step }) => step)).toEqual([4, 5, 6]);
    expect(appendBoundedFrame(frames, { step: 7, frontier: [], visited: [] }, 1).map(({ step }) => step)).toEqual([7]);
  });
});
