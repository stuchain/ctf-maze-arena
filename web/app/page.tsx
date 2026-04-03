'use client';

import { useState } from 'react';
import { MazeGrid, type MazeData } from '../components/MazeGrid';
import { GenerateForm, type GenerateFormParams } from '../components/GenerateForm';
import { SolverPicker } from '../components/SolverPicker';
import { useSolveStream } from '../hooks/useSolveStream';

const API = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

function toCellTuple(cell: any): [number, number] {
  if (Array.isArray(cell) && cell.length >= 2) {
    return [Number(cell[0]), Number(cell[1])];
  }
  if (cell && typeof cell === 'object' && 'x' in cell && 'y' in cell) {
    return [Number(cell.x), Number(cell.y)];
  }
  return [0, 0];
}

function backendMazeToMazeData(backendMaze: any): MazeData {
  const width = Number(backendMaze?.grid?.width ?? 0);
  const height = Number(backendMaze?.grid?.height ?? 0);

  const start = toCellTuple(backendMaze?.start);
  const goal = toCellTuple(backendMaze?.goal);

  // Rust `Walls` serializes as `{ inner: HashSet<Edge> }`.
  const rawEdges: any[] = Array.isArray(backendMaze?.walls?.inner)
    ? backendMaze.walls.inner
    : [];

  const walls: MazeData['walls'] = rawEdges
    .map((edge: any) => {
      // Rust tuple structs like `Edge(Cell, Cell)` usually serialize as `[cellA, cellB]`.
      if (Array.isArray(edge) && edge.length === 2) {
        return [toCellTuple(edge[0]), toCellTuple(edge[1])] as [
          [number, number],
          [number, number],
        ];
      }
      // Fallback for other shapes like `{ 0: cellA, 1: cellB }`.
      if (edge && typeof edge === 'object' && 0 in edge && 1 in edge) {
        return [toCellTuple(edge[0]), toCellTuple(edge[1])] as [
          [number, number],
          [number, number],
        ];
      }
      return null;
    })
    .filter(Boolean) as MazeData['walls'];

  return { width, height, walls, start, goal };
}

export default function Home() {
  const [solver, setSolver] = useState('ASTAR');

  const [maze, setMaze] = useState<MazeData | null>(null);
  const [mazeId, setMazeId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [runId, setRunId] = useState<string | null>(null);
  const [solveLoading, setSolveLoading] = useState(false);

  const {
    status: solveStreamStatus,
    frames,
    stats,
    error: solveStreamError,
  } = useSolveStream(runId);

  const frame = frames[frames.length - 1];

  const handleGenerate = async (params: GenerateFormParams) => {
    setLoading(true);
    setError(null);
    setRunId(null);

    try {
      const res = await fetch(`${API}/api/maze/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          w: params.w,
          h: params.h,
          seed: params.seed,
          algo: params.algo,
        }),
      });

      if (!res.ok) {
        const err = await res.json().catch(() => ({}));
        throw new Error((err as any).error || res.statusText);
      }

      const data = await res.json();
      setMazeId(data.mazeId);
      setMaze(backendMazeToMazeData(data.maze));
    } catch (e: any) {
      setError(e?.message ?? 'Failed to generate maze');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <div className="flex flex-col items-center gap-6 p-8">
        <SolverPicker value={solver} onChange={setSolver} />
        <GenerateForm onSubmit={handleGenerate} loading={loading} />

        {error ? <div className="text-red-600">{error}</div> : null}
        <MazeGrid
          maze={maze}
          frontier={frame?.frontier}
          visited={frame?.visited}
        />

        <button
          onClick={async () => {
            if (!mazeId) return;
            if (solveLoading) return;

            setSolveLoading(true);
            setError(null);
            setRunId(null);

            try {
              const res = await fetch(`${API}/api/solve`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ mazeId, solver }),
              });

              if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                throw new Error((err as any).error || res.statusText);
              }

              const data = await res.json();
              setRunId(data.runId);
            } catch (e: any) {
              setError(e?.message ?? 'Solve failed');
            } finally {
              setSolveLoading(false);
            }
          }}
          disabled={!mazeId || solveLoading}
          className="bg-green-500 text-white px-4 py-2 rounded disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {solveLoading ? 'Solving...' : 'Solve'}
        </button>

        {/* `mazeId` is stored now; Commit 59 will use it for the Solve button. */}
        <div className="text-sm text-zinc-600">
          {mazeId ? `mazeId: ${mazeId}` : 'no maze yet'}
        </div>

        <div className="text-sm text-zinc-600">
          {runId ? `runId: ${runId}` : null}
        </div>

        {runId ? (
          <div className="text-sm text-zinc-600">
            stream: {solveStreamStatus}
            {solveStreamError ? (
              <span className="text-red-600"> — {solveStreamError}</span>
            ) : null}
            {stats && solveStreamStatus === 'finished'
              ? ` | visited ${stats.visited} cost ${stats.cost} ${stats.ms}ms`
              : null}
          </div>
        ) : null}
      </div>
    </div>
  );
}
