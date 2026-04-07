# ctf-maze-arena

Interactive maze playground: generate seeded mazes (Kruskal, Prim, DFS), run solvers (BFS, DFS, A*, DP with keys/doors), stream solve animation over WebSocket, and browse leaderboards, replays, and a daily challenge. A Next.js UI talks to a Rust (Axum) backend with SQLite persistence.

## Tech stack

- **Backend:** Rust, Axum, SQLx + SQLite, Tokio, serde/json
- **Frontend:** Next.js (React), TypeScript
- **Tooling:** Criterion benchmarks (`cargo bench`), sqlx migrations

## Prerequisites

- **Rust** (stable) and **Cargo**
- **Node.js** and **npm** (for `web/`)
- Optional: **SQLite** file path you can write to (default `./data/ctf_maze.db`; parent directory is created on startup)

## Documentation

- [docs/API.md](docs/API.md) — HTTP and WebSocket API overview
- [docs/ALGORITHMS.md](docs/ALGORITHMS.md) — maze generation and solver notes
- [docs/deployment-runbook.md](docs/deployment-runbook.md) — deployment and environment operations
- [docs/observability-runbook.md](docs/observability-runbook.md) — request tracing, log triage, and 429 handling
- [docs/e2e-runbook.md](docs/e2e-runbook.md) — local Playwright setup, workflows, and trace debugging

## Quick start

1. Copy [`.env.example`](.env.example) to `.env` in the repo root and adjust if needed (`DATABASE_URL`, `PORT`, `RUST_LOG`). For the frontend, use `web/.env.local` with `NEXT_PUBLIC_API_URL` (default `http://localhost:8080`).
2. **Backend:** from the repo root:

```bash
cargo run
```

The server listens on `0.0.0.0` and the port from `PORT` (default **8080**). Migrations run automatically. Endpoints are under `/api`.

3. **Frontend:**

```bash
cd web
npm install
npm run dev
```

4. **Tests:**

```bash
cargo test
```

## Benchmarks

Run `cargo bench` for full Criterion output (HTML reports under `target/criterion/` when enabled). Example below used `cargo bench` with shorter warm-up/measurement windows (30 samples); **release** profile, **Windows 11 x86_64**, local dev machine.

| Benchmark       | Mean (ms) | Std (ms) |
|-----------------|-----------|----------|
| kruskal_10x10   | 0.026     | 0.001    |
| kruskal_50x50   | 0.91      | 0.02     |
| prim_10x10      | 0.059     | 0.002    |
| dfs_10x10       | 0.051     | 0.002    |
| bfs_20x20       | 0.53      | 0.03     |
| dfs_20x20       | 0.17      | 0.004    |
| astar_20x20     | 0.24      | 0.005    |

## Project structure

- `src/lib.rs` — library crate (maze, solvers, API, store, replay) used by the binary and benchmarks
- `src/main.rs` — HTTP server entrypoint
- `src/maze/` — maze model, generation, validation
- `src/solve/` — solver trait, registry, BFS / DFS / A* / DP
- `src/api/` — Axum REST + WebSocket
- `src/replay/` — replay JSON format
- `src/store/` — SQLite persistence
- `migrations/` — sqlx migrations
- `benches/` — Criterion benchmarks (`maze_gen`, `solvers`)
- `web/` — Next.js app (`web/components/`, `web/app/`, …)

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE).
