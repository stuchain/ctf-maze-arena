'use client';

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
}

function getCellFill(
  x: number,
  y: number,
  start: [number, number],
  goal: [number, number],
  current?: [number, number],
  frontier?: [number, number][],
  visited?: [number, number][],
) {
  if (start[0] === x && start[1] === y) return '#4ade80';
  if (goal[0] === x && goal[1] === y) return '#f87171';
  if (current && current[0] === x && current[1] === y) return '#fde047';
  if (frontier?.some(([fx, fy]) => fx === x && fy === y)) return '#93c5fd';
  if (visited?.some(([vx, vy]) => vx === x && vy === y)) return '#d1d5db';
  return '#f0f0f0';
}

export function MazeGrid({ maze, frontier, visited, current }: MazeGridProps) {
  if (!maze) return <div>No maze</div>;

  const { width, height, walls, start, goal } = maze;
  const cellSize = 24;

  return (
    <div style={{ display: 'inline-block', position: 'relative' }}>
      <svg width={width * cellSize + 1} height={height * cellSize + 1}>
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
              fill={getCellFill(x, y, start, goal, current, frontier, visited)}
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
            stroke="#333"
            strokeWidth={2}
          />
        ))}
      </svg>
    </div>
  );
}

