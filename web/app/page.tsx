'use client';

import { useEffect, useState } from 'react';
import { Achievements } from '../components/Achievements';
import { Leaderboard, type LeaderboardEntry } from '../components/Leaderboard';
import { MazeGrid, type MazeData } from '../components/MazeGrid';
import { GenerateForm, type GenerateFormParams } from '../components/GenerateForm';
import { SolverPicker } from '../components/SolverPicker';
import { useSolveStream } from '../hooks/useSolveStream';
import { backendMazeToMazeData } from '@/lib/maze';
import { signIn, signOut, useSession } from 'next-auth/react';
import {
  cancelResponseSchema,
  dailyResponseSchema,
  generateResponseSchema,
  leaderboardSubmitResponseSchema,
  leaderboardResponseSchema,
  requestJson,
  solveResponseSchema,
  toErrorMessage,
  tokenResponseSchema,
} from '@/lib/api';
import { publicEnv } from '@/lib/env';

const API = publicEnv.NEXT_PUBLIC_API_URL;

export default function Home() {
  const { data: session, status: authStatus } = useSession();
  const authEnabled = publicEnv.NEXT_PUBLIC_AUTH_MODE !== 'anonymous';
  const [solver, setSolver] = useState('ASTAR');

  const [maze, setMaze] = useState<MazeData | null>(null);
  const [mazeId, setMazeId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [runId, setRunId] = useState<string | null>(null);
  const [solveLoading, setSolveLoading] = useState(false);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [submissionStatus, setSubmissionStatus] = useState<string | null>(null);
  const [dailyInfo, setDailyInfo] = useState<{
    seed: number;
    date: string;
  } | null>(null);

  useEffect(() => {
    if (!mazeId) {
      setLeaderboard([]);
      return;
    }
    requestJson(
      `${API}/api/leaderboard?mazeId=${encodeURIComponent(mazeId)}`,
      leaderboardResponseSchema,
    )
      .then(setLeaderboard)
      .catch(() => setLeaderboard([]));
  }, [mazeId]);

  const {
    status: solveStreamStatus,
    frames,
    path: solvePath,
    stats,
    error: solveStreamError,
    sequence: solveSequence,
  } = useSolveStream(runId, solver);

  const frame = frames[frames.length - 1];

  const authHeaders = async (): Promise<Record<string, string>> => {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (authStatus !== 'authenticated') {
      return headers;
    }

    try {
      const tokenData = await requestJson('/api/token', tokenResponseSchema);
      headers.Authorization = `Bearer ${tokenData.token}`;
    } catch {
      return headers;
    }
    return headers;
  };

  const handleGenerate = async (params: GenerateFormParams) => {
    setLoading(true);
    setError(null);
    setRunId(null);
    setSubmissionStatus(null);

    try {
      const data = await requestJson(`${API}/api/maze/generate`, generateResponseSchema, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          w: params.w,
          h: params.h,
          seed: params.seed,
          algo: params.algo,
        }),
      });
      setMazeId(data.mazeId);
      setMaze(backendMazeToMazeData(data.maze));
    } catch (cause: unknown) {
      setError(toErrorMessage(cause, 'Failed to generate maze'));
    } finally {
      setLoading(false);
    }
  };

  const handleDaily = async () => {
    setError(null);
    try {
      const data = await requestJson(`${API}/api/daily`, dailyResponseSchema);
      const { seed, w, h } = data;
      setDailyInfo({ seed, date: String(data.date ?? '') });
      await handleGenerate({ w, h, seed, algo: 'KRUSKAL' });
    } catch (e: unknown) {
      setError(toErrorMessage(e, 'Daily challenge failed'));
    }
  };

  return (
    <main
      id="main-content"
      className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black"
    >
      <div className="flex flex-col items-center gap-6 p-8">
        <div className="flex items-center gap-3 text-sm">
          {!authEnabled ? (
            <span>Play anonymously. GitHub profiles are disabled in this environment.</span>
          ) : authStatus === 'authenticated' ? (
            <>
              <span>
                Signed in as {session?.user?.name ?? session?.user?.email ?? 'GitHub user'}
              </span>
              <button
                type="button"
                onClick={() => void signOut()}
                className="rounded border px-3 py-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-600 focus-visible:ring-offset-1"
              >
                Sign out
              </button>
            </>
          ) : (
            <>
              <span>Sign in to submit authenticated leaderboard scores.</span>
              <button
                type="button"
                onClick={() => void signIn('github')}
                className="rounded border px-3 py-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-600 focus-visible:ring-offset-1"
              >
                Sign in with GitHub
              </button>
            </>
          )}
        </div>
        <label htmlFor="solver-picker" className="text-sm text-zinc-700 dark:text-zinc-300">
          Solver
        </label>
        <SolverPicker value={solver} onChange={setSolver} id="solver-picker" />
        <button
          type="button"
          onClick={() => void handleDaily()}
          disabled={loading}
          className="rounded bg-violet-600 px-4 py-2 text-white text-sm disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-violet-300 focus-visible:ring-offset-2"
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
          path={solveStreamStatus === 'completed' ? solvePath : undefined}
        />

        <button
          onClick={async () => {
            if (!mazeId) return;
            if (solveLoading) return;

            setSolveLoading(true);
            setError(null);
            setRunId(null);

            try {
              const data = await requestJson(`${API}/api/solve`, solveResponseSchema, {
                method: 'POST',
                headers: await authHeaders(),
                body: JSON.stringify({ mazeId, solver }),
              });
              setRunId(data.runId);
              setSubmissionStatus(null);
            } catch (cause: unknown) {
              setError(toErrorMessage(cause, 'Solve failed'));
            } finally {
              setSolveLoading(false);
            }
          }}
          disabled={!mazeId || solveLoading}
          className="bg-green-500 text-white px-4 py-2 rounded disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-300 focus-visible:ring-offset-2"
          data-testid="solve-button"
        >
          {solveLoading ? 'Solving...' : 'Solve'}
        </button>

        {runId && ['waking', 'connecting', 'live', 'reconnecting'].includes(solveStreamStatus) ? (
          <button
            type="button"
            onClick={async () => {
              try {
                await requestJson(`${API}/api/run/${encodeURIComponent(runId)}/cancel`, cancelResponseSchema, {
                  method: 'POST',
                  headers: await authHeaders(),
                });
              } catch (cause: unknown) {
                setError(toErrorMessage(cause, 'Cancellation failed'));
              }
            }}
            className="rounded border border-red-500 px-4 py-2 text-sm text-red-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-300"
          >
            Cancel solve
          </button>
        ) : null}

        {runId && solveStreamStatus === 'completed' && authStatus === 'authenticated' ? (
          <button
            type="button"
            onClick={async () => {
              setSubmissionStatus('Submitting score…');
              setError(null);
              try {
                const result = await requestJson(
                  `${API}/api/leaderboard`,
                  leaderboardSubmitResponseSchema,
                  {
                    method: 'POST',
                    headers: await authHeaders(),
                    body: JSON.stringify({ runId }),
                  },
                );
                setSubmissionStatus(result.duplicate ? 'Score already submitted.' : 'Score submitted!');
                if (mazeId) {
                  const entries = await requestJson(
                    `${API}/api/leaderboard?mazeId=${encodeURIComponent(mazeId)}`,
                    leaderboardResponseSchema,
                  );
                  setLeaderboard(entries);
                }
              } catch (cause: unknown) {
                setSubmissionStatus(null);
                setError(toErrorMessage(cause, 'Score submission failed'));
              }
            }}
            disabled={submissionStatus === 'Submitting score…'}
            className="rounded bg-blue-600 px-4 py-2 text-sm text-white disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-300 focus-visible:ring-offset-2"
          >
            Submit score
          </button>
        ) : null}
        {submissionStatus ? <p role="status" className="text-sm text-zinc-600">{submissionStatus}</p> : null}

        <Achievements />

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
            {solveSequence > 0 ? ` | sequence ${solveSequence}` : null}
            {stats && solveStreamStatus === 'completed'
              ? ` | visited ${stats.visited} cost ${stats.cost} ${stats.ms}ms`
              : null}
          </div>
        ) : null}
      </div>
    </main>
  );
}
