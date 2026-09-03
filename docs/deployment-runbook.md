# Deployment runbook

The Rust API is shipped as a non-root container and requires PostgreSQL. Phase 8 adds the final Koyeb, Neon, and Vercel production procedure; this document records the backend runtime contract established in Phase 2.

## Environment variables

| Variable | Purpose | Example | Required in production? |
|---|---|---|---|
| `PORT` | HTTP listen port; defaults to `8080`. | `PORT=8080` | Platform-dependent |
| `DATABASE_URL` | PostgreSQL connection string. Add provider-required TLS parameters such as `sslmode=require`. | `postgresql://user:password@host/database?sslmode=require` | Yes |
| `DB_MAX_CONNECTIONS` | Maximum SQLx pool size; defaults to `5`. | `DB_MAX_CONNECTIONS=5` | Recommended |
| `DB_CONNECT_ATTEMPTS` | Bounded startup connection attempts; defaults to `5`. | `DB_CONNECT_ATTEMPTS=5` | Recommended |
| `MAX_CONCURRENT_SOLVES` | CPU-bound solver concurrency; defaults to `1`. | `MAX_CONCURRENT_SOLVES=1` | Recommended |
| `RUST_LOG` | Application log filter. | `RUST_LOG=info` | Recommended |
| `LOG_FORMAT` | `pretty` or machine-readable `json`. | `LOG_FORMAT=json` | Recommended |
| `ALLOWED_ORIGINS` | Comma-separated exact browser origins. | `ALLOWED_ORIGINS=https://example.vercel.app` | Yes |
| `TRUST_PROXY` | Enables forwarded-header rate-limit identity. | `TRUST_PROXY=true` | Only behind a trusted proxy |
| `RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST` | Baseline per-IP limits. | `20` / `40` | Recommended |
| `RATE_LIMIT_EXPENSIVE_PER_SECOND` / `RATE_LIMIT_EXPENSIVE_BURST` | Generate/solve limits. | `5` / `10` | Recommended |
| `JWT_SECRET` | Shared HMAC secret for web-issued API tokens. | 32+ random characters | Required when auth is enabled |
| `JWT_CLOCK_SKEW_SECS` | JWT clock tolerance. | `60` | Recommended |
| `AUTH_MODE` | `anonymous`, `optional_jwt`, or `jwt`. | `optional_jwt` | Recommended |

Never commit database credentials, OAuth credentials, or JWT secrets. Store them in the hosting provider’s secret manager.

## Startup and database behavior

- Startup rejects missing or non-PostgreSQL `DATABASE_URL` values.
- SQLx connects with a bounded pool and bounded retry loop, then runs forward-only embedded migrations.
- Any `queued` or `running` jobs left by a process interruption become terminal `failed` runs with `worker_interrupted` before traffic is accepted.
- PostgreSQL owns durable data; no container volume is required for the API.

## Health and observability

- `GET /api/health` is liveness and returns build version/SHA without requiring PostgreSQL.
- `GET /api/ready` checks PostgreSQL and returns `503` with a safe error envelope when unavailable. Use this for readiness routing.
- Set `LOG_FORMAT=json` in hosted environments and retain the `x-request-id` returned by failed requests.

## TLS, CORS, and proxy trust

Terminate HTTPS at the hosting edge. Set `ALLOWED_ORIGINS` to the exact Vercel production and preview origins that should access the API. Set `TRUST_PROXY=true` only when the platform overwrites inbound forwarding headers; otherwise clients could spoof rate-limit identity.

## GitHub authentication

- Local callback: `http://localhost:3000/api/auth/callback/github`
- Production callback: `https://<vercel-domain>/api/auth/callback/github`
- The web app signs the stable GitHub provider account ID into short-lived API JWTs. Display names and email addresses are never used as ownership keys.
- Anonymous solve creation stays available. Leaderboard submission requires a matching authenticated owner.

## Local parity

Run `docker compose up --build` to start PostgreSQL, the API, and the web app. The compose database uses a named development volume; remove that volume only when intentionally resetting local data. Run `./scripts/verify.ps1` for canonical checks, and use `TEST_DATABASE_URL` pointing at a dedicated database for integration tests.
