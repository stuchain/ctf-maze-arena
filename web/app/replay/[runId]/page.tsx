'use client';

import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useEffect, useMemo, useState } from 'react';
import { AppHeader } from '@/components/AppHeader';
import { MazeGrid } from '@/components/MazeGrid';
import { Badge, Button, Notice, Panel, Skeleton } from '@/components/ui/Primitives';
import { publicEnv } from '@/lib/env';
import { backendMazeToMazeData } from '@/lib/maze';
import { replayStates } from '@/lib/realtime';

interface ReplayPayload {
  mazeId: string;
  protocolVersion: number;
  solver: string;
  seed: number;
  events: unknown[];
  path: [number, number][];
  stats: { visited: number; cost: number; ms: number };
}

export default function ReplayPage() {
  const params = useParams();
  return <ReplayView key={String(params.runId)} runId={String(params.runId)} />;
}

function ReplayView({ runId }: { runId: string }) {
  const [replay, setReplay] = useState<ReplayPayload | null>(null);
  const [mazeJson, setMazeJson] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);
  const [frameIndex, setFrameIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [copied, setCopied] = useState(false);
  const frames = useMemo(() => replayStates(replay?.events ?? []), [replay]);

  useEffect(() => {
    const controller = new AbortController();
    fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/replay/${encodeURIComponent(runId)}`, { signal: controller.signal })
      .then((response) => response.ok ? response.json() : Promise.reject(new Error('This replay does not exist or has expired.')))
      .then((data: ReplayPayload) => {
        setReplay(data);
        return fetch(`${publicEnv.NEXT_PUBLIC_API_URL}/api/maze/${encodeURIComponent(data.mazeId)}`, { signal: controller.signal })
          .then((response) => response.ok ? response.json() : Promise.reject(new Error('The maze for this replay is unavailable.')));
      })
      .then(setMazeJson)
      .catch((cause: Error) => { if (cause.name !== 'AbortError') setError(cause.message); });
    return () => controller.abort();
  }, [runId]);

  useEffect(() => {
    if (!playing || !frames.length || frameIndex >= frames.length - 1) return;
    const timer = setTimeout(() => setFrameIndex((index) => index + 1), 100);
    return () => clearTimeout(timer);
  }, [playing, frameIndex, frames.length]);

  useEffect(() => {
    if (!copied) return;
    const timer = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timer);
  }, [copied]);

  const handleShare = async () => {
    await navigator.clipboard.writeText(`${window.location.origin}/replay/${runId}`);
    setCopied(true);
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
          replay={replay} mazeJson={mazeJson} frames={frames} frameIndex={frameIndex} playing={playing} copied={copied}
          onFrameChange={(index) => { setFrameIndex(index); setPlaying(false); }}
          onPlay={() => setPlaying(true)} onPause={() => setPlaying(false)}
          onReset={() => { setFrameIndex(0); setPlaying(false); }} onShare={() => void handleShare()}
        />
      )}
    </div>
  );
}

function ReplayReady({ replay, mazeJson, frames, frameIndex, playing, copied, onFrameChange, onPlay, onPause, onReset, onShare }: {
  replay: ReplayPayload; mazeJson: unknown; frames: ReturnType<typeof replayStates>; frameIndex: number; playing: boolean; copied: boolean;
  onFrameChange: (index: number) => void; onPlay: () => void; onPause: () => void; onReset: () => void; onShare: () => void;
}) {
  const maze = backendMazeToMazeData(mazeJson);
  const frame = frames[frameIndex];
  const lastFrame = !frames.length || frameIndex >= frames.length - 1;
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
          <span className="copy-status" role="status" aria-live="polite">{copied ? 'Link copied' : ''}</span>
        </div>
      </Panel>
      <Panel className="arena-panel">
        <div className="arena-heading">
          <div><p className="eyebrow">Seed {formatter.format(replay.seed)}</p><h2>Recorded search</h2></div>
          <Badge tone={lastFrame ? 'success' : playing ? 'info' : 'neutral'} pulse={playing && !lastFrame}>{lastFrame ? 'Complete' : playing ? 'Playing' : 'Paused'}</Badge>
        </div>
        <div className="stage-shell"><div className="stage-grid" aria-hidden="true" />
          <MazeGrid maze={maze} frontier={frame?.frontier} visited={frame?.visited} current={frame?.current} path={lastFrame ? replay.path : undefined} />
        </div>
        <div className="replay-controls" aria-label="Replay controls">
          <Button size="sm" onClick={onPlay} disabled={playing || lastFrame}>Play</Button>
          <Button size="sm" variant="secondary" onClick={onPause} disabled={!playing}>Pause</Button>
          <Button size="sm" variant="ghost" onClick={onReset} disabled={frameIndex === 0 && !playing}>Reset</Button>
          <label className="visually-hidden" htmlFor="replay-frame">Replay frame</label>
          <input id="replay-frame" className="replay-progress" type="range" min={0} max={Math.max(0, frames.length - 1)} value={Math.min(frameIndex, Math.max(0, frames.length - 1))} onChange={(event) => onFrameChange(Number(event.target.value))} />
          <span className="replay-counter" aria-live="polite">Frame {frames.length ? frameIndex + 1 : 0} / {frames.length}</span>
        </div>
      </Panel>
    </main>
  );
}
