'use client';

import { memo, useEffect, useId, useMemo, useRef, useState } from 'react';
import { MazeGlyph } from '@/components/BrandMark';
import { EmptyState, IconButton } from '@/components/ui/Primitives';
import {
  cellIndex,
  cellsToSvgPath,
  indexCell,
  indexedCells,
  wallSegments,
  type Cell,
  type MazeViewModel,
  type WallEdge,
} from '@/lib/maze-view';

export type MazeData = MazeViewModel;

export interface MazeGridProps {
  maze: MazeData | null;
  frontier?: Cell[];
  visited?: Cell[];
  current?: Cell;
  path?: Cell[];
}

const CELL_SIZE = 20;
const MIN_ZOOM = 1;
const MAX_ZOOM = 3;
const PAN_STEP = CELL_SIZE * 2;

interface LayerVisibility {
  visited: boolean;
  frontier: boolean;
  path: boolean;
}

const DEFAULT_LAYERS: LayerVisibility = { visited: true, frontier: true, path: true };

const StaticMazeLayer = memo(function StaticMazeLayer({ width, height, walls }: {
  width: number;
  height: number;
  walls: WallEdge[];
}) {
  const segments = useMemo(() => wallSegments(walls, CELL_SIZE), [walls]);
  const wallPath = useMemo(
    () => segments.map(({ x1, y1, x2, y2 }) => `M${x1} ${y1}L${x2} ${y2}`).join(''),
    [segments],
  );
  return (
    <g data-testid="maze-static-layer">
      <rect className="maze-base" x={0} y={0} width={width * CELL_SIZE} height={height * CELL_SIZE} />
      <path className="maze-walls" d={wallPath} data-testid="maze-inner-walls" />
      <rect className="maze-outer-wall" x={0} y={0} width={width * CELL_SIZE} height={height * CELL_SIZE} data-testid="maze-outer-wall" />
    </g>
  );
});

function markerPoint(cell: Cell) {
  return { x: cell[0] * CELL_SIZE + CELL_SIZE / 2, y: cell[1] * CELL_SIZE + CELL_SIZE / 2 };
}

function doorSegment([a, b]: WallEdge) {
  const segment = wallSegments([[a, b]], CELL_SIZE)[0];
  const horizontal = segment.y1 === segment.y2;
  return horizontal
    ? { x1: segment.x1 + 4, y1: segment.y1, x2: segment.x2 - 4, y2: segment.y2 }
    : { x1: segment.x1, y1: segment.y1 + 4, x2: segment.x2, y2: segment.y2 - 4 };
}

