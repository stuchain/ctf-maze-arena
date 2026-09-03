'use client';

import { useEffect, useState } from 'react';
import { useSession } from 'next-auth/react';
import { Achievements } from '@/components/Achievements';
import { AppHeader } from '@/components/AppHeader';
import { GenerateForm, type GenerateFormParams } from '@/components/GenerateForm';
import { Leaderboard, type LeaderboardEntry } from '@/components/Leaderboard';
import { MazeGrid, type MazeData } from '@/components/MazeGrid';
import { SolverPicker } from '@/components/SolverPicker';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { Badge, Button, Field, Notice, Panel, PanelHeader } from '@/components/ui/Primitives';
import { useSolveStream, type StreamStatus } from '@/hooks/useSolveStream';
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
import { backendMazeToMazeData } from '@/lib/maze';

const API = publicEnv.NEXT_PUBLIC_API_URL;
const ACTIVE_STATUSES: StreamStatus[] = ['waking', 'connecting', 'live', 'reconnecting'];
const STATUS_LABELS: Record<StreamStatus, string> = {
  idle: 'Standby',
  waking: 'Waking Arena',
  connecting: 'Connecting',
  live: 'Live Run',
  reconnecting: 'Reconnecting',
  completed: 'Completed',
  failed: 'Failed',
  cancelled: 'Cancelled',
};

function statusTone(status: StreamStatus): 'neutral' | 'info' | 'success' | 'warning' | 'danger' {
  if (status === 'completed') return 'success';
  if (status === 'failed' || status === 'cancelled') return 'danger';
  if (status === 'waking' || status === 'reconnecting') return 'warning';
  if (status === 'connecting' || status === 'live') return 'info';
  return 'neutral';
}

function Metric({ label, value }: { label: string; value: string | number }) {
  return <div className="metric"><span>{label}</span><strong>{value}</strong></div>;
}

function MazeLegend() {
  return (
    <ul className="maze-legend" aria-label="Maze state legend">
      {[
        ['Start', 'start'], ['Goal', 'goal'], ['Frontier', 'frontier'],
        ['Visited', 'visited'], ['Current', 'current'], ['Path', 'path'],
      ].map(([label, state]) => (
        <li key={state}><span className={`legend-swatch legend-swatch--${state}`} aria-hidden="true" />{label}</li>
      ))}
    </ul>
  );
}

function formatChallengeDate(value: string) {
  return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric', timeZone: 'UTC' })
    .format(new Date(`${value}T00:00:00Z`));
}

