'use client';

import type { FormEvent } from 'react';
import { useState } from 'react';

export interface GenerateFormParams {
  w: number;
  h: number;
  seed: number;
  algo: string;
}

export interface GenerateFormProps {
  onSubmit: (params: GenerateFormParams) => void;
  loading?: boolean;
}

export function GenerateForm({ onSubmit, loading }: GenerateFormProps) {
  const [w, setW] = useState(10);
  const [h, setH] = useState(10);
  const [seed, setSeed] = useState(() => Math.floor(Math.random() * 1e6));
  const [algo, setAlgo] = useState('KRUSKAL');

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit({ w, h, seed, algo });
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="flex flex-col gap-2 max-w-xs"
      data-testid="generate-form"
    >
      <label>
        Width:{' '}
        <input
          type="number"
          min={5}
          max={50}
          value={w}
          onChange={(e) => setW(Number(e.target.value))}
          className="border w-16"
        />
      </label>

      <label>
        Height:{' '}
        <input
          type="number"
          min={5}
          max={50}
          value={h}
          onChange={(e) => setH(Number(e.target.value))}
          className="border w-16"
        />
      </label>

      <label>
        Seed:{' '}
        <input
          type="number"
          min={0}
          value={seed}
          onChange={(e) => setSeed(Number(e.target.value))}
          className="border w-24"
        />
      </label>

      <label>
        Algorithm:{' '}
        <select
          value={algo}
          onChange={(e) => setAlgo(e.target.value)}
          className="border ml-2"
        >
          <option value="KRUSKAL">Kruskal</option>
          <option value="PRIM">Prim</option>
          <option value="DFS">DFS</option>
        </select>
      </label>

      <button
        type="submit"
        disabled={loading}
        className="bg-blue-500 text-white px-4 py-2 rounded disabled:opacity-50"
        data-testid="generate-button"
      >
        {loading ? 'Generating...' : 'Generate Maze'}
      </button>
    </form>
  );
}

