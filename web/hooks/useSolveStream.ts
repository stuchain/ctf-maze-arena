'use client';

import { useEffect, useRef, useState } from 'react';
import { createSolveStreamUrl } from '@/lib/ws';

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

function normalizeCell(c: unknown): [number, number] {
  if (Array.isArray(c) && c.length >= 2) {
    return [Number(c[0]), Number(c[1])];
  }
  return [0, 0];
}

function normalizeFrame(data: unknown): SolveFrame {
  const d = data as Record<string, unknown>;
  const frontierRaw = Array.isArray(d.frontier) ? d.frontier : [];
  const visitedRaw = Array.isArray(d.visited) ? d.visited : [];
  const frontier = frontierRaw.map((c) => normalizeCell(c));
  const visited = visitedRaw.map((c) => normalizeCell(c));
  const currentRaw = d.current;
  const current =
    currentRaw !== undefined && currentRaw !== null
      ? normalizeCell(currentRaw)
      : undefined;
  return {
    t: Number(d.t ?? 0),
    frontier,
    visited,
    current,
  };
}

export function useSolveStream(runId: string | null): UseSolveStreamResult {
  const [status, setStatus] = useState<StreamStatus>('idle');
  const [frames, setFrames] = useState<SolveFrame[]>([]);
  const [path, setPath] = useState<[number, number][]>([]);
  const [stats, setStats] = useState<SolveStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const terminalRef = useRef(false);

  useEffect(() => {
    terminalRef.current = false;
    if (!runId) {
      setStatus('idle');
      setFrames([]);
      setPath([]);
      setStats(null);
      setError(null);
      return;
    }

    const wsUrl = createSolveStreamUrl(runId);
    setStatus('connecting');
    setFrames([]);
    setPath([]);
    setStats(null);
    setError(null);

    const ws = new WebSocket(wsUrl);

    ws.onopen = () => setStatus('active');
    ws.onerror = () => {
      setStatus('error');
      setError('WebSocket error');
      terminalRef.current = true;
    };
    ws.onclose = () => {
      if (!terminalRef.current) {
        setStatus('idle');
      }
    };
    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as {
          type?: string;
          data?: unknown;
          path?: unknown;
          stats?: unknown;
          message?: string;
          error?: string;
        };
        if (msg.type === 'connected') {
          return;
        }
        if (msg.type === 'frame' && msg.data !== undefined) {
          setFrames((prev) => [...prev, normalizeFrame(msg.data)]);
        } else if (msg.type === 'finished') {
          const pathRaw = Array.isArray(msg.path) ? msg.path : [];
          setPath(pathRaw.map((c) => normalizeCell(c)));
          const s = msg.stats as Record<string, unknown> | undefined;
          if (s) {
            setStats({
              visited: Number(s.visited ?? 0),
              cost: Number(s.cost ?? 0),
              ms: Number(s.ms ?? 0),
            });
          } else {
            setStats(null);
          }
          setStatus('finished');
          terminalRef.current = true;
        } else if (msg.type === 'error') {
          const errText =
            typeof msg.message === 'string'
              ? msg.message
              : typeof msg.error === 'string'
                ? msg.error
                : 'Unknown error';
          setError(errText);
          setStatus('error');
          terminalRef.current = true;
        }
      } catch {
        setError('Invalid message');
        setStatus('error');
        terminalRef.current = true;
      }
    };

    return () => {
      ws.close();
    };
  }, [runId]);

  return { status, frames, path, stats, error };
}
