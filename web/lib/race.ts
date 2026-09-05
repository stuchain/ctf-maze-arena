import type { SolveStats } from '@/lib/realtime';

export const RACE_VERSION = 1;
export const RACE_SOLVERS = ['BFS', 'DFS', 'ASTAR', 'DP_KEYS'] as const;
export type RaceSolver = typeof RACE_SOLVERS[number];
export type RaceDisplayMode = 'overview' | 'side-by-side';
export type RaceFeaturePreset = 'classic' | 'keys';

export interface RaceConfig {
  version: 1;
  width: number;
  height: number;
  seed: number;
  generator: 'KRUSKAL' | 'PRIM' | 'DFS';
  featurePreset: RaceFeaturePreset;
  solvers: RaceSolver[];
  displayMode: RaceDisplayMode;
}

export const DEFAULT_RACE_CONFIG: RaceConfig = {
  version: RACE_VERSION,
  width: 10,
  height: 10,
  seed: 42,
  generator: 'KRUSKAL',
  featurePreset: 'classic',
  solvers: ['BFS', 'DFS', 'ASTAR'],
  displayMode: 'overview',
};

const solverOrder = new Map(RACE_SOLVERS.map((solver, index) => [solver, index]));

function boundedInteger(value: string | null, fallback: number, min: number, max: number) {
  if (value === null || !/^\d+$/.test(value)) return fallback;
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed >= min && parsed <= max ? parsed : fallback;
}

export function normalizeRaceSolvers(values: readonly string[], featurePreset: RaceFeaturePreset): RaceSolver[] {
  const allowed = new Set<RaceSolver>(RACE_SOLVERS);
  const unique = [...new Set(values.filter((value): value is RaceSolver => allowed.has(value as RaceSolver)))];
  const compatible = featurePreset === 'keys' ? unique : unique.filter((solver) => solver !== 'DP_KEYS');
  const fallback: RaceSolver[] = ['BFS', 'DFS', 'ASTAR'];
  return (compatible.length >= 2 ? compatible : fallback).sort(
    (left, right) => (solverOrder.get(left) ?? 0) - (solverOrder.get(right) ?? 0),
  ).slice(0, 4);
}

export function parseRaceConfig(params: URLSearchParams): RaceConfig | null {
  if (params.get('race') !== '1' || params.get('v') !== String(RACE_VERSION)) return null;
  const featurePreset: RaceFeaturePreset = params.get('features') === 'keys' ? 'keys' : 'classic';
  const generator = ['KRUSKAL', 'PRIM', 'DFS'].includes(params.get('generator') ?? '')
    ? params.get('generator') as RaceConfig['generator']
    : DEFAULT_RACE_CONFIG.generator;
  return {
    version: RACE_VERSION,
    width: boundedInteger(params.get('w'), DEFAULT_RACE_CONFIG.width, 5, 50),
    height: boundedInteger(params.get('h'), DEFAULT_RACE_CONFIG.height, 5, 50),
    seed: boundedInteger(params.get('seed'), DEFAULT_RACE_CONFIG.seed, 0, Number.MAX_SAFE_INTEGER),
    generator,
    featurePreset,
    solvers: normalizeRaceSolvers((params.get('solvers') ?? '').split(','), featurePreset),
    displayMode: params.get('mode') === 'side-by-side' ? 'side-by-side' : 'overview',
  };
}

export function raceSearchParams(config: RaceConfig) {
  const params = new URLSearchParams({ race: '1', v: String(RACE_VERSION), seed: String(config.seed) });
  if (config.width !== DEFAULT_RACE_CONFIG.width) params.set('w', String(config.width));
  if (config.height !== DEFAULT_RACE_CONFIG.height) params.set('h', String(config.height));
  if (config.generator !== DEFAULT_RACE_CONFIG.generator) params.set('generator', config.generator);
  if (config.featurePreset !== DEFAULT_RACE_CONFIG.featurePreset) params.set('features', config.featurePreset);
  const solvers = normalizeRaceSolvers(config.solvers, config.featurePreset);
  if (solvers.join(',') !== DEFAULT_RACE_CONFIG.solvers.join(',')) params.set('solvers', solvers.join(','));
  if (config.displayMode !== DEFAULT_RACE_CONFIG.displayMode) params.set('mode', config.displayMode);
  return params;
}

export function canonicalRaceUrl(config: RaceConfig, location: Pick<Location, 'origin' | 'pathname'>) {
  return `${location.origin}${location.pathname}?${raceSearchParams(config)}`;
}

export interface RaceResult { solver: RaceSolver; stats: SolveStats }

export function explainRace(results: RaceResult[], featurePreset: RaceFeaturePreset): string[] {
  if (!results.length) return [];
  const notes: string[] = [];
  const bySolver = new Map(results.map((result) => [result.solver, result.stats]));
  const bfs = bySolver.get('BFS');
  const astar = bySolver.get('ASTAR');
  const dfs = bySolver.get('DFS');
  if (bfs && astar) {
    notes.push(bfs.cost === astar.cost
      ? `BFS and A* both found the optimal ${bfs.cost}-step path; A* visited ${Math.abs(bfs.visited - astar.visited)} ${astar.visited <= bfs.visited ? 'fewer' : 'more'} cells.`
      : 'BFS and A* returned different costs, so this result should be treated as a correctness failure.');
  }
  if (dfs && bfs) notes.push(dfs.cost > bfs.cost
    ? `DFS found a path ${dfs.cost - bfs.cost} steps longer than the optimum, illustrating its non-optimal depth-first order.`
    : 'DFS happened to find an optimal path on this seed, but that is not guaranteed.');
  if (featurePreset === 'keys') notes.push('DP Keys searches (cell, key-set) states; its visited count is state expansions and is not directly comparable with cell-only solvers.');
  notes.push('Compute runtime is measured sequentially on the API host; playback speed and logical finish order are presentation metrics, not benchmarks.');
  return notes;
}
