'use client';

import { useEffect, useRef, useState } from 'react';
import { checkAndAward } from '@/lib/achievements';
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

const API = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

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

export function useSolveStream(
  runId: string | null,
  solver: string | null,
): UseSolveStreamResult {
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
    const finalizeFromReplay = async () => {
      for (let attempt = 0; attempt < 20; attempt += 1) {
        try {
          const replayRes = await fetch(`${API}/api/replay/${encodeURIComponent(runId)}`);
          if (!replayRes.ok) {
            await new Promise((resolve) => setTimeout(resolve, 250));
            continue;
          }
          const replay = (await replayRes.json()) as {
            path?: unknown;
            stats?: Record<string, unknown>;
          };
          const replayPathRaw = Array.isArray(replay.path) ? replay.path : [];
          setPath(replayPathRaw.map((c) => normalizeCell(c)));
          const replayStats = replay.stats;
          if (replayStats) {
            const visited = Number(replayStats.visited ?? 0);
            const cost = Number(replayStats.cost ?? 0);
            const ms = Number(replayStats.ms ?? 0);
            setStats({ visited, cost, ms });
            checkAndAward({
              visited,
              cost,
              solver: solver ?? '',
            });
          } else {
            setStats(null);
          }
          setStatus('finished');
          terminalRef.current = true;
          return true;
        } catch {
          await new Promise((resolve) => setTimeout(resolve, 250));
        }
      }
      return false;
    };

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
            const visited = Number(s.visited ?? 0);
            const cost = Number(s.cost ?? 0);
            const ms = Number(s.ms ?? 0);
            setStats({ visited, cost, ms });
            checkAndAward({
              visited,
              cost,
              solver: solver ?? '',
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
          if (errText.includes('unknown or completed runId')) {
            void finalizeFromReplay().then((ok) => {
              if (!ok) {
                setError(errText);
                setStatus('error');
                terminalRef.current = true;
              }
            });
          } else {
            setError(errText);
            setStatus('error');
            terminalRef.current = true;
          }
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
  }, [runId, solver]);

  return { status, frames, path, stats, error };
}
