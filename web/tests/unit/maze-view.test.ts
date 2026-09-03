import { describe, expect, it } from 'vitest';
import { backendMazeToMazeData, cellsToSvgPath, indexedCells, wallSegments } from '@/lib/maze-view';

function maze(walls: unknown[]) {
  return {
    grid: { width: 3, height: 2 }, walls: { inner: walls },
    start: { x: 0, y: 0 }, goal: { x: 2, y: 1 }, keys: [], doors: [],
  };
}

describe('maze view normalization', () => {
  it('normalizes cells and shared-boundary geometry', () => {
    const model = backendMazeToMazeData(maze([
      [{ x: 0, y: 0 }, { x: 1, y: 0 }],
      [{ x: 1, y: 0 }, { x: 1, y: 1 }],
    ]));
    expect(model.start).toEqual([0, 0]);
    expect(wallSegments(model.walls, 20)).toEqual([
      { x1: 20, y1: 0, x2: 20, y2: 20 },
      { x1: 20, y1: 20, x2: 40, y2: 20 },
    ]);
  });

  it.each([
    [[[{ x: 0, y: 0 }, { x: 2, y: 0 }]]],
    [[[{ x: 0, y: 0 }, { x: 0, y: 2 }]]],
    [[[{ x: 0, y: 0 }, { x: 1, y: 0 }], [{ x: 1, y: 0 }, { x: 0, y: 0 }]]],
  ])('rejects malformed, out-of-bounds, or duplicate walls', (walls) => {
    expect(() => backendMazeToMazeData(maze(walls))).toThrow(/wall edge/);
  });

  it('builds constant-time membership and compact layer paths', () => {
    const indexed = indexedCells([[0, 0], [2, 1], [9, 9]], 3, 2);
    expect(Array.from(indexed)).toEqual([1, 0, 0, 0, 0, 1]);
    expect(cellsToSvgPath([[1, 1]], 20, 2)).toBe('M22 22h16v16h-16Z');
  });
});