export function MazeGrid({ maze, frontier, visited, current, path }: MazeGridProps) {
  const titleId = useId();
  const descriptionId = useId();
  const containerRef = useRef<HTMLDivElement>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isFullscreen, setIsFullscreen] = useState(false);

  useEffect(() => {
    const syncFullscreenState = () => setIsFullscreen(document.fullscreenElement === containerRef.current);
    document.addEventListener('fullscreenchange', syncFullscreenState);
    return () => document.removeEventListener('fullscreenchange', syncFullscreenState);
  }, []);

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

  const { width, height, walls, start, goal, keys, doors } = maze;
  const safeSelectedIndex = Math.min(selectedIndex, width * height - 1);
  const selectedCell = indexCell(safeSelectedIndex, width);
  const visitedIndex = indexedCells(visited, width, height);
  const frontierIndex = indexedCells(frontier, width, height);
  const pathIndex = indexedCells(path, width, height);
  const selectedState = cellIndex(selectedCell, width);
  const stateLabel = current && current[0] === selectedCell[0] && current[1] === selectedCell[1]
    ? 'current'
    : start[0] === selectedCell[0] && start[1] === selectedCell[1]
      ? 'start'
      : goal[0] === selectedCell[0] && goal[1] === selectedCell[1]
        ? 'goal'
        : pathIndex[selectedState] ? 'solution path'
          : frontierIndex[selectedState] ? 'frontier'
            : visitedIndex[selectedState] ? 'visited' : 'unvisited';

  const resetView = () => { setZoom(1); setPan({ x: 0, y: 0 }); };
  const panBy = (x: number, y: number) => setPan((position) => ({ x: position.x + x, y: position.y + y }));
  const selectBy = (x: number, y: number) => {
    const nextX = Math.max(0, Math.min(width - 1, selectedCell[0] + x));
    const nextY = Math.max(0, Math.min(height - 1, selectedCell[1] + y));
    setSelectedIndex(cellIndex([nextX, nextY], width));
  };
  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    const selectionKeys: Record<string, Cell> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
    const panKeys: Record<string, Cell> = { a: [PAN_STEP, 0], d: [-PAN_STEP, 0], w: [0, PAN_STEP], s: [0, -PAN_STEP] };
    if (selectionKeys[event.key]) {
      event.preventDefault();
      selectBy(...selectionKeys[event.key]);
    } else if (panKeys[event.key.toLowerCase()]) {
      event.preventDefault();
      panBy(...panKeys[event.key.toLowerCase()]);
    } else if (event.key === '+' || event.key === '=') {
      event.preventDefault();
      setZoom((value) => Math.min(MAX_ZOOM, value + 0.25));
    } else if (event.key === '-') {
      event.preventDefault();
      setZoom((value) => Math.max(MIN_ZOOM, value - 0.25));
    } else if (event.key === '0') {
      event.preventDefault();
      resetView();
    }
  };
  const toggleFullscreen = async () => {
    if (document.fullscreenElement) await document.exitFullscreen();
    else await containerRef.current?.requestFullscreen();
  };

  const viewWidth = width * CELL_SIZE;
  const viewHeight = height * CELL_SIZE;
  const selectedPoint = markerPoint(selectedCell);
  const summary = `${width} by ${height} maze. ${visited?.length ?? 0} visited cells, ${frontier?.length ?? 0} frontier cells${current ? `, current cell ${current[0] + 1}, ${current[1] + 1}` : ''}.`;

  return (
    <div className="maze-experience" ref={containerRef} data-testid="maze-grid">
      <div className="maze-view-controls" aria-label="Maze view controls">
        <IconButton label="Zoom out" disabled={zoom === MIN_ZOOM} onClick={() => setZoom((value) => Math.max(MIN_ZOOM, value - 0.25))}>−</IconButton>
        <output className="zoom-readout" aria-live="polite">{Math.round(zoom * 100)}%</output>
        <IconButton label="Zoom in" disabled={zoom === MAX_ZOOM} onClick={() => setZoom((value) => Math.min(MAX_ZOOM, value + 0.25))}>+</IconButton>
        <IconButton label="Fit maze to stage" onClick={resetView}>Fit</IconButton>
        <IconButton label={isFullscreen ? 'Exit maze fullscreen' : 'View maze fullscreen'} onClick={() => void toggleFullscreen()}>⛶</IconButton>
      </div>
      <div className="maze-pan-controls" aria-label="Maze pan controls">
        <IconButton label="Pan maze up" onClick={() => panBy(0, PAN_STEP)}>↑</IconButton>
        <IconButton label="Pan maze left" onClick={() => panBy(PAN_STEP, 0)}>←</IconButton>
        <IconButton label="Pan maze right" onClick={() => panBy(-PAN_STEP, 0)}>→</IconButton>
        <IconButton label="Pan maze down" onClick={() => panBy(0, -PAN_STEP)}>↓</IconButton>
      </div>
      <div
        className="maze-viewport"
        tabIndex={0}
        role="group"
        aria-label="Interactive maze map"
        aria-describedby={descriptionId}
        onKeyDown={handleKeyDown}
        data-testid="maze-viewport"
      >
        <svg viewBox={`0 0 ${viewWidth} ${viewHeight}`} role="img" aria-labelledby={`${titleId} ${descriptionId}`} data-testid="maze-grid-svg">
          <title id={titleId}>Generated {width} by {height} maze</title>
          <desc id={descriptionId}>{summary} Use arrow keys to inspect cells, W A S D to pan, plus and minus to zoom, and 0 to fit.</desc>
          <g className="maze-transform" transform={`translate(${pan.x} ${pan.y}) scale(${zoom})`}>
            <StaticMazeLayer width={width} height={height} walls={walls} />
            {layers.visited && visited?.length ? <path className="maze-layer maze-layer--visited" d={cellsToSvgPath(visited, CELL_SIZE)} /> : null}
            {layers.frontier && frontier?.length ? <path className="maze-layer maze-layer--frontier" d={cellsToSvgPath(frontier, CELL_SIZE, 2)} /> : null}
            {layers.path && path?.length ? <path className="maze-layer maze-layer--path" d={cellsToSvgPath(path, CELL_SIZE, 5)} /> : null}
            {doors.map((door) => {
              const segment = doorSegment([door.from, door.to]);
              return <line key={`${door.from}-${door.to}`} className="maze-door" data-testid="maze-door" {...segment} />;
            })}
            {keys.map(({ cell, keyId }) => {
              const point = markerPoint(cell);
              return <g key={`${cell}-${keyId}`} className="maze-key" data-testid="maze-key" transform={`translate(${point.x} ${point.y})`}><circle r={5} /><text y={2.4}>{keyId + 1}</text></g>;
            })}
            <circle className="maze-marker maze-marker--start" {...markerPoint(start)} r={5} />
            <path className="maze-marker maze-marker--goal" d={`M${markerPoint(goal).x} ${markerPoint(goal).y - 6}l6 6-6 6-6-6Z`} />
            {current ? <circle className="maze-current" {...markerPoint(current)} r={6} /> : null}
            <rect className="maze-selection" x={selectedPoint.x - CELL_SIZE / 2 + 1} y={selectedPoint.y - CELL_SIZE / 2 + 1} width={CELL_SIZE - 2} height={CELL_SIZE - 2} />
          </g>
        </svg>
      </div>
      <div className="maze-inspection" aria-live="polite" data-testid="cell-inspection">
        <span>Selected Cell</span>
        <strong>Column {selectedCell[0] + 1}, Row {selectedCell[1] + 1}</strong>
        <small>{stateLabel}</small>
      </div>
      <fieldset className="maze-layer-toggles">
        <legend>Visible layers</legend>
        {(Object.keys(DEFAULT_LAYERS) as Array<keyof LayerVisibility>).map((layer) => (
          <label key={layer}><input type="checkbox" checked={layers[layer]} onChange={() => setLayers((value) => ({ ...value, [layer]: !value[layer] }))} />{layer}</label>
        ))}
      </fieldset>
      <details className="maze-shortcuts">
        <summary>Keyboard Help</summary>
        <p><kbd>Arrow keys</kbd> inspect · <kbd>W A S D</kbd> pan · <kbd>+ −</kbd> zoom · <kbd>0</kbd> fit</p>
      </details>
    </div>
  );
}
