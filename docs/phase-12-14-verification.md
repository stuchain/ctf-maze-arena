# Phase 12-14 Verification Matrix

This matrix maps each checklist requirement in phases 12, 13, and 14 to implementation locations and concrete verification commands.

## Phase 12: Containers and hosting

| Requirement | Implemented in | Verification command(s) | Expected result |
|---|---|---|---|
| Image runs as non-root | `Dockerfile` | `docker build -t ctf-maze-api .` then `docker run --rm ctf-maze-api whoami` | Output is non-root user (for this image, `app`) |
| SQLite path and volume are documented | `docs/deployment-runbook.md`, `.env.example`, `docker-compose.yml` | Inspect docs and compose mounts | Runbook references persistent volume and matching `DATABASE_URL`; compose maps `./data:/app/data` |
| Single replica caveat for SQLite | `docs/deployment-runbook.md` | Inspect runbook `SQLite and scaling` section | Explicit warning to run one instance per SQLite file |
| Local compose smoke flow | `docker-compose.yml` | `docker compose up --build -d` then `curl http://localhost:8080/api/health` | Health returns `ok` |

## Phase 13: Security baseline

| Requirement | Implemented in | Verification command(s) | Expected result |
|---|---|---|---|
| Security headers middleware | `src/main.rs` | `curl -sI http://localhost:8080/api/health` | Includes `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin` |
| CORS allowlist from `ALLOWED_ORIGINS` | `src/main.rs` | Allowed: set `ALLOWED_ORIGINS=http://example.com` and request with `Origin: http://example.com`; disallowed origin with `Origin: http://evil.com` | Allowed origin echoed in `Access-Control-Allow-Origin`; disallowed origin omitted |
| Release gating for permissive CORS | `src/main.rs`, `.env.example`, `docs/deployment-runbook.md` | Run release build with unset `ALLOWED_ORIGINS` and inspect behavior/log | Fails closed by default unless `CORS_PERMISSIVE=true` |
| WebSocket behavior remains valid | `src/api/mod.rs` | Open `/api/solve/stream?runId=...` in browser/devtools or ws client | Upgrade succeeds for valid flow |
| Production security env docs | `docs/deployment-runbook.md`, `.env.example` | Inspect docs table and notes | `ALLOWED_ORIGINS` documented as go-live requirement; secret-handling guidance present |

## Phase 14: Rate limiting

| Requirement | Implemented in | Verification command(s) | Expected result |
|---|---|---|---|
| Tunable env config for baseline limits | `src/main.rs`, `.env.example` | `cargo test` (env parsing tests) | Tests pass for defaults/invalid/fallback semantics |
| Global baseline limiter applied | `src/api/mod.rs`, `src/main.rs` | Burst parallel requests to baseline route | Requests exceed threshold and return `429` |
| Health endpoint not bricked under load | `src/api/mod.rs` | Burst traffic to limited routes while probing `/api/health` | `/api/health` continues returning `200` |
| Stricter limiter for `POST /api/solve` and `POST /api/maze/generate` | `src/api/mod.rs`, `src/main.rs`, `.env.example` | Send burst POSTs to expensive routes and compare against baseline route | Expensive routes hit `429` earlier than baseline routes |
| Proxy trust model documented | `docs/deployment-runbook.md`, `.env.example` | Inspect `Client IP and proxy trust` section | Docs explain peer-IP default, `TRUST_PROXY` behavior, and spoofing risk |
| WebSocket and health behavior under limits | `src/api/mod.rs` and verification script | Run verification script (below) | WS route exempt; health remains available |

## Reproducible verification entry point

- Script: `scripts/verify-phase12-14.ps1`
- Runs:
  - `cargo test`
  - header and CORS checks
  - baseline and expensive-route rate-limit checks
  - optional Docker and Compose checks

## Latest verification run (local)

- `cargo test`: pass (`45 + 7` tests, no failures).
- API verification script: pass with `powershell -ExecutionPolicy Bypass -File scripts/verify-phase12-14.ps1 -ApiBaseUrl http://127.0.0.1:18080`.
- `docker build -t ctf-maze-api .`: pass.
- `docker run --rm ctf-maze-api whoami`: pass (`app`, non-root).
- `docker compose up --build -d` + `GET /api/health`: pass (`200`), followed by `docker compose down`.
