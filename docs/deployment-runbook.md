# Deployment runbook (free-tier hosts)

This project ships a Rust API in a container ([`Dockerfile`](../Dockerfile)) with **SQLite** on disk. Use this runbook to deploy without surprises.

## SQLite and scaling

- SQLite allows **one writer at a time** on a single file. Run **one container instance** per database file.
- Do **not** point multiple replicas at the same SQLite path on shared storage; you risk locking failures or corruption.
- A future phase can move to Postgres for multi-instance and HA.

## Environment variables

| Variable | Purpose |
|----------|---------|
| `PORT` | HTTP listen port inside the container (the app reads `PORT`; default `8080`). |
| `DATABASE_URL` | e.g. `sqlite:./data/ctf_maze.db` — path must be **on a persistent volume** in production. |
| `RUST_LOG` | Logging level (e.g. `info`). |

Platforms often set `PORT` for you. **If the app fails to bind**, confirm you are not hardcoding `8080` in the platform UI while the process expects another port.

## TLS

- Terminate **TLS at the edge** (managed load balancer, reverse proxy, or the PaaS HTTPS endpoint).
- The container serves plain HTTP; do not rely on TLS inside the image for production.

## CORS and browser origins

- Today the API uses a **permissive** CORS configuration in `src/main.rs`.
- For production hardening, **phase 13** is expected to restrict origins (e.g. `ALLOWED_ORIGINS`). Until then, document which front-end origin will call the API and plan to align CORS with that origin.

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
