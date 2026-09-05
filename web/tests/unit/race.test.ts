import { describe, expect, it } from 'vitest';
import { canonicalRaceUrl, explainRace, parseRaceConfig, raceSearchParams, type RaceConfig } from '@/lib/race';

const config: RaceConfig = { version: 1, width: 20, height: 15, seed: 9001, generator: 'PRIM', featurePreset: 'keys', solvers: ['BFS', 'ASTAR', 'DP_KEYS'], displayMode: 'side-by-side' };

describe('race configuration', () => {
  it('roundtrips through a canonical versioned URL', () => {
    const params = raceSearchParams(config);
    expect(parseRaceConfig(params)).toEqual(config);
    expect(canonicalRaceUrl(config, { origin: 'https://arena.example', pathname: '/' } as Location))
      .toBe('https://arena.example/?race=1&v=1&seed=9001&w=20&h=15&generator=PRIM&features=keys&solvers=BFS%2CASTAR%2CDP_KEYS&mode=side-by-side');
  });

  it('rejects unsupported versions and repairs invalid solver selections', () => {
    expect(parseRaceConfig(new URLSearchParams('race=1&v=2'))).toBeNull();
    expect(parseRaceConfig(new URLSearchParams('race=1&v=1&solvers=DP_KEYS,BOGUS'))?.solvers)
      .toEqual(['BFS', 'DFS', 'ASTAR']);
  });
});

describe('race explanations', () => {
  it('derives factual notes from authoritative metrics', () => {
    const notes = explainRace([
      { solver: 'BFS', stats: { visited: 40, cost: 12, ms: 2, peakFrontier: 8 } },
      { solver: 'ASTAR', stats: { visited: 25, cost: 12, ms: 1, peakFrontier: 5 } },
      { solver: 'DFS', stats: { visited: 30, cost: 18, ms: 1, peakFrontier: 9 } },
    ], 'classic');
    expect(notes.join(' ')).toContain('15 fewer cells');
    expect(notes.join(' ')).toContain('6 steps longer');
    expect(notes.at(-1)).toContain('not benchmarks');
  });
});
