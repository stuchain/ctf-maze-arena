'use client';

import type { LeaderboardEntry } from '@/lib/api';

export type { LeaderboardEntry } from '@/lib/api';

interface LeaderboardProps {
  entries: LeaderboardEntry[];
  loading?: boolean;
  error?: string | null;
  solver: string;
  scope: 'all' | 'personal';
  page: number;
  pageSize: number;
  signedIn: boolean;
  onSolverChange: (solver: string) => void;
  onScopeChange: (scope: 'all' | 'personal') => void;
  onPageChange: (page: number) => void;
}

function formatCost(cost: number) {
  return Math.trunc(cost);
}

function formatTime(ms: number) {
  if (ms >= 1000) return `${(ms / 1000).toFixed(1)} s`;
  return `${ms} ms`;
}

function formatVisited(visited: number) {
  return Math.trunc(visited).toLocaleString();
}

export function Leaderboard({
  entries, loading = false, error, solver, scope, page, pageSize, signedIn,
  onSolverChange, onScopeChange, onPageChange,
}: LeaderboardProps) {
  const controls = <div className="leaderboard-controls">
    <label>Solver<select value={solver} onChange={(event) => onSolverChange(event.target.value)}>
      <option value="">All</option><option value="BFS">BFS</option><option value="DFS">DFS</option>
      <option value="ASTAR">A*</option><option value="DP_KEYS">DP Keys</option>
    </select></label>
    <label>View<select value={scope} disabled={!signedIn} onChange={(event) => onScopeChange(event.target.value as 'all' | 'personal')}>
      <option value="all">Everyone</option><option value="personal">My runs</option>
    </select></label>
  </div>;
  if (loading) return <>{controls}<p className="community-hint" role="status">Loading ranked runs…</p></>;
  if (error) return <>{controls}<p className="community-hint" role="alert">{error}</p></>;
  if (!entries.length) {
    return (
      <>{controls}<div className="compact-empty">
        <span className="compact-empty__rank" aria-hidden="true">01</span>
        <div>
          <strong>No Ranked Runs Yet</strong>
          <p>Complete a maze and sign in to claim the first position.</p>
        </div>
      </div>{page > 0 ? <button className="button button--ghost button--sm" onClick={() => onPageChange(page - 1)}>Previous page</button> : null}</>
    );
  }

  return (
    <>{controls}<div className="table-scroll">
    <table className="data-table">
      <caption className="visually-hidden">Ranked solve results for this maze</caption>
      <thead>
        <tr>
          <th scope="col">Rank</th>
          <th scope="col">Player</th>
          <th scope="col">Solver</th>
          <th scope="col">Cost</th>
          <th scope="col">Time</th>
          <th scope="col">Visited</th>
        </tr>
      </thead>
      <tbody>
        {entries.map((e, i) => (
          <tr key={e.runId}>
            <td><span className="rank-number">{e.tied ? 'T' : ''}{String(e.rank || page * pageSize + i + 1).padStart(2, '0')}</span></td>
            <td>{e.displayName ?? 'GitHub player'}</td>
            <td><span className="solver-token" translate="no">{e.solver}</span>{e.isPersonal ? <span className="personal-marker">You</span> : null}</td>
            <td className="numeric">{formatCost(e.cost)}</td>
            <td className="numeric">{formatTime(e.ms)}</td>
            <td className="numeric">{formatVisited(e.visited)}</td>
          </tr>
        ))}
      </tbody>
    </table>
    </div><nav className="pagination" aria-label="Leaderboard pages">
      <button disabled={page === 0} onClick={() => onPageChange(page - 1)}>Previous</button>
      <span>Page {page + 1}</span>
      <button disabled={entries.length < pageSize} onClick={() => onPageChange(page + 1)}>Next</button>
    </nav></>
  );
}
