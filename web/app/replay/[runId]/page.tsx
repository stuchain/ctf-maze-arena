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

  useEffect(() => {
    setReplay(null);
    setMazeJson(null);
    setError(null);

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

  if (error) return <div>Error: {error}</div>;
  if (!replay || !mazeJson) return <div>Loading...</div>;

  const maze = backendMazeToMazeData(mazeJson);

  return (
    <div className="p-4">
      <h1>Replay: {runId}</h1>
      <p>
        Solver: {replay.solver} | Visited: {replay.stats?.visited} | Cost:{' '}
        {replay.stats?.cost}
      </p>
      <MazeGrid maze={maze} path={replay.path} />
    </div>
  );
}
