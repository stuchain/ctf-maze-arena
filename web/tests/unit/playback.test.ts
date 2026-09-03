import { describe, expect, it } from 'vitest';
import { initialPlaybackState, playbackReducer } from '@/hooks/usePlaybackTimeline';

describe('playback reducer', () => {
  it('supports replay, pause, stepping, speed, and reset', () => {
    let state = initialPlaybackState('replay');
    state = playbackReducer(state, { type: 'play', lastIndex: 4 });
    expect(state.playing).toBe(true);
    state = playbackReducer(state, { type: 'tick', lastIndex: 4 });
    expect(state.cursor).toBe(1);
    state = playbackReducer(state, { type: 'speed', speed: 2 });
    expect(state.speed).toBe(2);
    state = playbackReducer(state, { type: 'step', cursor: 4 });
    expect(state).toMatchObject({ cursor: 4, playing: false, followLive: false });
    expect(playbackReducer(state, { type: 'reset', mode: 'replay' })).toEqual(initialPlaybackState('replay'));
  });

  it('can leave and return to the live edge', () => {
    let state = initialPlaybackState('live');
    expect(state.followLive).toBe(true);
    state = playbackReducer(state, { type: 'pause', cursor: 3 });
    expect(state).toMatchObject({ cursor: 3, followLive: false });
    expect(playbackReducer(state, { type: 'live' }).followLive).toBe(true);
  });
});
