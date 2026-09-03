# Phase 5 — Maze visualization and replay

**Status:** ready

**Depends on:** Phases 3 and 4

**Purpose:** make the maze a correct, fast, accessible, and memorable visual centerpiece

## Problem

Walls are currently drawn as lines between cell centers rather than shared cell boundaries. Cell styling repeatedly scans path/frontier/visited arrays, sizing is fixed, and replay controls are minimal. Large mazes and nuanced solver state do not read as a polished visualization.

## Goals

- Render correct maze geometry at responsive sizes.
- Visualize solver state smoothly without wasteful per-cell scans.
- Provide complete playback, navigation, legend, and status controls.
- Make replays shareable and understandable without exposing internal IDs as primary copy.
- Preserve meaning for keyboard, screen-reader, zoom, and reduced-motion users.

## Rendering design

### Geometry

Treat a wall pair as the blocked shared boundary between adjacent cells. Draw outer boundaries separately. Reject malformed/non-adjacent edges during normalization. Use a view box and responsive container so the maze fits available space independent of cell count.

### Layers

Render in stable order: base grid, visited, frontier, path, walls, start/goal/key/door markers, current node, focus/hover affordance. State precedence is explicit and mirrored in the legend.

### Technology decision gate

Start with optimized SVG because it supports crisp scalable geometry, semantics, and straightforward interaction. Benchmark representative 50×50 state updates. Move dynamic cell layers to Canvas only if measured SVG performance misses the target; retain accessible DOM controls and a textual summary either way.

### State representation

Normalize cells to numeric indices and use sets, typed arrays, or bitsets for constant-time membership. Apply protocol deltas to a reducer-owned visual state. Memoize static wall geometry and avoid React elements for every unchanged detail on every frame.

## Interaction design

- Generate, start, pause, resume, cancel, step backward/forward, reset.
- Timeline scrubber with current/total logical step.
- Playback speed options and “fit to stage.”
- Zoom, pan, reset view, fullscreen, and keyboard shortcuts with discoverable help.
- Toggleable layers and persistent legend.
- Results summary: path cost, visited count, runtime, and solver guarantee.

The live run and stored replay use one playback model. A live run follows the newest frame until the user scrubs backward; a clear action returns to live.

## Accessibility design

The graphic has a concise label and a detailed textual state summary, not thousands of focusable cells by default. Playback controls use standard names/states and keyboard operation. Announcements are throttled to meaningful milestones rather than every frame. Users can inspect a selected cell through a keyboard-operable details panel.

## Performance targets

- Smooth perceived playback for a representative 50×50 maze on a typical laptop.
- No unbounded frame array retained in React component state.
- Static geometry does not rerender for each progress delta.
- Frame application remains comfortably below the selected presentation interval.
- Bundle and memory measurements are recorded rather than claimed informally.

## Work checklist

- [ ] Define validated maze-view and replay-view models.
- [ ] Correct shared-boundary and outer-wall geometry.
- [ ] Implement indexed state and memoized static layers.
- [ ] Build responsive stage, legend, markers, zoom/pan/fit/fullscreen.
- [ ] Build unified live/replay playback state machine and controls.
- [ ] Add textual summary, selected-cell inspection, shortcuts, and reduced-motion behavior.
- [ ] Add share/copy feedback and polished replay loading/error metadata.
- [ ] Benchmark SVG and document whether a Canvas hybrid is needed.

## Test strategy

- Geometry unit tests using known small mazes and malformed edges.
- Reducer tests for snapshot/delta application and scrubbing.
- Component tests for every playback control and keyboard shortcut.
- Visual regression for layers, themes, sizes, start/goal, keys/doors, and terminal path.
- Performance benchmark with recorded browser trace for target maze sizes.
- E2E live-to-replay parity check.

## Risks

- A visually impressive animation can misrepresent algorithm timing. Clearly distinguish logical playback duration from measured solver runtime.
- Canvas can improve performance while reducing accessibility and testability. It is a measured fallback, not the starting assumption.
- Zoom/pan libraries can inflate the bundle. Prefer a focused local implementation if requirements are modest.

## Exit criteria

- [ ] Known mazes render geometrically correct walls and markers.
- [ ] Live and replay playback share consistent controls and state.
- [ ] Target performance and memory evidence is recorded.
- [ ] Keyboard, reduced-motion, zoom, and textual alternatives are verified.
- [ ] Replay pages are polished, shareable, and resilient to missing data.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| — | Not implemented | — |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| — | None | — |
