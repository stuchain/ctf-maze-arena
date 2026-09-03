# CTF Maze Arena v0.1 design roadmap

**Status:** active

**Last updated:** 2026-09-03

**Current phase:** Phase 5 is ready
**Target:** a polished, public, completely free portfolio experience

## 1. Vision

CTF Maze Arena will become an interactive algorithm laboratory that lets anyone generate deterministic mazes, watch pathfinding algorithms work, compare their behavior, share replays, and compete in daily challenges. It must demonstrate developer fundamentals through correctness, clear architecture, testing, performance awareness, accessibility, security, operations, and exceptional product presentation.

The signature experience is **Algorithm Race**: synchronized BFS, DFS, A*, and key-aware solving on the same maze with understandable live metrics and shareable configuration.

## 2. Binding product decisions

| Decision | v0.1 choice |
|---|---|
| Cost | Completely free; no paid services or custom domain |
| Visual direction | Premium dark algorithm lab with a complete light theme |
| Product name | CTF Maze Arena |
| Access | Anonymous play; GitHub sign-in for persistent identity features |
| Database | PostgreSQL |
| Public URL | Vercel-provided frontend URL |
| Frontend hosting | Vercel Hobby |
| API hosting | Koyeb free instance, containerized Rust/Axum service |
| Database hosting | Neon free Postgres |

Free tiers provide no production SLA. Koyeb may scale the API to zero after one hour without traffic. The product must display a deliberate “Waking the arena” state and retry safely. It must not use artificial keep-alive traffic to evade provider policies.

## 3. Current-state assessment

### Existing strengths

- Rust/Axum backend with deterministic generation and four solvers.
- PostgreSQL persistence, replay storage, WebSocket transport, GitHub OAuth, JWT modes, CORS, rate limiting, request IDs, structured logging, Docker, CI, benchmarks, and Playwright smoke tests.
- 68 Rust tests, 11 frontend unit tests, and 11 browser flows pass; formatting, Clippy, full ESLint, TypeScript, production build, and Playwright flows share enforced quality gates.
- Typed frontend API parsing, fail-fast runtime configuration, secret-safe environment examples, dependency review, and hardened container scanning are established.
- A responsive three-region application shell, semantic two-theme token system, accessible UI primitives, branded metadata, and intentional product states are established.

### Remaining credibility gaps

- Maze walls are drawn between cell centers instead of on boundaries; rendering repeatedly scans arrays per cell.
- Strict cross-platform pixel baselines remain deferred until the browser environment is standardized; Phase 4 captures reviewable screenshots and gates layout geometry at representative viewports.
- Maze-renderer component and protocol-evolution coverage remain incomplete.

## 4. Phase plan

| Phase | Status | Outcome | Depends on |
|---|---|---|---|
| 0. Product and architecture | complete | Binding decisions and quality bar | — |
| 1. Engineering foundations | complete | Honest CI, safe configuration, clean repository | 0 |
| 2. Backend and Postgres | complete | Correct lifecycle, durable schema, trustworthy leaderboard | 1 |
| 3. Realtime engine | complete | Incremental, resumable, bounded solve streaming | 2 |
| 4. Design system | complete | Premium responsive shell and reusable UI primitives | 1 |
| 5. Maze visualization | ready | Correct, fast, accessible maze and replay controls | 3, 4 |
| 6. Algorithm Race | not-started | Signature comparison and education experience | 5 |
| 7. Identity and community | not-started | GitHub-backed profiles, scores, challenges, achievements | 2, 4, 6 |
| 8. Free deployment | not-started | Vercel + Koyeb + Neon public environment | 2, 3, 4 |
| 9. Portfolio launch | not-started | Demo media, README, diagrams, audit, tagged release | 6, 7, 8 |

Phase 4 can begin after Phase 1 while Phases 2–3 progress, but the default implementation sequence is serial to keep review and verification clear.

## 5. Cross-phase quality gates

Every phase must address, where applicable:

- **Correctness:** unit, integration, property, protocol, and E2E evidence.
- **Performance:** measured solver, payload, query, rendering, and bundle budgets.
- **Accessibility:** keyboard, focus, screen reader, contrast, zoom, and reduced motion.
- **Security:** least privilege, safe secrets, validated input, origin/auth boundaries, quotas, and dependency scanning.
- **Operations:** migrations, logs, health, recovery, cleanup, and rollback.
- **Presentation:** responsive polish, precise copy, intentional states, and current documentation.

## 6. Release definition

v0.1 is complete only when:

- The anonymous generate → solve → replay → share flow works publicly.
- GitHub sign-in enables legitimate score submission, personal results, and achievements.
- Algorithm Race is deterministic, smooth, understandable, and shareable.
- PostgreSQL data survives application redeploys and migrations are repeatable.
- WebSocket reconnect, cancellation, fast-completion, and failure paths are tested.
- Full local and CI checks pass without suppressed correctness rules.
- Core flows meet the agreed accessibility and responsive criteria.
- The Vercel URL, architecture story, demo media, and verified setup appear in the root README.

## 7. Global risks

| Risk | Response |
|---|---|
| Free API cold starts | Warm-up handshake, clear UI state, bounded retry, no fake keep-alive |
| Free-tier limits change | Isolate providers behind configuration; document replacement path |
| 0.1 vCPU API capacity | Bound maze sizes/concurrency, use `spawn_blocking`, benchmark on target host |
| Neon scale-to-zero latency | Region alignment, pooled connections, readiness distinction, retry transient startup |
| WebSocket interruption | Sequence IDs, replayable events, exponential reconnect, terminal-state fallback |
| Public abuse | Input caps, per-IP/per-user quotas, concurrency limit, retention cleanup |
| Feature sprawl | Phase exit gates; community features remain subordinate to Algorithm Race |
| Documentation drift | Update design ledger and verification evidence in implementation changes |

## 8. Roadmap change log

| Date | Change |
|---|---|
| 2026-09-02 | Created v0.1 roadmap and analytical phase documents. Recorded free-only hosting, premium dark visual direction, retained product name, GitHub identity, Postgres, and Vercel URL decisions. |
| 2026-09-02 | Completed Phase 1 with canonical local/CI checks, typed frontend boundaries, validated configuration, repository cleanup, dependency automation, and hardened production containers. Advanced Phase 2 to ready. |
| 2026-09-03 | Completed Phase 2 with PostgreSQL persistence, explicit durable run states, bounded blocking execution, restart recovery, stable GitHub ownership, submitted-only rankings, safe API errors, and database/HTTP integration coverage. Advanced Phase 3 to ready. |
| 2026-09-03 | Completed Phase 3 with live solver progress, protocol v1 snapshots and deltas, race-free retained subscriptions, deterministic bounded replays, reconnect fallback, cancellation, graceful shutdown, and unit/Postgres/browser coverage. Advanced Phase 4 to ready. |
| 2026-09-03 | Completed Phase 4 with a premium responsive lab shell, two accessible persistent themes, reusable semantic primitives, branded metadata, intentional states, axe automation, visual viewport evidence, and keyboard/zoom/reduced-motion coverage. Advanced Phase 5 to ready. |
