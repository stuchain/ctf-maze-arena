'use client';

export interface LeaderboardEntry {
  runId: string;
  solver: string;
  cost: number;
  ms: number;
  visited: number;
}

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
  return (
    <table className="border-collapse border w-full max-w-md">
      <thead>
        <tr className="bg-gray-100">
          <th className="border p-2">Rank</th>
          <th className="border p-2">Solver</th>
          <th className="border p-2">Cost</th>
          <th className="border p-2">Time</th>
          <th className="border p-2">Visited</th>
        </tr>
      </thead>
      <tbody>
        {entries.map((e, i) => (
          <tr key={e.runId}>
            <td className="border p-2">{i + 1}</td>
            <td className="border p-2">{e.solver}</td>
            <td className="border p-2">{formatCost(e.cost)}</td>
            <td className="border p-2">{formatTime(e.ms)}</td>
            <td className="border p-2">{formatVisited(e.visited)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
