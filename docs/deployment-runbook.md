# Deployment runbook (free-tier hosts)

This project ships a Rust API in a container ([`Dockerfile`](../Dockerfile)) with **SQLite** on disk. Use this runbook to deploy without surprises.

## SQLite and scaling

- SQLite allows **one writer at a time** on a single file. Run **one container instance** per database file.
- Do **not** point multiple replicas at the same SQLite path on shared storage; you risk locking failures or corruption.
- A future phase can move to Postgres for multi-instance and HA.

## Environment variables

| Variable | Purpose | Example | Required in prod? |
|----------|---------|---------|-------------------|
| `PORT` | HTTP listen port inside the container (the app reads `PORT`; default `8080`). | `PORT=10000` | Usually platform-provided |
| `DATABASE_URL` | SQLite path; must be on persistent storage in production. | `DATABASE_URL=sqlite:./data/ctf_maze.db` | Yes |
| `RUST_LOG` | Application log verbosity. | `RUST_LOG=info` | Recommended |
| `ALLOWED_ORIGINS` | Comma-separated browser origin allowlist for CORS. Trailing slashes are normalized. | `ALLOWED_ORIGINS=https://app.example.com,https://www.example.com` | Yes (go-live requirement) |
| `CORS_PERMISSIVE` | Escape hatch to allow permissive CORS in release when explicitly set to `true`. | `CORS_PERMISSIVE=true` | No (use only for controlled staging) |
| `RATE_LIMIT_PER_SECOND` | Baseline per-IP refill rate for limited API routes. | `RATE_LIMIT_PER_SECOND=20` | Recommended |
| `RATE_LIMIT_BURST` | Baseline per-IP burst size. | `RATE_LIMIT_BURST=40` | Recommended |
| `RATE_LIMIT_EXPENSIVE_PER_SECOND` | Stricter per-IP refill rate for expensive routes (`POST /api/solve`, `POST /api/maze/generate`). | `RATE_LIMIT_EXPENSIVE_PER_SECOND=5` | Recommended |
| `RATE_LIMIT_EXPENSIVE_BURST` | Stricter per-IP burst size for expensive routes. | `RATE_LIMIT_EXPENSIVE_BURST=10` | Recommended |
| `TRUST_PROXY` | Toggle proxy-aware IP extraction for rate limiting (`SmartIpKeyExtractor` when `true`). | `TRUST_PROXY=false` | No (enable only behind trusted proxy) |

Platforms often set `PORT` for you. **If the app fails to bind**, confirm you are not hardcoding `8080` in the platform UI while the process expects another port.

Do not commit `.env` files with real credentials or tokens. Keep secrets in your platform's secret manager (Render/Fly/etc.).

## TLS

- Terminate **TLS at the edge** (managed load balancer, reverse proxy, or the PaaS HTTPS endpoint).
- The container serves plain HTTP; do not rely on TLS inside the image for production.

## CORS and browser origins

- `ALLOWED_ORIGINS` controls which browser origins may call the API cross-origin.
- If `ALLOWED_ORIGINS` is unset, the app falls back to permissive behavior for local development convenience.
- If `ALLOWED_ORIGINS` is set but empty/invalid, cross-origin CORS is disabled.
- Before go-live, set `ALLOWED_ORIGINS` explicitly to your production web origins.

## Client IP and proxy trust (rate limiting)

- Default mode (`TRUST_PROXY=false`) keys rate limits by direct peer socket IP. This is safest and avoids header spoofing risk.
- Proxy mode (`TRUST_PROXY=true`) uses forwarded headers (`x-forwarded-for`, `x-real-ip`, `forwarded`) to infer client IP.
- Enable `TRUST_PROXY=true` only when your platform/load balancer overwrites or strips inbound forwarding headers from clients.
- If this is misconfigured, attackers can spoof forwarding headers and evade per-IP limits.
- For Render/Fly/Cloudflare-style deployments, confirm trusted proxy behavior in platform docs before enabling proxy mode.

## Render (Docker)

1. Create a **Web Service** from the repo with **Docker** runtime (root [`Dockerfile`](../Dockerfile)).
2. Add a **persistent disk** and mount it at a path that matches `DATABASE_URL` (for example mount at `/data` and set `DATABASE_URL=sqlite:./data/ctf_maze.db` with `WORKDIR` `/app` and a subfolder, or mount at `/app/data` to match the default).
3. Set `PORT` if Render does not inject it automatically for your service type (their docs describe the env var they provide).
4. Keep **instance count at one** for SQLite unless you migrate the database.

## Fly.io

1. Use `fly launch` (or a `fly.toml`) with the same Docker image.
2. Add a **volume** with `[[mounts]]` and mount it where the DB file lives; set `DATABASE_URL` accordingly (e.g. under `/data` in the machine).
3. Set secrets: `fly secrets set DATABASE_URL=...` (and `RUST_LOG` if desired). Use Fly’s assigned **internal/external port** documentation so `PORT` matches what the proxy expects.
4. Prefer **one region / one machine** for SQLite workloads on a single file.

## Health check

- `GET /api/health` should return `ok`. Configure your platform health check to use that path.

## Local parity

- See [`docker-compose.yml`](../docker-compose.yml) and [`.env.example`](../.env.example) for a local stack with a bind-mounted `./data` directory.
- Use [`scripts/verify-phase12-14.ps1`](../scripts/verify-phase12-14.ps1) for reproducible phase 12-14 checks.
- Checklist/evidence mapping is tracked in [`phase-12-14-verification.md`](./phase-12-14-verification.md).
