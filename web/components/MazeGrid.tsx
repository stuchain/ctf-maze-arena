'use client';

import { MazeGlyph } from '@/components/BrandMark';
import { EmptyState } from '@/components/ui/Primitives';

export interface MazeData {
  width: number;
  height: number;
  // Wall edges, each edge is [[ax, ay], [bx, by]] between two neighboring cells.
  walls: [[number, number], [number, number]][];
  start: [number, number];
  goal: [number, number];
}

export interface MazeGridProps {
  maze: MazeData | null;
  frontier?: [number, number][];
  visited?: [number, number][];
  current?: [number, number];
  path?: [number, number][];
}

function getCellFill(
  x: number,
  y: number,
  start: [number, number],
  goal: [number, number],
  current?: [number, number],
  path?: [number, number][],
  frontier?: [number, number][],
  visited?: [number, number][],
) {
  if (start[0] === x && start[1] === y) return 'var(--maze-start)';
  if (goal[0] === x && goal[1] === y) return 'var(--maze-goal)';
  if (current && current[0] === x && current[1] === y) return 'var(--maze-current)';
  if (path?.some(([px, py]) => px === x && py === y)) return 'var(--maze-path)';
  if (frontier?.some(([fx, fy]) => fx === x && fy === y)) return 'var(--maze-frontier)';
  if (visited?.some(([vx, vy]) => vx === x && vy === y)) return 'var(--maze-visited)';
  return 'var(--maze-unvisited)';
}

export function MazeGrid({ maze, frontier, visited, current, path }: MazeGridProps) {
  if (!maze) {
    return (
      <div data-testid="maze-grid-empty" className="maze-empty">
        <EmptyState
          icon={<MazeGlyph size={34} />}
          title="Your Arena Is Ready"
          description="Choose a configuration or load today’s challenge to generate a deterministic maze."
        />
      </div>
    );
  }

  const { width, height, walls, start, goal } = maze;
  const cellSize = 24;

  return (
    <div
      className="maze-viewport"
      data-testid="maze-grid"
    >
      <svg
        viewBox={`0 0 ${width * cellSize + 1} ${height * cellSize + 1}`}
        role="img"
        aria-labelledby="maze-title maze-description"
        data-testid="maze-grid-svg"
      >
        <title id="maze-title">Generated {width} by {height} maze</title>
        <desc id="maze-description">The start is green, goal is coral, frontier is cyan, visited cells are slate, current cell is amber, and the solution path is violet.</desc>
        {/* Cells */}
        {Array.from({ length: width * height }, (_, i) => {
          const x = i % width;
          const y = Math.floor(i / width);

          return (
            <rect
              key={i}
              x={x * cellSize + 1}
              y={y * cellSize + 1}
              width={cellSize - 1}
              height={cellSize - 1}
              fill={getCellFill(x, y, start, goal, current, path, frontier, visited)}
            />
          );
        })}

        {/* Walls: lines between cell centers */}
        {walls.map(([[ax, ay], [bx, by]], i) => (
          <line
            key={i}
            x1={ax * cellSize + cellSize / 2}
            y1={ay * cellSize + cellSize / 2}
            x2={bx * cellSize + cellSize / 2}
            y2={by * cellSize + cellSize / 2}
            stroke="var(--maze-wall)"
            strokeWidth={2}
          />
        ))}
      </svg>
    </div>
  );
}

