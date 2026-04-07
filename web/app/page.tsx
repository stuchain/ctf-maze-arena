'use client';

import { useEffect, useState } from 'react';
import { Achievements } from '../components/Achievements';
import { Leaderboard, type LeaderboardEntry } from '../components/Leaderboard';
import { MazeGrid, type MazeData } from '../components/MazeGrid';
import { GenerateForm, type GenerateFormParams } from '../components/GenerateForm';
import { SolverPicker } from '../components/SolverPicker';
import { useSolveStream } from '../hooks/useSolveStream';
import { backendMazeToMazeData } from '@/lib/maze';

const API = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export default function Home() {
  const [solver, setSolver] = useState('ASTAR');

  const [maze, setMaze] = useState<MazeData | null>(null);
  const [mazeId, setMazeId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [runId, setRunId] = useState<string | null>(null);
  const [solveLoading, setSolveLoading] = useState(false);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [dailyInfo, setDailyInfo] = useState<{
    seed: number;
    date: string;
  } | null>(null);
  const [achievementsRefresh, setAchievementsRefresh] = useState(0);

  useEffect(() => {
    if (!mazeId) {
      setLeaderboard([]);
      return;
    }
    fetch(`${API}/api/leaderboard?mazeId=${encodeURIComponent(mazeId)}`)
      .then((r) => (r.ok ? r.json() : Promise.resolve([])))
      .then((data) =>
        setLeaderboard(Array.isArray(data) ? data : []),
      )
      .catch(() => setLeaderboard([]));
  }, [mazeId]);

  const {
    status: solveStreamStatus,
    frames,
    path: solvePath,
    stats,
    error: solveStreamError,
  } = useSolveStream(runId, solver);

  useEffect(() => {
    if (solveStreamStatus === 'finished') {
      setAchievementsRefresh((v) => v + 1);
    }
  }, [solveStreamStatus]);

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

  const handleDaily = async () => {
    setError(null);
    try {
      const res = await fetch(`${API}/api/daily`);
      if (!res.ok) throw new Error('Daily challenge unavailable');
      const data = await res.json();
      const seed = Number(data.seed);
      const w = Number(data.w);
      const h = Number(data.h);
      setDailyInfo({ seed, date: String(data.date ?? '') });
      await handleGenerate({ w, h, seed, algo: 'KRUSKAL' });
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Daily challenge failed';
      setError(msg);
    }
  };

  return (
    <main
      id="main-content"
      className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black"
    >
      <div className="flex flex-col items-center gap-6 p-8">
        <label htmlFor="solver-picker" className="text-sm text-zinc-700 dark:text-zinc-300">
          Solver
        </label>
        <SolverPicker value={solver} onChange={setSolver} id="solver-picker" />
        <button
          type="button"
          onClick={() => void handleDaily()}
          disabled={loading}
          className="rounded bg-violet-600 px-4 py-2 text-white text-sm disabled:opacity-50"
        >
          Daily Challenge
        </button>
        {dailyInfo ? (
          <p className="text-sm text-zinc-600">
            Today&apos;s seed: {dailyInfo.seed}
            {dailyInfo.date ? ` (${dailyInfo.date})` : null}
          </p>
        ) : null}
        <GenerateForm onSubmit={handleGenerate} loading={loading} />

        {error ? <div className="text-red-600">{error}</div> : null}
        <MazeGrid
          maze={maze}
          frontier={frame?.frontier}
          visited={frame?.visited}
          current={frame?.current}
          path={solveStreamStatus === 'finished' ? solvePath : undefined}
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
          data-testid="solve-button"
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

        <Achievements refreshVersion={achievementsRefresh} />

        <div className="w-full max-w-md">
          <h2 className="text-sm font-semibold text-zinc-700 mb-2">Leaderboard</h2>
          <Leaderboard entries={leaderboard} />
        </div>

        {runId ? (
          <div
            className="text-sm text-zinc-600"
            data-testid="stream-status"
            role={solveStreamError ? 'alert' : 'status'}
            aria-live={solveStreamError ? 'assertive' : 'polite'}
          >
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
    </main>
  );
}
