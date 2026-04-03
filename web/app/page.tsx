/* eslint-disable @next/next/no-img-element */
'use client';

import { useState } from 'react';
import { MazeGrid } from '../components/MazeGrid';
import { GenerateForm, type GenerateFormParams } from '../components/GenerateForm';
import { SolverPicker } from '../components/SolverPicker';

export default function Home() {
  const [solver, setSolver] = useState('ASTAR');

  const handleGenerate = (params: GenerateFormParams) => {
    // Temporary: Commit 58 will replace this with a real API call.
    // eslint-disable-next-line no-console
    console.log('generate params', params);
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <div className="flex flex-col items-center gap-6 p-8">
        <SolverPicker value={solver} onChange={setSolver} />
        <GenerateForm onSubmit={handleGenerate} />
        <MazeGrid maze={null} />
      </div>
    </div>
  );
}
