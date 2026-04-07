'use client';

const SOLVERS = [
  { value: 'BFS', label: 'BFS' },
  { value: 'DFS', label: 'DFS' },
  { value: 'ASTAR', label: 'A*' },
  { value: 'DP_KEYS', label: 'DP (Keys)' },
] as const;

export interface SolverPickerProps {
  value: string;
  onChange: (solver: string) => void;
  id?: string;
}

export function SolverPicker({ value, onChange, id = 'solver-picker' }: SolverPickerProps) {
  return (
    <select
      id={id}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="border rounded px-3 py-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-600 focus-visible:ring-offset-1"
      data-testid="solver-picker"
    >
      {SOLVERS.map((s) => (
        <option key={s.value} value={s.value}>
          {s.label}
        </option>
      ))}
    </select>
  );
}

