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
}

export function MazeGrid({ maze }: MazeGridProps) {
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
          const isStart = x === start[0] && y === start[1];
          const isGoal = x === goal[0] && y === goal[1];

          return (
            <rect
              key={i}
              x={x * cellSize + 1}
              y={y * cellSize + 1}
              width={cellSize - 1}
              height={cellSize - 1}
              fill={isStart ? '#4ade80' : isGoal ? '#f87171' : '#f0f0f0'}
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

