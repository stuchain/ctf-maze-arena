'use client';

export interface MazeGridProps {
  width: number;
  height: number;
}

export function MazeGrid({ width, height }: MazeGridProps) {
  const cellSize = 24;

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${width}, ${cellSize}px)`,
        gridTemplateRows: `repeat(${height}, ${cellSize}px)`,
        gap: 0,
        border: '1px solid #333',
      }}
    >
      {Array.from({ length: width * height }, (_, i) => (
        <div
          key={i}
          style={{
            width: cellSize,
            height: cellSize,
            backgroundColor: '#f0f0f0',
            border: '1px solid #ddd',
          }}
        />
      ))}
    </div>
  );
}

