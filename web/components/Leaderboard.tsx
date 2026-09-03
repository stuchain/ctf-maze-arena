'use client';

import type { LeaderboardEntry } from '@/lib/api';

export type { LeaderboardEntry } from '@/lib/api';

interface LeaderboardProps {
  entries: LeaderboardEntry[];
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

export function Leaderboard({ entries }: LeaderboardProps) {
  if (!entries.length) {
    return (
      <div className="compact-empty">
        <span className="compact-empty__rank" aria-hidden="true">01</span>
        <div>
          <strong>No Ranked Runs Yet</strong>
          <p>Complete a maze and sign in to claim the first position.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="table-scroll">
    <table className="data-table">
      <caption className="visually-hidden">Ranked solve results for this maze</caption>
      <thead>
        <tr>
          <th scope="col">Rank</th>
          <th scope="col">Solver</th>
          <th scope="col">Cost</th>
          <th scope="col">Time</th>
          <th scope="col">Visited</th>
        </tr>
      </thead>
      <tbody>
        {entries.map((e, i) => (
          <tr key={e.runId}>
            <td><span className="rank-number">{String(i + 1).padStart(2, '0')}</span></td>
            <td><span className="solver-token" translate="no">{e.solver}</span></td>
            <td className="numeric">{formatCost(e.cost)}</td>
            <td className="numeric">{formatTime(e.ms)}</td>
            <td className="numeric">{formatVisited(e.visited)}</td>
          </tr>
        ))}
      </tbody>
    </table>
    </div>
  );
}
