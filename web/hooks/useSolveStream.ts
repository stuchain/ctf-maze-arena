'use client';

import { useEffect, useReducer, useRef } from 'react';
import { checkAndAward } from '@/lib/achievements';
import { createSolveStreamUrl } from '@/lib/ws';
import { publicEnv } from '@/lib/env';
import {
  connectingState,
  idleStreamState,
  PROTOCOL_VERSION,
  reduceServerMessage,
  type SolveStats,
  type StreamState,
  type StreamStatus,
  type VisualState,
} from '@/lib/realtime';

export type { SolveStats, StreamStatus } from '@/lib/realtime';
export type SolveFrame = VisualState;

export interface UseSolveStreamResult {
  status: StreamStatus;
  frames: SolveFrame[];
  path: [number, number][];
  stats: SolveStats | null;
  error: string | null;
  sequence: number;
}

export const LIVE_FRAME_CAPACITY = 512;

interface ClientStreamState extends StreamState {
  frames: VisualState[];
}

const idleClientState: ClientStreamState = { ...idleStreamState, frames: [] };

export function appendBoundedFrame(frames: VisualState[], frame: VisualState, capacity = LIVE_FRAME_CAPACITY) {
  if (frames.at(-1)?.step === frame.step) return frames;
  const boundedCapacity = Math.max(1, capacity);
  return frames.length >= boundedCapacity
    ? [...frames.slice(frames.length - boundedCapacity + 1), frame]
    : [...frames, frame];
}

type Action =
  | { type: 'reset'; runId: string }
  | { type: 'status'; status: StreamStatus }
  | { type: 'message'; message: unknown }
  | { type: 'replay'; path: [number, number][]; stats: SolveStats | null }
  | { type: 'cancelled' }
  | { type: 'failure'; message: string };

function reducer(state: ClientStreamState, action: Action): ClientStreamState {
  switch (action.type) {
    case 'reset': return { ...connectingState(action.runId), frames: [] };
    case 'status': return { ...state, status: action.status };
    case 'message': {
      const next = reduceServerMessage(state, action.message);
      if (!next.visual || next.visual.step === state.visual?.step) return { ...next, frames: state.frames };
      const frames = appendBoundedFrame(state.frames, next.visual);
      return { ...next, frames };
    }
    case 'replay': return { ...state, status: 'completed', path: action.path, stats: action.stats, error: null };
    case 'cancelled': return { ...state, status: 'cancelled', error: null };
    case 'failure': return { ...state, status: 'failed', error: action.message };
  }
}

function normalizeCell(value: unknown): [number, number] {
  return Array.isArray(value) && value.length >= 2 ? [Number(value[0]), Number(value[1])] : [0, 0];
}
function normalizeStats(value: unknown): SolveStats | null {
  if (!value || typeof value !== 'object') return null;
  const item = value as Record<string, unknown>;
  return { visited: Number(item.visited ?? 0), cost: Number(item.cost ?? 0), ms: Number(item.ms ?? 0) };
}

export function useSolveStream(runId: string | null, solver: string | null): UseSolveStreamResult {
  const [state, dispatch] = useReducer(reducer, idleClientState);
  const sequence = useRef(0);

  useEffect(() => {
    if (!runId) return;
    let stopped = false;
    let terminal = false;
    let socket: WebSocket | null = null;
    let retryTimer: ReturnType<typeof setTimeout> | null = null;
    let attempt = 0;
    sequence.current = 0;
    dispatch({ type: 'reset', runId });

    const finishFromReplay = async () => {
      for (let index = 0; index < 20 && !stopped; index += 1) {
        try {
          const runResponse = await fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/run/${encodeURIComponent(runId)}`);
          if (runResponse.ok) {
            const run = (await runResponse.json()) as Record<string, unknown>;
            if (run.status === 'cancelled') {
              terminal = true;
              dispatch({ type: 'cancelled' });
              return true;
            }
            if (run.status === 'failed') {
              terminal = true;
              dispatch({ type: 'failure', message: `The solve failed (${String(run.errorCode ?? 'unknown')}).` });
              return true;
            }
            if (run.status === 'completed') {
              const response = await fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/replay/${encodeURIComponent(runId)}`);
              if (response.ok) {
                const replay = (await response.json()) as Record<string, unknown>;
                const path = Array.isArray(replay.path) ? replay.path.map(normalizeCell) : [];
                const resultStats = normalizeStats(replay.stats);
                if (resultStats) checkAndAward({ ...resultStats, solver: solver ?? '' });
                terminal = true;
                dispatch({ type: 'replay', path, stats: resultStats });
                return true;
              }
            }
          }
        } catch { /* bounded retry handles sleeping or reconnecting services */ }
        await new Promise((resolve) => setTimeout(resolve, 250));
      }
      return false;
    };

    const connect = () => {
      if (stopped || terminal) return;
      dispatch({ type: 'status', status: attempt === 0 ? 'connecting' : 'reconnecting' });
      socket = new WebSocket(createSolveStreamUrl(runId, sequence.current));
      socket.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as Record<string, unknown>;
          if (message.protocolVersion !== PROTOCOL_VERSION || message.runId !== runId) return;
          const nextSequence = Number(message.sequence ?? sequence.current);
          if (message.type === 'connected') {
            sequence.current = nextSequence;
            attempt = 0;
          } else if (nextSequence > sequence.current) {
            sequence.current = nextSequence;
          }
          if (message.type === 'completed') {
            const resultStats = normalizeStats(message.stats);
            if (resultStats) checkAndAward({ ...resultStats, solver: solver ?? '' });
            terminal = true;
          } else if (message.type === 'failed') {
            if (message.code === 'stream_expired') {
              void finishFromReplay().then((finished) => {
                if (!finished && !stopped) dispatch({ type: 'failure', message: 'Live history and replay are unavailable.' });
              });
              return;
            }
            terminal = true;
          } else if (message.type === 'cancelled') {
            terminal = true;
          }
          dispatch({ type: 'message', message });
        } catch {
          terminal = true;
          dispatch({ type: 'failure', message: 'Invalid realtime message.' });
        }
      };
      socket.onclose = () => {
        if (stopped || terminal) return;
        if (attempt >= 5) {
          void finishFromReplay().then((finished) => {
            if (!finished && !stopped) dispatch({ type: 'failure', message: 'Realtime connection could not be restored.' });
          });
          return;
        }
        dispatch({ type: 'status', status: attempt === 0 ? 'waking' : 'reconnecting' });
        attempt += 1;
        const base = Math.min(250 * 2 ** (attempt - 1), 4000);
        const jitter = Math.floor(Math.random() * Math.max(1, base / 4));
        retryTimer = setTimeout(connect, base + jitter);
      };
      socket.onerror = () => socket?.close();
    };

    connect();
    return () => {
      stopped = true;
      if (retryTimer) clearTimeout(retryTimer);
      socket?.close(1000, 'component cleanup');
    };
  }, [runId, solver]);

  if (!runId) return { status: 'idle', frames: [], path: [], stats: null, error: null, sequence: 0 };
  if (state.runId !== runId) return { status: 'connecting', frames: [], path: [], stats: null, error: null, sequence: 0 };
  return { status: state.status, frames: state.frames, path: state.path, stats: state.stats, error: state.error, sequence: state.sequence };
}
