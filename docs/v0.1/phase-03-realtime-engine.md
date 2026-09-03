# Phase 3 — Realtime solve engine

**Status:** complete

**Depends on:** Phase 2

**Purpose:** replace replay-like frame bursts with resilient incremental solve streaming

## Problem

Solvers currently collect all frames, finish, then broadcast them. The browser may subscribe after a fast run has already been removed. Full visited/frontier arrays are repeatedly transmitted and stored, and errors or disconnects do not form a resumable protocol.

## Goals

- Stream progress as exploration occurs.
- Remove the solve/subscription race.
- Bound memory, CPU, event rate, and client lag.
- Recover from short WebSocket interruptions using persisted or retained state.
- Support cancellation and deterministic replay.

## Non-goals

- No indefinite collaboration socket.
- No Redis/pub-sub or multi-instance coordination on the free single-instance API.
- No guarantee that every internal solver expansion becomes a client frame.

## Solver interface

Evolve solvers from “return result plus all frames” to a callback/channel-based progress sink. The solver emits semantic deltas at an internal cadence; a recorder constructs the canonical replay while a broadcaster samples client frames. Solver correctness must not depend on an attached client.

## Protocol

All messages include `protocolVersion`, `runId`, and a monotonic `sequence`.

- `connected`: accepted subscription and latest available sequence.
- `snapshot`: complete visual state used initially or after a gap.
- `delta`: cells added/removed from frontier, visited additions, current cell, logical step.
- `completed`: final path, authoritative metrics, terminal sequence.
- `failed`: stable error code and safe message.
- `cancelled`: terminal cancellation acknowledgment.
- `heartbeat`: liveness without changing solve state.

Clients provide `afterSequence` when reconnecting. If retained deltas cover the gap, replay them; otherwise send a snapshot. Persisted replay remains the terminal fallback.

## Backpressure and lifecycle

- Use bounded channels; a slow visual client may skip intermediate frames but must receive a later snapshot and terminal event.
- Limit active solves globally and per IP/user.
- Retain terminal channel state for a short configured period so fast runs remain subscribable.
- Cancel only on an explicit request or policy; a browser disconnect does not invalidate a run.
- On graceful shutdown, reject new solves, mark interrupted work appropriately, and close sockets with a reconnectable reason.

## Replay format

Version the replay schema. Prefer periodic snapshots plus compact deltas over repeated full collections. Decimation is deterministic so shared replays behave consistently. Add retention limits by age and payload size.

## Frontend client behavior

Use a reducer/state machine rather than independent effect-driven state setters. Reconnect with capped exponential backoff and jitter, request from the last applied sequence, and fall back to the replay endpoint after confirmed completion. Surface `waking`, `connecting`, `live`, `reconnecting`, `completed`, `failed`, and `cancelled` distinctly.

## Work checklist

- [x] Define and document protocol v1 schemas.
- [x] Refactor solver progress emission and replay recording.
- [x] Add bounded solve concurrency, channels, sampling, retention, and cancellation.
- [x] Implement sequence-based subscribe/resume and snapshots.
- [x] Implement client reducer, reconnect, terminal fallback, and cleanup.
- [x] Add heartbeat and graceful shutdown behavior.
- [x] Measure payload and memory growth on target maze sizes.

## Test strategy

- Protocol serialization and reducer unit tests.
- Integration tests for subscribe-before-start, subscribe-after-fast-completion, lag, reconnect, cancellation, failure, and shutdown.
- Property check that replay reconstruction equals the final solver state.
- Load test concurrent target-sized solves within free-host constraints.
- E2E test observes intermediate progress, not only the final status.

## Risks

- Excessive event detail can overwhelm the 0.1 vCPU host and browser. Sampling and deltas are requirements, not optional optimization.
- Stateful in-memory channels disappear on redeploy. Durable run state and replay fallback preserve correctness.
- Resume complexity can grow. Protocol v1 supports one run per socket and a simple monotonic sequence only.

## Exit criteria

- [x] Fast solves cannot outrun subscription.
- [x] Intermediate progress is visibly and testably live.
- [x] Reconnect reconstructs a correct state without duplicate application.
- [x] Slow clients cannot cause unbounded memory growth.
- [x] Cancellation, failure, and shutdown reach durable terminal states.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| 2026-09-03 | Protocol v1, incremental solver progress, snapshot/delta replay, bounded retention, reconnect, heartbeat, cancellation, and shutdown | `cargo test --all-targets`; real Postgres integration suite with `TEST_DATABASE_URL`; `npm run check`; `npm run test:e2e` (3 browser flows) |
| 2026-09-03 | Concurrent target-size budget verification | Four 50×50 solves reconstruct their final visual states while remaining within 256 retained messages, 2,048 replay events, and the 8 MiB payload ceiling. |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-03 | Retain terminal streams for 30 seconds, then use durable run state and persisted replay as fallback. | Fast solves remain subscribable without making in-memory state a durability dependency. |
| 2026-09-03 | Persist replay schema v1 as periodic snapshots plus deltas, capped at 2,048 events and 8 MiB with seven-day expiry. | Replay storage and browser state are bounded and deterministic. |
| 2026-09-03 | Treat explicit cancellation as terminal and durably cancel active work during graceful shutdown. | Disconnecting a browser does not cancel work; process shutdown cannot leave active runs indefinitely. |
