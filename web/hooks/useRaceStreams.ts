'use client';

import { useSolveStream } from '@/hooks/useSolveStream';
import type { RaceSolver } from '@/lib/race';

export type RaceRunMap = Partial<Record<RaceSolver, string>>;

export function useRaceStreams(runs: RaceRunMap) {
  const bfs = useSolveStream(runs.BFS ?? null, 'BFS');
  const dfs = useSolveStream(runs.DFS ?? null, 'DFS');
  const astar = useSolveStream(runs.ASTAR ?? null, 'ASTAR');
  const dpKeys = useSolveStream(runs.DP_KEYS ?? null, 'DP_KEYS');
  return { BFS: bfs, DFS: dfs, ASTAR: astar, DP_KEYS: dpKeys };
}
