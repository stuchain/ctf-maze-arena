'use client';

import { Button, Select } from '@/components/ui/Primitives';
import { PLAYBACK_SPEEDS, type PlaybackMode, type PlaybackSpeed } from '@/hooks/usePlaybackTimeline';

interface PlaybackControlsProps {
  mode: PlaybackMode;
  totalFrames: number;
  currentIndex: number;
  playing: boolean;
  followLive: boolean;
  speed: PlaybackSpeed;
  onPlay: () => void;
  onPause: () => void;
  onReset: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onGoLive: () => void;
  onIndexChange: (index: number) => void;
  onSpeedChange: (speed: PlaybackSpeed) => void;
}

export function PlaybackControls(props: PlaybackControlsProps) {
  const lastIndex = Math.max(0, props.totalFrames - 1);
  const atStart = props.currentIndex === 0;
  const atEnd = props.currentIndex === lastIndex;
  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.target !== event.currentTarget) return;
    if (event.key === ' ') {
      event.preventDefault();
      if (props.playing) props.onPause();
      else props.onPlay();
    }
    else if (event.key === 'ArrowLeft') { event.preventDefault(); props.onPrevious(); }
    else if (event.key === 'ArrowRight') { event.preventDefault(); props.onNext(); }
    else if (event.key === 'Home') { event.preventDefault(); props.onReset(); }
    else if (event.key.toLowerCase() === 'l' && props.mode === 'live') { event.preventDefault(); props.onGoLive(); }
  };

  return (
    <div className="playback" role="group" tabIndex={0} aria-label={`${props.mode === 'live' ? 'Live run' : 'Replay'} playback controls`} onKeyDown={handleKeyDown}>
      <div className="playback__buttons">
        <Button size="sm" variant="ghost" onClick={props.onReset} disabled={atStart && !props.playing}>Reset</Button>
        <Button size="sm" variant="secondary" onClick={props.onPrevious} disabled={atStart}>Previous</Button>
        {props.playing ? <Button size="sm" onClick={props.onPause}>Pause</Button> : <Button size="sm" onClick={props.onPlay} disabled={props.totalFrames < 2}>Play</Button>}
        <Button size="sm" variant="secondary" onClick={props.onNext} disabled={atEnd}>Next</Button>
        {props.mode === 'live' && !props.followLive ? <Button size="sm" variant="secondary" onClick={props.onGoLive}>Return to Live</Button> : null}
      </div>
      <label className="visually-hidden" htmlFor={`${props.mode}-timeline`}>{props.mode === 'live' ? 'Live run' : 'Replay'} frame</label>
      <input
        id={`${props.mode}-timeline`}
        className="replay-progress"
        type="range"
        min={0}
        max={lastIndex}
        value={props.currentIndex}
        onChange={(event) => props.onIndexChange(Number(event.target.value))}
      />
      <output className="replay-counter" aria-live="polite">Step {props.totalFrames ? props.currentIndex + 1 : 0} / {props.totalFrames}</output>
      <label className="playback__speed">
        <span>Speed</span>
        <Select value={String(props.speed)} onChange={(event) => props.onSpeedChange(Number(event.target.value) as PlaybackSpeed)} aria-label="Playback speed">
          {PLAYBACK_SPEEDS.map((speed) => <option key={speed} value={speed}>{speed}×</option>)}
        </Select>
      </label>
      <details className="playback__help">
        <summary>Shortcuts</summary>
        <p><kbd>Space</kbd> play/pause · <kbd>← →</kbd> step · <kbd>Home</kbd> reset{props.mode === 'live' ? <> · <kbd>L</kbd> live</> : null}</p>
      </details>
    </div>
  );
}
