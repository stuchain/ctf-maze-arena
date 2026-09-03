import { z } from 'zod';

export type Cell = [number, number];
export type WallEdge = [Cell, Cell];

export interface MazeKey {
  cell: Cell;
  keyId: number;
}

export interface MazeDoor {
  from: Cell;
  to: Cell;
  keyId: number;
}

export interface MazeViewModel {
  width: number;
  height: number;
  walls: WallEdge[];
  start: Cell;
  goal: Cell;
  keys: MazeKey[];
  doors: MazeDoor[];
}

export interface WallSegment {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

const dimensionSchema = z.number().int().min(1).max(100);
const coordinateSchema = z.number().int().nonnegative();
export const cellSchema = z.union([
  z.tuple([coordinateSchema, coordinateSchema]),
  z.object({ x: coordinateSchema, y: coordinateSchema }).transform(({ x, y }) => [x, y] as Cell),
]);
const edgeSchema = z.union([
  z.tuple([cellSchema, cellSchema]),
  z.object({ 0: cellSchema, 1: cellSchema }).transform((edge) => [edge[0], edge[1]] as WallEdge),
]);
const mazeSchema = z.object({
  grid: z.object({ width: dimensionSchema, height: dimensionSchema }),
  walls: z.object({ inner: z.array(edgeSchema) }),
  start: cellSchema,
  goal: cellSchema,
  keys: z.unknown().optional(),
  doors: z.unknown().optional(),
});

function cellKey([x, y]: Cell) {
  return `${x}:${y}`;
}

function edgeKey([a, b]: WallEdge) {
  const first = cellKey(a);
  const second = cellKey(b);
  return first < second ? `${first}|${second}` : `${second}|${first}`;
}

export function cellIndex([x, y]: Cell, width: number) {
  return y * width + x;
}

export function indexCell(index: number, width: number): Cell {
  return [index % width, Math.floor(index / width)];
}

export function isCellInBounds([x, y]: Cell, width: number, height: number) {
  return x >= 0 && y >= 0 && x < width && y < height;
}

export function areAdjacent([ax, ay]: Cell, [bx, by]: Cell) {
  return Math.abs(ax - bx) + Math.abs(ay - by) === 1;
}

function parseKeys(value: unknown, width: number, height: number): MazeKey[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((entry) => {
    const parsed = z.tuple([cellSchema, z.number().int().min(0).max(31)]).safeParse(entry);
    if (!parsed.success || !isCellInBounds(parsed.data[0], width, height)) return [];
    return [{ cell: parsed.data[0], keyId: parsed.data[1] }];
  });
}

function parseDoors(value: unknown, width: number, height: number): MazeDoor[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((entry) => {
    const parsed = z.tuple([edgeSchema, z.number().int().min(0).max(31)]).safeParse(entry);
    if (!parsed.success) return [];
    const [from, to] = parsed.data[0];
    if (!isCellInBounds(from, width, height) || !isCellInBounds(to, width, height) || !areAdjacent(from, to)) return [];
    return [{ from, to, keyId: parsed.data[1] }];
  });
}

export function backendMazeToMazeData(value: unknown): MazeViewModel {
  const parsed = mazeSchema.safeParse(value);
  if (!parsed.success) throw new Error('The API returned an invalid maze model.');
  const { width, height } = parsed.data.grid;
  if (!isCellInBounds(parsed.data.start, width, height) || !isCellInBounds(parsed.data.goal, width, height)) {
    throw new Error('The maze start or goal is outside the grid.');
  }

  const seen = new Set<string>();
  const walls = parsed.data.walls.inner.filter((edge) => {
    const valid = edge.every((cell) => isCellInBounds(cell, width, height)) && areAdjacent(edge[0], edge[1]);
    const key = edgeKey(edge);
    if (!valid || seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  if (walls.length !== parsed.data.walls.inner.length) {
    throw new Error('The API returned a malformed or duplicate wall edge.');
  }

  return {
    width,
    height,
    walls,
    start: parsed.data.start,
    goal: parsed.data.goal,
    keys: parseKeys(parsed.data.keys, width, height),
    doors: parseDoors(parsed.data.doors, width, height),
  };
}

export function wallSegments(walls: WallEdge[], cellSize: number): WallSegment[] {
  return walls.map(([a, b]) => {
    if (a[0] !== b[0]) {
      const boundaryX = Math.max(a[0], b[0]) * cellSize;
      return { x1: boundaryX, y1: a[1] * cellSize, x2: boundaryX, y2: (a[1] + 1) * cellSize };
    }
    const boundaryY = Math.max(a[1], b[1]) * cellSize;
    return { x1: a[0] * cellSize, y1: boundaryY, x2: (a[0] + 1) * cellSize, y2: boundaryY };
  });
}

export function indexedCells(cells: Cell[] | undefined, width: number, height: number) {
  const result = new Uint8Array(width * height);
  for (const cell of cells ?? []) {
    if (isCellInBounds(cell, width, height)) result[cellIndex(cell, width)] = 1;
  }
  return result;
}

export function cellsToSvgPath(cells: Cell[] | undefined, cellSize: number, inset = 1) {
  let path = '';
  const size = Math.max(0, cellSize - inset * 2);
  for (const [x, y] of cells ?? []) {
    path += `M${x * cellSize + inset} ${y * cellSize + inset}h${size}v${size}h-${size}Z`;
  }
  return path;
}
