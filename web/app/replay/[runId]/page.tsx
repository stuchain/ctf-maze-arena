'use client';

import { useParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { MazeGrid } from '@/components/MazeGrid';
import { backendMazeToMazeData } from '@/lib/maze';
import { publicEnv } from '@/lib/env';

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
  const runId = String(params.runId);
  return <ReplayView key={runId} runId={runId} />;
}

function ReplayView({ runId }: { runId: string }) {
  const [replay, setReplay] = useState<ReplayPayload | null>(null);
  const [mazeJson, setMazeJson] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/replay/${encodeURIComponent(runId)}`)
      .then((response) => response.ok ? response.json() : Promise.reject(new Error('Replay not found')))
      .then((data: ReplayPayload) => {
        setReplay(data);
        return fetch(
          `${publicEnv.NEXT_PUBLIC_API_URL}/api/maze/${encodeURIComponent(data.mazeId)}`,
        ).then((response) => response.ok
          ? response.json()
          : Promise.reject(new Error('Maze not found')));
      })
      .then(setMazeJson)
      .catch((cause: Error) => setError(cause.message));
  }, [runId]);

  useEffect(() => {
    if (!playing || !replay?.frames.length || frameIndex >= replay.frames.length - 1) return;
    const timer = setTimeout(() => setFrameIndex((index) => index + 1), 100);
    return () => clearTimeout(timer);
  }, [playing, frameIndex, replay?.frames.length]);

  useEffect(() => {
    if (!copied) return;
    const timer = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timer);
  }, [copied]);

  const handleShare = async () => {
    await navigator.clipboard.writeText(`${window.location.origin}/replay/${runId}`);
    setCopied(true);
  };

  if (error) return <main id="main-content" className="p-4" role="alert">{error}</main>;
  if (!replay || !mazeJson) return <main id="main-content" className="p-4">Loading replay…</main>;

  const maze = backendMazeToMazeData(mazeJson);
  const frame = replay.frames[frameIndex];
  const lastFrame = !replay.frames.length || frameIndex >= replay.frames.length - 1;
  const isPlaying = playing && !lastFrame;

  return (
    <main id="main-content" className="p-4">
      <div className="mb-2 flex flex-wrap items-center gap-3">
        <h1 className="text-xl font-semibold">Solve replay</h1>
        <button type="button" className="rounded border border-zinc-400 px-3 py-1 text-sm" onClick={() => void handleShare()}>
          Share replay
        </button>
        {copied ? <span className="text-sm text-green-600" role="status">Link copied</span> : null}
      </div>
      <p>Solver: {replay.solver} | Visited: {replay.stats.visited} | Cost: {replay.stats.cost}</p>

      <div className="mb-4 flex gap-2">
        <button
          type="button"
          className="rounded bg-zinc-800 px-3 py-1 text-sm text-white disabled:opacity-50"
          onClick={() => setPlaying(true)}
          disabled={isPlaying || lastFrame}
        >
          Play
        </button>
        <button type="button" className="rounded bg-zinc-500 px-3 py-1 text-sm text-white" onClick={() => setPlaying(false)}>
          Pause
        </button>
        <button
          type="button"
          className="rounded border border-zinc-400 px-3 py-1 text-sm"
          onClick={() => { setFrameIndex(0); setPlaying(false); }}
        >
          Reset
        </button>
        <span className="self-center text-sm text-zinc-600">
          Frame {replay.frames.length ? frameIndex + 1 : 0} / {replay.frames.length}
        </span>
      </div>

      <MazeGrid
        maze={maze}
        frontier={frame?.frontier}
        visited={frame?.visited}
        current={frame?.current}
        path={lastFrame ? replay.path : undefined}
      />
    </main>
  );
}
