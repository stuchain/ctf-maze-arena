import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Solve Replay',
  description: 'Inspect every logical step of a deterministic CTF Maze Arena solver run.',
  openGraph: {
    title: 'CTF Maze Arena Solve Replay',
    description: 'Step through a deterministic pathfinding run and inspect how the solver explored the maze.',
  },
};

export default function ReplayLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return children;
}
