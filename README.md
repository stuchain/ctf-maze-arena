# ctf-maze-arena

A maze playground with a Next.js frontend and Rust (Axum) API.  
Generate mazes, run solvers, stream solve progress, and view replays/leaderboards.

## Stack

- Backend: Rust, Axum, SQLx, PostgreSQL
- Frontend: Next.js, React, TypeScript
- Testing: Rust tests, Vitest unit tests, and Playwright E2E

## Prerequisites

- Rust (stable) + Cargo
- Node.js + npm
- PostgreSQL 16+ (or Docker Compose)

## Quick Start

1. Copy `.env.example` to `.env` (repo root) and start PostgreSQL (`docker compose up postgres -d` is the quickest option).
2. Start the backend:

   ```bash
   cargo run
   ```

3. Start frontend:

   ```bash
   cd web
   cp .env.example .env.local
   npm ci
   npm run dev
   ```

Backend default: `http://localhost:8080`  
Frontend default: `http://localhost:3000`

Run the canonical repository checks from PowerShell with `./scripts/verify.ps1`.

## Project structure

- `src/main.rs` - API server entrypoint
- `src/api/` - HTTP + WebSocket handlers
- `src/maze/` - maze generation/model
- `src/solve/` - solver implementations
- `src/services/` - domain workflows and background execution
- `src/store/` - PostgreSQL persistence
- `migrations/` - database migrations
- `web/` - Next.js app

## Authentication

- Web auth: GitHub OAuth via NextAuth
- API auth: short-lived Bearer JWT from `/api/token`
- `AUTH_MODE` options:
  - `anonymous` (default)
  - `optional_jwt`
  - `jwt` (requires a JWT for protected identity operations; anonymous solves remain available)
- Quick rollback: set `AUTH_MODE=anonymous`

## Useful docs

- `docs/v0.1/README.md` — active v0.1 design roadmap and phase documents
- `docs/API.md`
- `docs/deployment-runbook.md`
- `docs/e2e-runbook.md`
- `docs/observability-runbook.md`
- `docs/ALGORITHMS.md`
