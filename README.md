# ctf-maze-arena

`ctf-maze-arena` is a maze algorithm playground. The Rust backend currently includes maze core data structures, deterministic generators, validation utilities, and solvers (BFS, DFS, A*, DP with keys/doors).

The Next.js frontend is scaffolded, with the interactive arena UI and full integration in progress.

## Goal

Build an interactive maze arena to generate mazes, run multiple solvers, and compare solver behavior/performance.

## Current Status

- Core maze and solver logic is implemented and tested in Rust.
- Key/door puzzle support is implemented in the maze model and DP solver.
- Frontend exists as a Next.js scaffold; arena UI is still in progress.
- Backend runtime wiring is minimal right now (core modules are ready, full API/server routes are next).

## Built So Far

- Maze core: `Cell`, `Grid`, `Walls`, `Maze`, plus key/door support.
- Generators: Kruskal, Prim, DFS backtracker (seeded/deterministic).
- Validators: connectivity, wall symmetry, start-to-goal reachability.
- Solvers: BFS, DFS, A*, DP (keys/doors state with bitmask).
- Tests: unit/integration-style tests across maze + solver modules.
- Frontend: Next.js app scaffold and client helpers (UI still minimal).

## Quick Start

### Backend
```bash
cargo run
```

Run backend tests:
```bash
cargo test
```

### Frontend
```bash
cd web
npm install
npm run dev
```

## Project Structure

- `src/maze/` - maze model, generation, validation
- `src/solve/` - solver trait, registry, BFS/DFS/A*/DP solvers
- `web/` - Next.js frontend scaffold
- `docs/commit/` - phase-by-phase implementation notes


