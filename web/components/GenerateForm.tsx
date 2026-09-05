'use client';

import type { FormEvent } from 'react';
import { useState } from 'react';
import { Button, Field, Select } from '@/components/ui/Primitives';

export interface GenerateFormParams {
  w: number;
  h: number;
  seed: number;
  algo: string;
  featurePreset: 'classic' | 'keys';
}

export interface GenerateFormProps {
  onSubmit: (params: GenerateFormParams) => void;
  loading?: boolean;
  initialParams?: GenerateFormParams;
}

export function GenerateForm({ onSubmit, loading, initialParams }: GenerateFormProps) {
  const [w, setW] = useState(initialParams?.w ?? 10);
  const [h, setH] = useState(initialParams?.h ?? 10);
  const [seed, setSeed] = useState(() => initialParams?.seed ?? Math.floor(Math.random() * 1e6));
  const [algo, setAlgo] = useState(initialParams?.algo ?? 'KRUSKAL');
  const [featurePreset, setFeaturePreset] = useState<'classic' | 'keys'>(initialParams?.featurePreset ?? 'classic');

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit({ w, h, seed, algo, featurePreset });
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="configuration-form"
      data-testid="generate-form"
    >
      <div className="field-row">
        <Field label="Width" htmlFor="maze-width-input" hint="5–50 cells">
        <input
          id="maze-width-input"
          name="maze-width"
          type="number"
          inputMode="numeric"
          autoComplete="off"
          aria-describedby="maze-width-input-description"
          min={5}
          max={50}
          value={w}
          onChange={(e) => setW(Number(e.target.value))}
          className="control"
        />
        </Field>
        <Field label="Height" htmlFor="maze-height-input" hint="5–50 cells">
        <input
          id="maze-height-input"
          name="maze-height"
          type="number"
          inputMode="numeric"
          autoComplete="off"
          aria-describedby="maze-height-input-description"
          min={5}
          max={50}
          value={h}
          onChange={(e) => setH(Number(e.target.value))}
          className="control"
        />
        </Field>
      </div>

      <Field label="Seed" htmlFor="maze-seed-input" hint="Share this seed to recreate the maze.">
        <input
          id="maze-seed-input"
          name="maze-seed"
          type="number"
          inputMode="numeric"
          autoComplete="off"
          aria-describedby="maze-seed-input-description"
          min={0}
          value={seed}
          onChange={(e) => setSeed(Number(e.target.value))}
          className="control"
        />
      </Field>

      <Field label="Generator" htmlFor="maze-algo-select">
        <Select
          id="maze-algo-select"
          name="maze-generator"
          autoComplete="off"
          value={algo}
          onChange={(e) => setAlgo(e.target.value)}
        >
          <option value="KRUSKAL">Kruskal</option>
          <option value="PRIM">Prim</option>
          <option value="DFS">Depth-First Search</option>
        </Select>
      </Field>

      <Field label="Maze Features" htmlFor="maze-feature-select" hint="Keys adds a deterministic key and locked passage.">
        <Select id="maze-feature-select" value={featurePreset} onChange={(event) => setFeaturePreset(event.target.value as 'classic' | 'keys')}>
          <option value="classic">Classic passages</option>
          <option value="keys">Key &amp; locked passage</option>
        </Select>
      </Field>

      <Button
        type="submit"
        loading={loading}
        className="button--full"
        data-testid="generate-button"
      >
        {loading ? 'Generating…' : 'Generate Maze'}
      </Button>
    </form>
  );
}

