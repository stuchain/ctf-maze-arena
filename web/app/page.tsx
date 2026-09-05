'use client';

import { useEffect, useState } from 'react';
import { useSession } from 'next-auth/react';
import { Achievements } from '@/components/Achievements';
import { AppHeader } from '@/components/AppHeader';
import { AlgorithmGuide } from '@/components/AlgorithmGuide';
import { GenerateForm, type GenerateFormParams } from '@/components/GenerateForm';
import { Leaderboard, type LeaderboardEntry } from '@/components/Leaderboard';
import { MazeGrid, type MazeData } from '@/components/MazeGrid';
import { PlaybackControls } from '@/components/PlaybackControls';
import { RaceExperience } from '@/components/RaceExperience';
import { SolverPicker } from '@/components/SolverPicker';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { Badge, Button, Field, Notice, Panel, PanelHeader } from '@/components/ui/Primitives';
import { useSolveStream, type StreamStatus } from '@/hooks/useSolveStream';
import { useRaceStreams, type RaceRunMap } from '@/hooks/useRaceStreams';
import { usePlaybackTimeline } from '@/hooks/usePlaybackTimeline';
import {
  cancelResponseSchema,
  dailyResponseSchema,
  generateResponseSchema,
  leaderboardSubmitResponseSchema,
  leaderboardResponseSchema,
  requestJson,
  solveResponseSchema,
  raceResponseSchema,
  toErrorMessage,
  tokenResponseSchema,
} from '@/lib/api';
import { publicEnv } from '@/lib/env';
import { backendMazeToMazeData } from '@/lib/maze';
import { canonicalRaceUrl, DEFAULT_RACE_CONFIG, parseRaceConfig, RACE_SOLVERS, type RaceConfig, type RaceDisplayMode, type RaceSolver } from '@/lib/race';

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

