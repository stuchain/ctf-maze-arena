'use client';

import { useEffect, useReducer } from 'react';
import { checkAndAward } from '@/lib/achievements';
import { createSolveStreamUrl } from '@/lib/ws';
import { publicEnv } from '@/lib/env';

export type StreamStatus = 'idle' | 'connecting' | 'active' | 'finished' | 'error';

export interface SolveFrame {
  t: number;
  frontier: [number, number][];
  visited: [number, number][];
  current?: [number, number];
}

export interface SolveStats {
  visited: number;
  cost: number;
  ms: number;
}

export interface UseSolveStreamResult {
  status: StreamStatus;
  frames: SolveFrame[];
  path: [number, number][];
  stats: SolveStats | null;
  error: string | null;
}

interface StreamState extends UseSolveStreamResult {
  runId: string | null;
}

type StreamAction =
  | { type: 'connected'; runId: string }
  | { type: 'frame'; runId: string; frame: SolveFrame }
  | { type: 'finished'; runId: string; path: [number, number][]; stats: SolveStats | null }
  | { type: 'error'; runId: string; message: string };

const IDLE_STATE: StreamState = {
  runId: null,
  status: 'idle',
  frames: [],
  path: [],
  stats: null,
  error: null,
};

function reducer(state: StreamState, action: StreamAction): StreamState {
  if (state.runId !== action.runId && action.type !== 'connected') {
    return state;
  }
  switch (action.type) {
    case 'connected':
      return { ...IDLE_STATE, runId: action.runId, status: 'active' };
    case 'frame':
      return { ...state, frames: [...state.frames, action.frame] };
    case 'finished':
      return { ...state, status: 'finished', path: action.path, stats: action.stats, error: null };
    case 'error':
      return { ...state, status: 'error', error: action.message };
  }
}

function normalizeCell(cell: unknown): [number, number] {
  if (Array.isArray(cell) && cell.length >= 2) {
    return [Number(cell[0]), Number(cell[1])];
  }
  return [0, 0];
}

function normalizeFrame(data: unknown): SolveFrame {
  const value = data as Record<string, unknown>;
  const frontier = Array.isArray(value.frontier) ? value.frontier.map(normalizeCell) : [];
  const visited = Array.isArray(value.visited) ? value.visited.map(normalizeCell) : [];
  const current = value.current == null ? undefined : normalizeCell(value.current);
  return { t: Number(value.t ?? 0), frontier, visited, current };
}

function normalizeStats(data: unknown): SolveStats | null {
  if (!data || typeof data !== 'object') return null;
  const value = data as Record<string, unknown>;
  return {
    visited: Number(value.visited ?? 0),
    cost: Number(value.cost ?? 0),
    ms: Number(value.ms ?? 0),
  };
}

export function useSolveStream(
  runId: string | null,
  solver: string | null,
): UseSolveStreamResult {
  const [state, dispatch] = useReducer(reducer, IDLE_STATE);

  useEffect(() => {
    if (!runId) return;
    const ws = new WebSocket(createSolveStreamUrl(runId));

    const finishFromReplay = async () => {
      for (let attempt = 0; attempt < 20; attempt += 1) {
        try {
          const response = await fetch(
            `${publicEnv.NEXT_PUBLIC_API_URL}/api/replay/${encodeURIComponent(runId)}`,
          );
          if (!response.ok) {
            await new Promise((resolve) => setTimeout(resolve, 250));
            continue;
          }
          const replay = (await response.json()) as Record<string, unknown>;
          const path = Array.isArray(replay.path) ? replay.path.map(normalizeCell) : [];
          const stats = normalizeStats(replay.stats);
          if (stats) checkAndAward({ ...stats, solver: solver ?? '' });
          dispatch({ type: 'finished', runId, path, stats });
          return true;
        } catch {
          await new Promise((resolve) => setTimeout(resolve, 250));
        }
      }
      return false;
    };

    ws.onopen = () => dispatch({ type: 'connected', runId });
    ws.onerror = () => {
      dispatch({ type: 'error', runId, message: 'WebSocket error' });
    };
    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as Record<string, unknown>;
        if (message.type === 'frame') {
          dispatch({ type: 'frame', runId, frame: normalizeFrame(message.data) });
          return;
        }
        if (message.type === 'finished') {
          const path = Array.isArray(message.path) ? message.path.map(normalizeCell) : [];
          const stats = normalizeStats(message.stats);
          if (stats) checkAndAward({ ...stats, solver: solver ?? '' });
          dispatch({ type: 'finished', runId, path, stats });
          return;
        }
        if (message.type === 'error') {
          const text = typeof message.message === 'string'
            ? message.message
            : typeof message.error === 'string'
              ? message.error
              : 'Unknown error';
          if (text.includes('unknown or completed runId')) {
            void finishFromReplay().then((finished) => {
              if (!finished) {
                dispatch({ type: 'error', runId, message: text });
              }
            });
          } else {
            dispatch({ type: 'error', runId, message: text });
          }
        }
      } catch {
        dispatch({ type: 'error', runId, message: 'Invalid message' });
      }
    };

    return () => ws.close();
  }, [runId, solver]);

  if (!runId) return IDLE_STATE;
  if (state.runId !== runId) {
    return { status: 'connecting', frames: [], path: [], stats: null, error: null };
  }
  return state;
}
