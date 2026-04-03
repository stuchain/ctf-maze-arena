# ctf-maze-arena

`ctf-maze-arena` is a maze algorithm playground. The Rust backend currently includes maze core data structures, deterministic generators, validation utilities, and solvers (BFS, DFS, A*, DP with keys/doors).

The Next.js frontend is scaffolded, with the interactive arena UI and full integration in progress.

## Goal

Build an interactive maze arena to generate mazes, run multiple solvers, and compare solver behavior/performance.

## Current Status

- Core maze and solver logic is implemented and tested in Rust.
- Key/door puzzle support is implemented in the maze model and DP solver.
- Frontend exists as a Next.js app; core UI for generating a maze and starting a solve is implemented.
- Backend now includes replay + persistence modules (SQLite/sqlx) and also exposes an API/WebSocket under `/api` for:
  - maze generation
  - starting a solve job
  - streaming solve progress over WebSocket
  - fetching stored replays
  - returning a leaderboard

## Built So Far

- Maze core: `Cell`, `Grid`, `Walls`, `Maze`, plus key/door support.
- Generators: Kruskal, Prim, DFS backtracker (seeded/deterministic).
- Validators: connectivity, wall symmetry, start-to-goal reachability.
- Solvers: BFS, DFS, A*, DP (keys/doors state with bitmask).
- Tests: unit/integration-style tests across maze + solver modules.
- Replay + persistence: replay frames/JSON and SQLite (migrations + `src/store`) for mazes/runs/replays.
- Backend API + WebSocket: Axum routes under `/api` (maze generate/solve, `/api/replay/:run_id`, `/api/leaderboard`, and `GET /api/solve/stream`).
- Frontend: Next.js core UI components for generating a maze, drawing it, choosing a solver, and triggering solve.

## Quick Start

### Backend
```bash
cargo run
```
If using persistence, set `DATABASE_URL` (default: `sqlite:./data/ctf_maze.db`).
The backend will create the `data/` folder and the SQLite DB file if they don’t exist yet.
Migrations run automatically on backend startup.
Backend listens on `http://localhost:8080` and serves endpoints under `/api`.

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
The frontend calls the backend using `NEXT_PUBLIC_API_URL` (default: `http://localhost:8080`).

## Project Structure

- `src/maze/` - maze model, generation, validation
- `src/solve/` - solver trait, registry, BFS/DFS/A*/DP solvers
- `src/api/` - Axum REST/WebSocket API under `/api`
- `src/replay/` - replay frame + JSON replay format
- `src/store/` - SQLite/sqlx persistence for mazes/runs/replays
- `migrations/` - SQLite schema migrations used by the backend
- `web/` - Next.js frontend scaffold
- `web/components/` - core UI components (maze drawing, solver picker, generate form)


