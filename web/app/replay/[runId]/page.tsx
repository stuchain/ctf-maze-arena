'use client';

import { useParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { MazeGrid } from '@/components/MazeGrid';
import { backendMazeToMazeData } from '@/lib/maze';

const API = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

interface ReplayPayload {
  mazeId: string;
  solver: string;
  seed: number;
  frames: Array<{
    t: number;
    frontier: [number, number][];
    visited: [number, number][];
    current?: [number, number];
  }>;
  path: [number, number][];
  stats: { visited: number; cost: number; ms: number };
}

export default function ReplayPage() {
  const params = useParams();
  const runId = params.runId as string;
  const [replay, setReplay] = useState<ReplayPayload | null>(null);
  const [mazeJson, setMazeJson] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    setReplay(null);
    setMazeJson(null);
    setError(null);
    setFrameIndex(0);
    setPlaying(false);

    fetch(`${API}/api/replay/${encodeURIComponent(runId)}`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error('Not found'))))
      .then((data: ReplayPayload) => {
        setReplay(data);
        return fetch(
          `${API}/api/maze/${encodeURIComponent(data.mazeId)}`,
        ).then((mr) =>
          mr.ok ? mr.json() : Promise.reject(new Error('Maze not found')),
        );
      })
      .then(setMazeJson)
      .catch((e: Error) => setError(e.message));
  }, [runId]);

  useEffect(() => {
    if (!playing || !replay?.frames?.length) return;
    const idx = frameIndex;
    if (idx >= replay.frames.length - 1) {
      setPlaying(false);
      return;
    }
    const t = setTimeout(() => setFrameIndex(idx + 1), 100);
    return () => clearTimeout(t);
  }, [playing, frameIndex, replay?.frames]);

  if (error) return <div>Error: {error}</div>;
  if (!replay || !mazeJson) return <div>Loading...</div>;

  const maze = backendMazeToMazeData(mazeJson);
  const frame = replay.frames[frameIndex];
  const lastFrame =
    !replay.frames.length ||
    frameIndex >= replay.frames.length - 1;
  const showPath = lastFrame ? replay.path : undefined;

  return (
    <div className="p-4">
      <h1>Replay: {runId}</h1>
      <p>
        Solver: {replay.solver} | Visited: {replay.stats?.visited} | Cost:{' '}
        {replay.stats?.cost}
      </p>

      <div className="flex gap-2 mb-4">
        <button
          type="button"
          className="rounded bg-zinc-800 px-3 py-1 text-white text-sm disabled:opacity-50"
          onClick={() => setPlaying(true)}
          disabled={
            playing || !replay.frames.length || frameIndex >= replay.frames.length - 1
          }
        >
          Play
        </button>
        <button
          type="button"
          className="rounded bg-zinc-500 px-3 py-1 text-white text-sm"
          onClick={() => setPlaying(false)}
        >
          Pause
        </button>
        <button
          type="button"
          className="rounded border border-zinc-400 px-3 py-1 text-sm"
          onClick={() => {
            setFrameIndex(0);
            setPlaying(false);
          }}
        >
          Reset
        </button>
        <span className="text-sm text-zinc-600 self-center">
          Frame {replay.frames.length ? frameIndex + 1 : 0} /{' '}
          {replay.frames.length}
        </span>
      </div>

      <MazeGrid
        maze={maze}
        frontier={frame?.frontier}
        visited={frame?.visited}
        current={frame?.current}
        path={showPath}
      />
    </div>
  );
}
