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
            <td className="border p-2">{e.cost}</td>
            <td className="border p-2">{e.ms} ms</td>
            <td className="border p-2">{e.visited}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
