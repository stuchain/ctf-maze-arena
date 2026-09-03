'use client';

import { useEffect, useReducer } from 'react';

export type PlaybackMode = 'live' | 'replay';
export const PLAYBACK_SPEEDS = [0.5, 1, 2, 4] as const;
export type PlaybackSpeed = typeof PLAYBACK_SPEEDS[number];

export interface PlaybackState {
  cursor: number;
  playing: boolean;
  followLive: boolean;
  speed: PlaybackSpeed;
}

type Action =
  | { type: 'reset'; mode: PlaybackMode }
  | { type: 'play'; lastIndex: number }
  | { type: 'pause'; cursor: number }
  | { type: 'step'; cursor: number }
  | { type: 'tick'; lastIndex: number }
  | { type: 'live' }
  | { type: 'speed'; speed: PlaybackSpeed };

export function initialPlaybackState(mode: PlaybackMode): PlaybackState {
  return { cursor: 0, playing: false, followLive: mode === 'live', speed: 1 };
}

export function playbackReducer(state: PlaybackState, action: Action): PlaybackState {
  switch (action.type) {
    case 'reset': return initialPlaybackState(action.mode);
    case 'play': return { ...state, cursor: state.cursor >= action.lastIndex ? 0 : state.cursor, playing: action.lastIndex > 0, followLive: false };
    case 'pause': return { ...state, cursor: action.cursor, playing: false, followLive: false };
    case 'step': return { ...state, cursor: action.cursor, playing: false, followLive: false };
    case 'tick': return state.cursor >= action.lastIndex
      ? { ...state, cursor: action.lastIndex, playing: false }
      : { ...state, cursor: state.cursor + 1 };
    case 'live': return { ...state, playing: false, followLive: true };
    case 'speed': return { ...state, speed: action.speed };
  }
}

export function usePlaybackTimeline(totalFrames: number, mode: PlaybackMode, sourceId: string | null) {
  const [state, dispatch] = useReducer(playbackReducer, mode, initialPlaybackState);
  const lastIndex = Math.max(0, totalFrames - 1);
  const displayIndex = state.followLive ? lastIndex : Math.min(state.cursor, lastIndex);

  useEffect(() => {
    dispatch({ type: 'reset', mode });
  }, [sourceId, mode]);

  useEffect(() => {
    if (!state.playing || totalFrames < 2) return;
    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const delay = (reduceMotion ? 240 : 120) / state.speed;
    const timer = window.setTimeout(() => dispatch({ type: 'tick', lastIndex }), delay);
    return () => window.clearTimeout(timer);
  }, [lastIndex, state.playing, state.speed, totalFrames, state.cursor]);

  const setIndex = (index: number) => dispatch({ type: 'step', cursor: Math.max(0, Math.min(lastIndex, index)) });

  return {
    ...state,
    displayIndex,
    lastIndex,
    atEnd: displayIndex === lastIndex,
    play: () => dispatch({ type: 'play', lastIndex }),
    pause: () => dispatch({ type: 'pause', cursor: displayIndex }),
    reset: () => dispatch({ type: 'step', cursor: 0 }),
    previous: () => setIndex(displayIndex - 1),
    next: () => setIndex(displayIndex + 1),
    goLive: () => dispatch({ type: 'live' }),
    setIndex,
    setSpeed: (speed: PlaybackSpeed) => dispatch({ type: 'speed', speed }),
  };
}