const SOLVER_GUARANTEES: Record<string, string> = {
  BFS: 'Shortest path in an unweighted maze',
  DFS: 'Complete search; path is not guaranteed shortest',
  ASTAR: 'Shortest path with an admissible heuristic',
  DP_KEYS: 'Complete key-aware state search',
};

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
  const [experienceMode, setExperienceMode] = useState<'single' | 'race'>('single');
  const [raceSolvers, setRaceSolvers] = useState<RaceSolver[]>(DEFAULT_RACE_CONFIG.solvers);
  const [raceDisplayMode, setRaceDisplayMode] = useState<RaceDisplayMode>('overview');
  const [raceId, setRaceId] = useState<string | null>(null);
  const [raceRunIds, setRaceRunIds] = useState<RaceRunMap>({});
  const [raceLoading, setRaceLoading] = useState(false);
  const [shareStatus, setShareStatus] = useState<string | null>(null);
  const [generationConfig, setGenerationConfig] = useState<GenerateFormParams | null>(null);
  const [formDefaults, setFormDefaults] = useState<GenerateFormParams | null>(null);

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
  const raceStreams = useRaceStreams(raceRunIds);
  const playback = usePlaybackTimeline(frames.length, 'live', runId);
  const frame = frames[playback.displayIndex];
  const isActive = ACTIVE_STATUSES.includes(solveStreamStatus);
  const selectedRaceStreams = raceSolvers.map((candidate) => raceStreams[candidate]);
  const raceIsActive = Boolean(raceId) && selectedRaceStreams.some((stream) => ACTIVE_STATUSES.includes(stream.status));
  const raceIsComplete = Boolean(raceId) && selectedRaceStreams.every((stream) => stream.status === 'completed');

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
    setRaceId(null);
    setRaceRunIds({});
    if (params.featurePreset === 'classic') setRaceSolvers((current) => current.filter((value) => value !== 'DP_KEYS'));
    try {
      const data = await requestJson(`${API}/api/maze/generate`, generateResponseSchema, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(params),
      });
      setMazeId(data.mazeId);
      setMaze(backendMazeToMazeData(data.maze));
      setGenerationConfig(params);
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
      await handleGenerate({ w: data.w, h: data.h, seed: data.seed, algo: 'KRUSKAL', featurePreset: 'classic' });
    } catch (cause: unknown) {
      setError(`${toErrorMessage(cause, 'Could not load today’s challenge.')} Try a custom maze instead.`);
    }
  };

  const currentRaceConfig = (): RaceConfig => ({
    version: 1,
    width: generationConfig?.w ?? DEFAULT_RACE_CONFIG.width,
    height: generationConfig?.h ?? DEFAULT_RACE_CONFIG.height,
    seed: generationConfig?.seed ?? DEFAULT_RACE_CONFIG.seed,
    generator: (generationConfig?.algo as RaceConfig['generator']) ?? DEFAULT_RACE_CONFIG.generator,
    featurePreset: generationConfig?.featurePreset ?? 'classic',
    solvers: raceSolvers,
    displayMode: raceDisplayMode,
  });

  const handleRace = async () => {
    if (!mazeId || raceLoading || raceSolvers.length < 2) return;
    setRaceLoading(true); setError(null); setRunId(null); setRaceId(null); setRaceRunIds({});
    try {
      const data = await requestJson(`${API}/api/race`, raceResponseSchema, {
        method: 'POST', headers: await authHeaders(), body: JSON.stringify({ mazeId, solvers: raceSolvers }),
      });
      setRaceId(data.raceId);
      setRaceRunIds(Object.fromEntries(data.runs.map((run) => [run.solver, run.runId])) as RaceRunMap);
    } catch (cause: unknown) {
      setError(`${toErrorMessage(cause, 'Could not start the race.')} Check the competitor selection and try again.`);
    } finally { setRaceLoading(false); }
  };

  const handleCancelRace = async () => {
    const headers = await authHeaders();
    await Promise.allSettled(Object.values(raceRunIds).map((id) => requestJson(
      `${API}/api/run/${encodeURIComponent(id)}/cancel`, cancelResponseSchema, { method: 'POST', headers },
    )));
  };

  const handleShareRace = async () => {
    if (!generationConfig) { setError('Generate a maze before sharing its race configuration.'); return; }
    const url = canonicalRaceUrl(currentRaceConfig(), window.location);
    window.history.replaceState(null, '', url);
    try { await navigator.clipboard.writeText(url); setShareStatus('Race link copied.'); }
    catch { setShareStatus('Race URL is ready in the address bar.'); }
  };

  const handleLoadSharedRace = async () => {
    const config = parseRaceConfig(new URLSearchParams(window.location.search));
    if (!config) { setError('This URL does not contain a supported v1 race configuration.'); return; }
    setExperienceMode('race'); setRaceSolvers(config.solvers); setRaceDisplayMode(config.displayMode);
    const params = { w: config.width, h: config.height, seed: config.seed, algo: config.generator, featurePreset: config.featurePreset } as const;
    setFormDefaults(params);
    await handleGenerate(params);
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
          <GenerateForm key={formDefaults ? `${formDefaults.w}:${formDefaults.h}:${formDefaults.seed}:${formDefaults.algo}:${formDefaults.featurePreset}` : 'custom'} initialParams={formDefaults ?? undefined} onSubmit={handleGenerate} loading={loading} />
          <div className="panel-divider" />
          <fieldset className="experience-picker"><legend>Experience</legend><div className="segmented-control">
            <button type="button" aria-pressed={experienceMode === 'single'} onClick={() => setExperienceMode('single')}>Single Solver</button>
            <button type="button" aria-pressed={experienceMode === 'race'} onClick={() => setExperienceMode('race')}>Algorithm Race</button>
          </div></fieldset>
          {experienceMode === 'single' ? <Field label="Pathfinding Strategy" htmlFor="solver-picker" hint="A* balances optimal paths with focused exploration.">
            <SolverPicker value={solver} onChange={setSolver} id="solver-picker" describedBy="solver-picker-description" />
          </Field> : <div className="race-configuration">
            <fieldset><legend>Competitors <small>Choose 2–4</small></legend>{RACE_SOLVERS.map((candidate) => {
              const incompatible = candidate === 'DP_KEYS' && generationConfig?.featurePreset !== 'keys';
              return <label key={candidate}><input type="checkbox" checked={raceSolvers.includes(candidate)} disabled={incompatible} onChange={() => setRaceSolvers((current) => current.includes(candidate) ? current.filter((value) => value !== candidate) : [...current, candidate])} />{candidate.replace('_', ' ')}{incompatible ? ' (keys preset only)' : ''}</label>;
            })}</fieldset>
            <fieldset><legend>Display</legend><div className="segmented-control"><button type="button" aria-pressed={raceDisplayMode === 'overview'} onClick={() => setRaceDisplayMode('overview')}>Overview</button><button type="button" aria-pressed={raceDisplayMode === 'side-by-side'} onClick={() => setRaceDisplayMode('side-by-side')}>Side by Side</button></div></fieldset>
            <Button type="button" variant="secondary" onClick={() => void handleLoadSharedRace()}>Load Race from URL</Button>
            <Button type="button" variant="secondary" onClick={() => void handleShareRace()}>Share Configuration</Button>
            {shareStatus ? <small role="status">{shareStatus}</small> : null}
          </div>}
        </Panel>

        <Panel id="arena" className="arena-panel">
          <div className="arena-heading">
            <div><p className="eyebrow">02 · Live Arena</p><h1>{experienceMode === 'race' ? 'Compare Every Decision' : 'Watch the Search Unfold'}</h1></div>
            {experienceMode === 'race' ? <Badge tone={raceIsComplete ? 'success' : raceIsActive ? 'info' : 'neutral'} pulse={raceIsActive}>{raceIsComplete ? 'Race Complete' : raceIsActive ? 'Race Live' : 'Race Ready'}</Badge> : <Badge tone={statusTone(solveStreamStatus)} pulse={isActive}>{STATUS_LABELS[solveStreamStatus]}</Badge>}
          </div>
          {error ? <Notice title="Action Needed" tone="danger">{error}</Notice> : null}
          {solveStreamError ? <Notice title="Stream Interrupted" tone="warning">{solveStreamError}</Notice> : null}
          {experienceMode === 'race' && maze && raceId ? <RaceExperience maze={maze} config={currentRaceConfig()} raceId={raceId} runIds={raceRunIds} streams={raceStreams} onCancel={() => void handleCancelRace()} /> : <><div className="stage-shell">
            <div className="stage-grid" aria-hidden="true" />
            <MazeGrid
              key={mazeId ?? 'empty'}
              maze={maze} frontier={frame?.frontier} visited={frame?.visited}
              current={frame?.current} path={solveStreamStatus === 'completed' && playback.atEnd ? solvePath : undefined}
            />
          </div>
          {experienceMode === 'single' && frames.length ? (
            <PlaybackControls
              mode="live" totalFrames={frames.length} currentIndex={playback.displayIndex}
              playing={playback.playing} followLive={playback.followLive} speed={playback.speed}
              onPlay={playback.play} onPause={playback.pause} onReset={playback.reset}
              onPrevious={playback.previous} onNext={playback.next} onGoLive={playback.goLive}
              onIndexChange={playback.setIndex} onSpeedChange={playback.setSpeed}
            />
          ) : null}</>}
          <div className="arena-toolbar">
            <MazeLegend />
            <div className="arena-actions">
              {experienceMode === 'single' && isActive ? <Button variant="destructive" onClick={() => setConfirmCancel(true)}>Cancel Run</Button> : null}
              {experienceMode === 'single' ? <Button
                onClick={() => void handleSolve()} disabled={!mazeId || isActive}
                loading={solveLoading} data-testid="solve-button"
              >
                {solveLoading ? 'Starting Solver…' : solveStreamStatus === 'completed' ? 'Run Again' : 'Start Solver'}
              </Button> : <Button onClick={() => void handleRace()} disabled={!mazeId || raceSolvers.length < 2} loading={raceLoading} data-testid="race-button">{raceLoading ? 'Starting Race…' : raceId ? 'Race Again' : 'Start Algorithm Race'}</Button>}
            </div>
          </div>
        </Panel>

        <Panel as="aside" id="inspector" className="inspector-panel">
          <PanelHeader eyebrow="03 · Inspect" title="Run Telemetry" description="Live signals from the active search." />
          {experienceMode === 'race' ? <div className="run-status" data-testid="race-inspector" role="status" aria-live="polite">
            <div className="run-status__line"><span>Race</span><strong>{raceIsComplete ? 'analysis ready' : raceIsActive ? 'in progress' : 'configured'}</strong></div>
            <div className="metric-grid"><Metric label="Competitors" value={raceSolvers.length} /><Metric label="Display" value={raceDisplayMode === 'overview' ? 'Overview' : 'Side by side'} /><Metric label="Compute" value="Sequential" /><Metric label="Playback" value="Synchronized" /></div>
            <p className="solver-guarantee"><strong>Fairness contract</strong>Each solver receives an equivalent clone of the same immutable maze. Runtime is measured one solver at a time.</p>
          </div> : <div
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
              <Metric label="Peak Frontier" value={stats?.peakFrontier ?? '—'} />
              <Metric label="Solver" value={solver.replace('_', ' ')} />
            </div>
            <p className="solver-guarantee"><strong>Guarantee</strong>{SOLVER_GUARANTEES[solver] ?? 'Solver-specific result'}</p>
          </div>}
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
        <AlgorithmGuide />
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
