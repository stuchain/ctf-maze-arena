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
}

export function SolverPicker({ value, onChange }: SolverPickerProps) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="border rounded px-3 py-2"
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

