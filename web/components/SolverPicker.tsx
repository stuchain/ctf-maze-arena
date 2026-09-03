'use client';

import { Select } from '@/components/ui/Primitives';

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
  describedBy?: string;
}

export function SolverPicker({ value, onChange, id = 'solver-picker', describedBy }: SolverPickerProps) {
  return (
    <Select
      id={id}
      name="maze-solver"
      autoComplete="off"
      aria-describedby={describedBy}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      data-testid="solver-picker"
    >
      {SOLVERS.map((s) => (
        <option key={s.value} value={s.value}>
          {s.label}
        </option>
      ))}
    </Select>
  );
}