export default function Home() {
  const { status: authStatus } = useSession();
  const [solver, setSolver] = useState('ASTAR');
  const [maze, setMaze] = useState<MazeData | null>(null);
  const [mazeId, setMazeId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [runId, setRunId] = useState<string | null>(null);
  const [solveLoading, setSolveLoading] = useState(false);
  const [cancelLoading, setCancelLoading] = useState(false);
  const [confirmCancel, setConfirmCancel] = useState(false);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [submissionStatus, setSubmissionStatus] = useState<string | null>(null);
  const [dailyInfo, setDailyInfo] = useState<{ seed: number; date: string } | null>(null);

  useEffect(() => {
    if (!mazeId) {
      setLeaderboard([]);
      return;
    }
    requestJson(`${API}/api/leaderboard?mazeId=${encodeURIComponent(mazeId)}`, leaderboardResponseSchema)
      .then(setLeaderboard)
      .catch(() => setLeaderboard([]));
  }, [mazeId]);

  const {
    status: solveStreamStatus, frames, path: solvePath, stats,
    error: solveStreamError, sequence: solveSequence,
  } = useSolveStream(runId, solver);
  const frame = frames[frames.length - 1];
  const isActive = ACTIVE_STATUSES.includes(solveStreamStatus);

  const authHeaders = async (): Promise<Record<string, string>> => {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (authStatus !== 'authenticated') return headers;
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
        body: JSON.stringify(params),
      });
      setMazeId(data.mazeId);
      setMaze(backendMazeToMazeData(data.maze));
    } catch (cause: unknown) {
      setError(`${toErrorMessage(cause, 'Could not generate the maze.')} Check the API connection and try again.`);
    } finally {
      setLoading(false);
    }
  };

  const handleDaily = async () => {
    setError(null);
    try {
      const data = await requestJson(`${API}/api/daily`, dailyResponseSchema);
      setDailyInfo({ seed: data.seed, date: data.date });
      await handleGenerate({ w: data.w, h: data.h, seed: data.seed, algo: 'KRUSKAL' });
    } catch (cause: unknown) {
      setError(`${toErrorMessage(cause, 'Could not load today’s challenge.')} Try a custom maze instead.`);
    }
  };

  const handleSolve = async () => {
    if (!mazeId || solveLoading) return;
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
      setError(`${toErrorMessage(cause, 'Could not start the solver.')} Wait a moment and try again.`);
    } finally {
      setSolveLoading(false);
    }
  };

  const handleCancel = async () => {
    if (!runId) return;
    setCancelLoading(true);
    setError(null);
    try {
      await requestJson(`${API}/api/run/${encodeURIComponent(runId)}/cancel`, cancelResponseSchema, {
        method: 'POST', headers: await authHeaders(),
      });
      setConfirmCancel(false);
    } catch (cause: unknown) {
      setError(`${toErrorMessage(cause, 'Could not cancel the run.')} Refresh its status and try again.`);
    } finally {
      setCancelLoading(false);
    }
  };

  const handleSubmitScore = async () => {
    if (!runId) return;
    setSubmissionStatus('Submitting score…');
    setError(null);
    try {
      const result = await requestJson(`${API}/api/leaderboard`, leaderboardSubmitResponseSchema, {
        method: 'POST', headers: await authHeaders(), body: JSON.stringify({ runId }),
      });
      setSubmissionStatus(result.duplicate ? 'Score already submitted.' : 'Score submitted.');
      if (mazeId) {
        setLeaderboard(await requestJson(
          `${API}/api/leaderboard?mazeId=${encodeURIComponent(mazeId)}`,
          leaderboardResponseSchema,
        ));
      }
    } catch (cause: unknown) {
      setSubmissionStatus(null);
      setError(`${toErrorMessage(cause, 'Could not submit the score.')} Confirm you are signed in and try again.`);
    }
  };

  return (
    <div className="app-frame">
      <AppHeader />
      <nav className="mobile-lab-nav" aria-label="Workspace sections">
        <a href="#arena">Arena</a><a href="#configuration">Configure</a><a href="#inspector">Inspect</a>
      </nav>

      <main id="main-content" className="workspace" tabIndex={-1}>
        <Panel as="aside" id="configuration" className="configuration-panel">
          <PanelHeader eyebrow="01 · Configure" title="Build the Challenge" description="Every seed is deterministic. Tune the arena, then replay it exactly." />
          <div className="preset-card">
            <div>
              <span className="preset-card__label">Daily Seed</span>
              <strong>{dailyInfo ? dailyInfo.seed.toLocaleString() : 'Fresh at 00:00 UTC'}</strong>
              <small>{dailyInfo?.date ? formatChallengeDate(dailyInfo.date) : 'Same challenge for everyone'}</small>
            </div>
            <Button variant="secondary" size="sm" onClick={() => void handleDaily()} loading={loading}>Load Daily</Button>
          </div>
          <GenerateForm onSubmit={handleGenerate} loading={loading} />
          <div className="panel-divider" />
          <Field label="Pathfinding Strategy" htmlFor="solver-picker" hint="A* balances optimal paths with focused exploration.">
            <SolverPicker value={solver} onChange={setSolver} id="solver-picker" describedBy="solver-picker-description" />
          </Field>
        </Panel>

        <Panel id="arena" className="arena-panel">
          <div className="arena-heading">
            <div><p className="eyebrow">02 · Live Arena</p><h1>Watch the Search Unfold</h1></div>
            <Badge tone={statusTone(solveStreamStatus)} pulse={isActive}>{STATUS_LABELS[solveStreamStatus]}</Badge>
          </div>
          {error ? <Notice title="Action Needed" tone="danger">{error}</Notice> : null}
          {solveStreamError ? <Notice title="Stream Interrupted" tone="warning">{solveStreamError}</Notice> : null}
          <div className="stage-shell">
            <div className="stage-grid" aria-hidden="true" />
            <MazeGrid
              maze={maze} frontier={frame?.frontier} visited={frame?.visited}
              current={frame?.current} path={solveStreamStatus === 'completed' ? solvePath : undefined}
            />
          </div>
          <div className="arena-toolbar">
            <MazeLegend />
            <div className="arena-actions">
              {isActive ? <Button variant="destructive" onClick={() => setConfirmCancel(true)}>Cancel Run</Button> : null}
              <Button
                onClick={() => void handleSolve()} disabled={!mazeId || isActive}
                loading={solveLoading} data-testid="solve-button"
              >
                {solveLoading ? 'Starting Solver…' : solveStreamStatus === 'completed' ? 'Run Again' : 'Start Solver'}
              </Button>
            </div>
          </div>
        </Panel>

        <Panel as="aside" id="inspector" className="inspector-panel">
          <PanelHeader eyebrow="03 · Inspect" title="Run Telemetry" description="Live signals from the active search." />
          <div
            className="run-status" data-testid="stream-status"
            role={solveStreamError ? 'alert' : 'status'}
            aria-live={solveStreamError ? 'assertive' : 'polite'}
          >
            <div className="run-status__line">
              <span>Stream</span><strong>{solveStreamStatus}</strong>
              {solveSequence > 0 ? <small>sequence {solveSequence}</small> : null}
            </div>
            <div className="metric-grid">
              <Metric label="Visited" value={stats?.visited.toLocaleString() ?? frame?.visited.length.toLocaleString() ?? '—'} />
              <Metric label="Path Cost" value={stats?.cost ?? '—'} />
              <Metric label="Runtime" value={stats ? `${stats.ms} ms` : '—'} />
              <Metric label="Solver" value={solver.replace('_', ' ')} />
            </div>
          </div>
          {solveStreamStatus === 'completed' && authStatus === 'authenticated' ? (
            <Button className="button--full" variant="secondary" onClick={() => void handleSubmitScore()} loading={submissionStatus === 'Submitting score…'}>
              Submit Ranked Score
            </Button>
          ) : null}
          {submissionStatus ? <Notice title="Leaderboard" tone="success">{submissionStatus}</Notice> : null}
          <div className="panel-divider" />
          <div className="section-heading"><h3>Achievements</h3><span>Local Progress</span></div>
          <Achievements />
        </Panel>
      </main>

      <section className="secondary-grid" aria-label="Arena intelligence">
        <Panel>
          <PanelHeader eyebrow="Community" title="Maze Leaderboard" description="Ranked by path cost, runtime, then explored cells." />
          <Leaderboard entries={leaderboard} />
        </Panel>
        <Panel className="concept-panel">
          <PanelHeader eyebrow="Algorithm Note" title="Why A* Feels Focused" description="A* combines distance travelled with a heuristic estimate to prioritize promising cells." />
          <div className="formula" translate="no"><span>f(n)</span><b>=</b><span>g(n)</span><b>+</b><span>h(n)</span></div>
          <div className="formula-key"><span><i>g</i> Known cost from start</span><span><i>h</i> Estimated cost to goal</span></div>
        </Panel>
      </section>

      <footer className="app-footer"><span>Built to make algorithm behavior visible.</span><span translate="no">Protocol v1 · Deterministic Replays</span></footer>
      <ConfirmDialog
        open={confirmCancel} title="Cancel This Solver Run?"
        description="The current exploration will stop and the run will be recorded as cancelled."
        confirmLabel="Cancel Run" loading={cancelLoading}
        onCancel={() => setConfirmCancel(false)} onConfirm={() => void handleCancel()}
      />
    </div>
  );
}
