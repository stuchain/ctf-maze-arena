/* eslint-disable @next/next/no-img-element */
'use client';

import { useState } from 'react';
import { MazeGrid } from '../components/MazeGrid';
import { SolverPicker } from '../components/SolverPicker';

export default function Home() {
  const [solver, setSolver] = useState('ASTAR');

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <div className="flex flex-col items-center gap-6 p-8">
        <SolverPicker value={solver} onChange={setSolver} />
        <MazeGrid maze={null} />
      </div>
    </div>
  );
}
