'use client';

import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useEffect, useMemo, useState } from 'react';
import { AppHeader } from '@/components/AppHeader';
import { MazeGrid } from '@/components/MazeGrid';
import { PlaybackControls } from '@/components/PlaybackControls';
import { Badge, Button, Notice, Panel, Skeleton } from '@/components/ui/Primitives';
import { publicEnv } from '@/lib/env';
import { backendMazeToMazeData } from '@/lib/maze';
import { replayStates } from '@/lib/realtime';
import { usePlaybackTimeline } from '@/hooks/usePlaybackTimeline';
import { parseReplayView, type ReplayViewModel } from '@/lib/replay-view';

export default function ReplayPage() {
  const params = useParams();
  return <ReplayView key={String(params.runId)} runId={String(params.runId)} />;
}

function ReplayView({ runId }: { runId: string }) {
  const [replay, setReplay] = useState<ReplayViewModel | null>(null);
  const [mazeJson, setMazeJson] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [shareStatus, setShareStatus] = useState('');
  const frames = useMemo(() => replayStates(replay?.events ?? []), [replay]);

  useEffect(() => {
    const controller = new AbortController();
    fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/replay/${encodeURIComponent(runId)}`, { signal: controller.signal })
      .then((response) => response.ok ? response.json() : Promise.reject(new Error('This replay does not exist or has expired.')))
      .then((payload: unknown) => {
        const data = parseReplayView(payload);
        setReplay(data);
        return fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/maze/${encodeURIComponent(data.mazeId)}`, { signal: controller.signal })
          .then((response) => response.ok ? response.json() : Promise.reject(new Error('The maze for this replay is unavailable.')));
      })
      .then(setMazeJson)
      .catch((cause: Error) => { if (cause.name !== 'AbortError') setError(cause.message); });
    return () => controller.abort();
  }, [runId]);

  useEffect(() => {
    if (!shareStatus) return;
    const timer = setTimeout(() => setShareStatus(''), 2500);
    return () => clearTimeout(timer);
  }, [shareStatus]);

  const handleShare = async () => {
    const url = `${window.location.origin}/replay/${runId}`;
    try {
      if (navigator.share) {
        await navigator.share({ title: 'CTF Maze Arena Replay', text: 'Explore this deterministic solver replay.', url });
        setShareStatus('Replay shared');
      } else {
        await navigator.clipboard.writeText(url);
        setShareStatus('Link copied');
      }
    } catch (cause) {
      if ((cause as Error).name !== 'AbortError') setShareStatus('Could not share the link');
    }
  };

  return (
    <div className="replay-shell">
      <AppHeader />
      {error ? (
        <main id="main-content" className="state-page">
          <Panel className="state-card">
            <Badge tone="danger">Replay unavailable</Badge>
            <h1>That trail has gone cold.</h1>
            <Notice title="Could not load replay" tone="danger">{error}</Notice>
            <Link href="/" className="button button--primary button--md">Return to the arena</Link>
          </Panel>
        </main>
      ) : !replay || !mazeJson ? (
        <main id="main-content" className="loading-layout" aria-busy="true" aria-label="Loading replay">
          <Panel className="loading-card">
            <Skeleton className="skeleton--label" /><Skeleton className="skeleton--title" /><Skeleton className="skeleton--stage" />
            <span className="visually-hidden">Loading replay…</span>
          </Panel>
        </main>
      ) : (
        <ReplayReady
          replay={replay} mazeJson={mazeJson} frames={frames} shareStatus={shareStatus}
          onShare={() => void handleShare()}
        />
      )}
    </div>
  );
}

function ReplayReady({ replay, mazeJson, frames, shareStatus, onShare }: {
  replay: ReplayViewModel; mazeJson: unknown; frames: ReturnType<typeof replayStates>; shareStatus: string; onShare: () => void;
}) {
  const maze = useMemo(() => backendMazeToMazeData(mazeJson), [mazeJson]);
  const playback = usePlaybackTimeline(frames.length, 'replay', replay.mazeId);
  const frame = frames[playback.displayIndex];
  const formatter = new Intl.NumberFormat();

  return (
    <main id="main-content" className="replay-main app-frame">
      <Panel className="replay-summary">
        <div className="replay-summary__copy">
          <p className="eyebrow">Deterministic replay · Protocol v{replay.protocolVersion}</p>
          <h1>Solve replay</h1>
          <p className="replay-meta">
            <span><b>Solver</b> {replay.solver.toUpperCase()}</span><span><b>Visited</b> {formatter.format(replay.stats.visited)}</span>
            <span><b>Cost</b> {formatter.format(replay.stats.cost)}</span><span><b>Time</b> {formatter.format(replay.stats.ms)} ms</span>
          </p>
        </div>
        <div className="replay-actions">
          <Link href="/" className="button button--ghost button--sm">Arena</Link>
          <Button variant="secondary" size="sm" onClick={onShare}>Share replay</Button>
          <span className="copy-status" role="status" aria-live="polite">{shareStatus}</span>
        </div>
      </Panel>
      <Panel className="arena-panel">
        <div className="arena-heading">
          <div><p className="eyebrow">Seed {formatter.format(replay.seed)}</p><h2>Recorded search</h2></div>
          <Badge tone={playback.atEnd ? 'success' : playback.playing ? 'info' : 'neutral'} pulse={playback.playing && !playback.atEnd}>{playback.atEnd ? 'Complete' : playback.playing ? 'Playing' : 'Paused'}</Badge>
        </div>
        <div className="stage-shell"><div className="stage-grid" aria-hidden="true" />
          <MazeGrid key={replay.mazeId} maze={maze} frontier={frame?.frontier} visited={frame?.visited} current={frame?.current} path={playback.atEnd ? replay.path : undefined} />
        </div>
        <PlaybackControls
          mode="replay" totalFrames={frames.length} currentIndex={playback.displayIndex}
          playing={playback.playing} followLive={playback.followLive} speed={playback.speed}
          onPlay={playback.play} onPause={playback.pause} onReset={playback.reset}
          onPrevious={playback.previous} onNext={playback.next} onGoLive={playback.goLive}
          onIndexChange={playback.setIndex} onSpeedChange={playback.setSpeed}
        />
      </Panel>
    </main>
  );
}
