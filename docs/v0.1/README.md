# CTF Maze Arena v0.1 design package

This directory is the source of truth for the v0.1 improvement program. It is intentionally versioned separately from the historical implementation guides in `docs/commit/`.

## Documents

- [Design roadmap](DESIGN_ROADMAP.md)
- [Phase 0 — Product and architecture](phase-00-product-and-architecture.md)
- [Phase 1 — Engineering foundations](phase-01-engineering-foundations.md)
- [Phase 2 — Backend correctness and Postgres](phase-02-backend-and-postgres.md)
- [Phase 3 — Realtime solve engine](phase-03-realtime-engine.md)
- [Realtime protocol v1](realtime-protocol-v1.md)
- [Phase 4 — Design system and application shell](phase-04-design-system.md)
- [Phase 5 — Maze visualization and replay](phase-05-maze-visualization.md)
- [Phase 6 — Algorithm Race](phase-06-algorithm-race.md)
- [Phase 7 — Identity and community](phase-07-identity-and-community.md)
- [Phase 8 — Free deployment and operations](phase-08-free-deployment.md)
- [Phase 9 — Portfolio launch](phase-09-portfolio-launch.md)

## Maintenance protocol

These are living design documents. During implementation:

1. Update `DESIGN_ROADMAP.md` when a phase changes status.
2. Check off completed work in the active phase document in the same change that implements it.
3. Add verification evidence, including commands and relevant manual checks.
4. Record material design changes under **Decision and deviation log** before or alongside the code change.
5. Update risks when evidence changes their likelihood or impact.
6. Do not mark a phase complete until every exit criterion is met or explicitly waived with a reason.
7. Keep future phases aligned when an earlier decision changes their assumptions.

Allowed statuses are `not-started`, `in-design`, `ready`, `in-progress`, `blocked`, and `complete`.

## Current position

- Design package: `complete`
- Phase 0: `complete`
- Phase 1 — Engineering foundations: `complete`
- Phase 2 — Backend correctness and Postgres: `complete`
- Phase 3 — Realtime solve engine: `complete`
- Current phase: Phase 4 — Design system and application shell (`ready`)
- Product implementation under this roadmap: active
