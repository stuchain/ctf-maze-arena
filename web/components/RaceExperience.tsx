'use client';

import { useState } from 'react';
import { MazeGrid, type MazeData } from '@/components/MazeGrid';
import { PlaybackControls } from '@/components/PlaybackControls';
import { Badge, Button, Notice, Panel, PanelHeader } from '@/components/ui/Primitives';
import { usePlaybackTimeline } from '@/hooks/usePlaybackTimeline';
import type { UseSolveStreamResult } from '@/hooks/useSolveStream';
import { explainRace, type RaceConfig, type RaceSolver } from '@/lib/race';

const LABELS: Record<RaceSolver, string> = { BFS: 'BFS', DFS: 'DFS', ASTAR: 'A*', DP_KEYS: 'DP Keys' };
const GUARANTEES: Record<RaceSolver, string> = {
  BFS: 'Complete · optimal on unit-cost mazes', DFS: 'Complete · not optimal',
  ASTAR: 'Complete · optimal with Manhattan heuristic', DP_KEYS: 'Complete · optimal in key-state space',
};

interface Props {
  maze: MazeData;
  config: RaceConfig;
  raceId: string;
  runIds: Partial<Record<RaceSolver, string>>;
  streams: Record<RaceSolver, UseSolveStreamResult>;
  onCancel: () => void;
}

export function RaceExperience({ maze, config, raceId, runIds, streams, onCancel }: Props) {
  const [focusedSolver, setFocusedSolver] = useState<RaceSolver>(config.solvers[0]);
  const selected = config.solvers.map((solver) => ({ solver, stream: streams[solver] }));
  const totalFrames = Math.max(0, ...selected.map(({ stream }) => stream.frames.length));
  const playback = usePlaybackTimeline(totalFrames, 'replay', raceId);
  const allTerminal = selected.every(({ stream }) => ['completed', 'failed', 'cancelled'].includes(stream.status));
  const results = selected.flatMap(({ solver, stream }) => stream.stats ? [{ solver, stats: stream.stats }] : []);
  const notes = explainRace(results, config.featurePreset);
  const logicalOrder = [...selected]
    .filter(({ stream }) => stream.status === 'completed')
    .sort((left, right) => left.stream.frames.length - right.stream.frames.length)
    .map(({ solver }) => solver);

  const frameFor = (solver: RaceSolver) => {
    const frames = streams[solver].frames;
    if (!frames.length) return undefined;
    return frames[Math.min(frames.length - 1, playback.displayIndex)];
  };
  const pathFor = (solver: RaceSolver) => playback.displayIndex >= streams[solver].frames.length - 1 ? streams[solver].path : undefined;

  return (
    <section className="race-experience" data-testid="race-experience" aria-labelledby="race-title">
      <div className="race-heading">
        <div><p className="eyebrow">Signature Experience</p><h2 id="race-title">Algorithm Race</h2><p>One immutable maze. Sequential compute. Synchronized logical playback.</p></div>
        <Badge tone={allTerminal ? 'success' : 'info'} pulse={!allTerminal}>{allTerminal ? 'Analysis Ready' : 'Race Running'}</Badge>
      </div>
      <div className="race-lanes" aria-label="Competitor status lanes">
        {selected.map(({ solver, stream }, index) => {
          const frame = stream.frames.at(-1);
          return <button type="button" className={focusedSolver === solver ? 'race-lane race-lane--active' : 'race-lane'} key={solver} onClick={() => setFocusedSolver(solver)}>
            <span>{logicalOrder.includes(solver) ? `#${logicalOrder.indexOf(solver) + 1} finish · ` : `${index + 1} · `}{LABELS[solver]}</span><strong>{stream.status}</strong><small>{frame?.visited.length ?? stream.stats?.visited ?? 0} visited · {frame?.frontier.length ?? 0} frontier</small>
          </button>;
        })}
      </div>
      {config.displayMode === 'overview' ? (
        <div className="race-overview-stage"><MazeGrid maze={maze} frontier={frameFor(focusedSolver)?.frontier} visited={frameFor(focusedSolver)?.visited} current={frameFor(focusedSolver)?.current} path={pathFor(focusedSolver)} /></div>
      ) : (
        <div className="race-stage-grid">
          {selected.map(({ solver }) => <article key={solver} className={focusedSolver === solver ? 'race-stage race-stage--active' : 'race-stage'}>
            <h3>{LABELS[solver]}</h3><MazeGrid maze={maze} frontier={frameFor(solver)?.frontier} visited={frameFor(solver)?.visited} current={frameFor(solver)?.current} path={pathFor(solver)} />
          </article>)}
        </div>
      )}
      {totalFrames > 0 ? <PlaybackControls mode="replay" totalFrames={totalFrames} currentIndex={playback.displayIndex} playing={playback.playing} followLive={false} speed={playback.speed} onPlay={playback.play} onPause={playback.pause} onReset={playback.reset} onPrevious={playback.previous} onNext={playback.next} onGoLive={() => undefined} onIndexChange={playback.setIndex} onSpeedChange={playback.setSpeed} /> : null}
      {!allTerminal ? <Button variant="destructive" onClick={onCancel}>Cancel Entire Race</Button> : null}
      {allTerminal ? <Panel className="race-results">
        <PanelHeader eyebrow="Results Analysis" title="Structural Tradeoffs" description="Compute runtime is recorded independently from user-controlled playback." />
        <div className="race-table-wrap"><table className="race-table"><thead><tr><th>Solver</th><th>Logical finish</th><th>Path cost</th><th>Visited</th><th>Peak frontier</th><th>Compute</th><th>Guarantee</th><th>Replay</th></tr></thead>
          <tbody>{selected.map(({ solver, stream }) => <tr key={solver}><th>{LABELS[solver]}</th><td>{logicalOrder.includes(solver) ? `#${logicalOrder.indexOf(solver) + 1}` : '—'}</td><td>{stream.stats?.cost ?? '—'}</td><td>{stream.stats?.visited ?? '—'}</td><td>{stream.stats?.peakFrontier ?? '—'}</td><td>{stream.stats ? `${stream.stats.ms} ms` : '—'}</td><td>{GUARANTEES[solver]}</td><td>{runIds[solver] ? <a href={`/replay/${runIds[solver]}`}>Open</a> : '—'}</td></tr>)}</tbody>
        </table></div>
        <div className="race-explanations">{notes.map((note) => <Notice key={note} title="What the metrics show" tone="info">{note}</Notice>)}</div>
      </Panel> : null}
    </section>
  );
}
