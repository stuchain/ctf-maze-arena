# Phase 8 — Free deployment and operations

**Status:** not-started

**Depends on:** Phases 2, 3, and 4

**Purpose:** publish a reliable-as-practical zero-cost demo with honest free-tier behavior

## Problem

The repository has Docker and deployment notes but no public environment. SQLite is incompatible with ephemeral free compute, OAuth needs production callbacks, WebSockets need reconnect behavior, and free services introduce cold starts and quotas.

## Goals

- Deploy the frontend at a Vercel-provided URL for $0.
- Deploy the Rust container on Koyeb’s free instance for $0.
- Use Neon free Postgres for durable data.
- Make cold starts, reconnects, migrations, secrets, health, and rollback operationally explicit.
- Keep provider configuration reproducible and replaceable.

## Non-goals

- No paid uptime SLA, custom domain, multi-region, horizontal scaling, or always-on guarantee.
- No artificial pinging to bypass scale-to-zero.
- No production claim beyond the documented free-tier limits.

## Topology

- **Vercel Hobby:** Next.js frontend, NextAuth routes, preview deployments, TLS, public `vercel.app` URL.
- **Koyeb free instance:** one Rust/Axum container, 512 MB RAM, 0.1 vCPU, one region, scale-to-zero.
- **Neon free:** managed Postgres, pooled TLS connection string, scale-to-zero behavior.
- **GitHub Actions:** repository checks; provider-native Git integration performs deployment.

Prefer a European region pairing, subject to free-tier availability, to reduce API/database latency for the project owner and likely audience. Confirm actual region choices during implementation.

## Cold-start experience

On application load, the frontend performs a bounded API readiness request. If the service is sleeping, show a branded “Waking the arena” state with a concise explanation, elapsed feedback, and retry. Configuration/help content remains usable. Once healthy, queued user intent may continue exactly once. Fail after a documented timeout with manual retry and status detail. WebSocket open alone is not the only wake-up path.

## Health and readiness

- Liveness: process is running; no database dependency.
- Readiness: database reachable, migrations compatible, solver capacity available.
- Health responses expose version/commit but no secrets or infrastructure detail.
- Frontend displays a safe request ID for support-worthy failures.

## Configuration and security

- Vercel: GitHub OAuth credentials, NextAuth secret/URL, JWT secret, public API URL.
- Koyeb: Postgres URL, JWT secret, auth mode, exact Vercel production/preview origin policy, rate/concurrency settings, logging.
- GitHub OAuth callbacks use the Vercel production URL. Preview authentication is either deliberately disabled or uses a controlled callback strategy; never wildcard unsafe origins.
- Secrets live only in provider secret stores and local ignored env files.

## Database operations

Run forward-only migrations as a controlled pre-deploy/release step with advisory locking or SQLx migration locking. Backups are constrained by the free plan; document Neon restore/time-travel capability actually available at launch and provide a periodic logical export procedure if feasible without secrets in CI artifacts.

## Resource policy

- Conservative maze dimension and active-solve limits.
- Bounded replay size and retention cleanup.
- Connection pool sized for Koyeb and Neon free limits.
- No paid overage: free services should pause/reject rather than generate cost.
- Document provider quotas and a replacement/migration path.

## Work checklist

- [ ] Add reproducible Koyeb/Vercel configuration and deployment documentation.
- [ ] Provision Neon, migrate schema, and verify pooled TLS connectivity.
- [ ] Deploy Rust container and verify HTTP, WebSocket, shutdown, and cold start.
- [ ] Deploy Next.js and configure API URL, CORS, JWT, and GitHub callbacks.
- [ ] Implement API wake-up UX and retry behavior.
- [ ] Configure liveness/readiness, structured logs, version metadata, and error correlation.
- [ ] Verify migrations, rollback/redeploy, retention cleanup, and recovery procedure.
- [ ] Record free-tier quotas and launch-time provider settings.

## Test and launch strategy

- Production smoke: health, generate, live solve, reconnect, replay, GitHub sign-in, submit, leaderboard, share URL.
- Cold-start smoke after confirmed scale-to-zero.
- Verify persistence across API redeploy and frontend promotion.
- Verify exact CORS and WebSocket origins for production and rejected origins.
- Run constrained load test below provider abuse thresholds.
- Exercise rollback and database migration compatibility before linking publicly.

## Risks

- Provider free tiers can change or disappear. Keep standard Docker, Postgres, and environment interfaces; document replacements.
- Koyeb free CPU may make large races slow. Enforce measured limits and degrade visualization detail before correctness.
- Vercel preview URLs vary, complicating OAuth/CORS. Production auth is the release requirement; previews may run anonymous-only.
- No SLA means occasional downtime is expected. Communicate honestly and keep the repository/demo media valuable when services are unavailable.

## Exit criteria

- [ ] Vercel public URL serves the polished application over TLS.
- [ ] Koyeb API and Neon Postgres operate at zero cost within documented limits.
- [ ] Cold-start, reconnect, OAuth, CORS, persistence, and redeploy are verified publicly.
- [ ] No secrets exist in source, logs, client bundles, or CI artifacts.
- [ ] Recovery, migration, rollback, quotas, and provider replacement are documented.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| — | Not implemented | — |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-02 | Use Vercel Hobby + Koyeb free + Neon free. | UI must design for scale-to-zero; architecture has no availability SLA. |
| 2026-09-02 | Use provider URL rather than a custom domain. | OAuth, metadata, CORS, and README use the stable Vercel production URL. |

## Provider references

- <https://vercel.com/docs/plans>
- <https://www.koyeb.com/docs/reference/instances>
- <https://www.koyeb.com/docs/run-and-scale/scale-to-zero>
- <https://www.koyeb.com/docs/deploy/rust>
- <https://neon.com/pricing>
