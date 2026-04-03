import type { MazeData } from '@/components/MazeGrid';

export function toCellTuple(cell: unknown): [number, number] {
  if (Array.isArray(cell) && cell.length >= 2) {
    return [Number(cell[0]), Number(cell[1])];
  }
  if (cell && typeof cell === 'object' && 'x' in cell && 'y' in cell) {
    const o = cell as { x: unknown; y: unknown };
    return [Number(o.x), Number(o.y)];
  }
  return [0, 0];
}

export function backendMazeToMazeData(backendMaze: unknown): MazeData {
  const m = backendMaze as Record<string, unknown>;
  const width = Number((m?.grid as { width?: unknown })?.width ?? 0);
  const height = Number((m?.grid as { height?: unknown })?.height ?? 0);

  const start = toCellTuple(m?.start);
  const goal = toCellTuple(m?.goal);

  const wallsObj = m?.walls as { inner?: unknown } | undefined;
  const rawEdges: unknown[] = Array.isArray(wallsObj?.inner)
    ? (wallsObj.inner as unknown[])
    : [];

  const walls: MazeData['walls'] = rawEdges
    .map((edge: unknown) => {
      if (Array.isArray(edge) && edge.length === 2) {
        return [toCellTuple(edge[0]), toCellTuple(edge[1])] as [
          [number, number],
          [number, number],
        ];
      }
      if (edge && typeof edge === 'object' && 0 in edge && 1 in edge) {
        const e = edge as { 0: unknown; 1: unknown };
        return [toCellTuple(e[0]), toCellTuple(e[1])] as [
          [number, number],
          [number, number],
        ];
      }
      return null;
    })
    .filter(Boolean) as MazeData['walls'];

  return { width, height, walls, start, goal };
}
