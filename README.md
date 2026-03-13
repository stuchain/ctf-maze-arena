# ctf-maze-arena

A web arena for maze generation, algorithm visualization, and solver comparison. Built with Rust + TypeScript.

## Tech Stack

- **Backend:** Rust, axum, tokio, serde
- **Frontend:** Next.js (App Router), React, TypeScript
- **Database:** SQLite (via sqlx)

## Prerequisites

- Rust 1.70+
- Node.js 18+
- SQLite 3

## Quick Start

### Backend
```bash
cargo run
```
Server runs on `http://localhost:8080` (or port from env).

### Frontend
```bash
cd web && npm install && npm run dev
```
App runs on `http://localhost:3000`.

## Project Structure

- `src/` — Rust backend
- `web/` — Next.js frontend
- `docs/` — Phase docs, API reference (see Phase 10)
