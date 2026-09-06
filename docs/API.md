# API Reference

Base URL: `http://localhost:8080` (or set `NEXT_PUBLIC_API_URL` in the Next.js app).

All REST routes below are under `/api`.

## Authentication model

- The web app signs users in with GitHub via NextAuth.
- `GET /api/token` (web route) mints short-lived API JWTs (10 minute TTL) from the authenticated web session.
- API JWT middleware accepts only `HS256` and validates `sub`, `exp`, `iat`, `iss`, and `aud` with `JWT_CLOCK_SKEW_SECS` tolerance. Issuer and audience default to `ctf-maze-web` and `ctf-maze-api` and can be set with `JWT_ISSUER` / `JWT_AUDIENCE` in both services.
- `AUTH_MODE` controls enforcement:
  - `anonymous`: no JWT required.
  - `optional_jwt`: JWT accepted when present.
  - `jwt`: JWT required on protected identity routes (`POST /api/leaderboard`, profile/export/deletion). Anonymous solves remain available.

## GET /api/health

Returns build-aware JSON:

```json
{
  "status": "ok",
  "version": "0.1.0",
  "gitSha": "a1b2c3d4e5f6"
}
```

This is process liveness and does not query dependencies. `GET /api/ready` checks PostgreSQL and returns `200` only when the API can serve database-backed traffic.

## POST /api/maze/generate

Generate a new maze and persist it.

Returns `201 Created`.

**Request body** (`w` / `h` / `seed` / `algo` are lowercase keys as in JSON):

```json
{
  "w": 10,
  "h": 10,
  "seed": 42,
  "algo": "KRUSKAL",
  "featurePreset": "classic",
  "dailyChallengeId": "optional-versioned-challenge-uuid"
}
```

`algo` is one of `KRUSKAL`, `PRIM`, `DFS`. `featurePreset` is optional and is either `classic` (default) or `keys`; the latter deterministically places one key before a locked passage on the generated solution route.

**Response** (camelCase):

```json
{
  "mazeId": "uuid-string",
  "maze": {
    "grid": { "width": 10, "height": 10 },
    "walls": { "inner": [ /* edges as pairs of cells */ ] },
    "start": { "x": 0, "y": 0 },
    "goal": { "x": 9, "y": 9 },
    "keys": {},
    "doors": {}
  }
}
```

The exact `walls.inner` edge shape matches Rust serialization (typically arrays of two cell objects or tuples). The Next.js client maps this in `web/lib/maze.ts`.

## GET /api/maze/:mazeId

Returns the same maze JSON object as in `generate`’s `maze` field, for a stored maze id.

## POST /api/solve

Start an asynchronous solve. Responds immediately with a `runId`; progress and results are delivered over the WebSocket (see below).

Returns `202 Accepted`. The run is first stored as `queued`, then transitions to `running` and a terminal state.

**Request** (camelCase keys):

```json
{
  "mazeId": "...",
  "solver": "ASTAR"
}
```

`solver` is one of `BFS`, `DFS`, `ASTAR`, `DP_KEYS`.

**Response:**

```json
{
  "runId": "uuid-string"
}
```

## POST /api/race

Atomically validates a bounded set of two to four unique solvers, clones the same stored maze for every competitor, and starts their runs under one per-actor race lease. Every run uses the global compute semaphore; the default deployment therefore measures competitors sequentially while the client presents synchronized logical playback.

```json
{ "mazeId": "...", "solvers": ["BFS", "DFS", "ASTAR"] }
```

Returns `202 Accepted`:

```json
{
  "raceId": "...",
  "executionMode": "sequential_compute_synchronized_playback",
  "runs": [
    { "solver": "BFS", "runId": "..." },
    { "solver": "DFS", "runId": "..." },
    { "solver": "ASTAR", "runId": "..." }
  ]
}
```

Each run uses the normal stream, replay, status, and cancellation APIs.

## POST /api/run/:runId/cancel

Explicitly cancel a queued or running solve. Anonymous runs can be cancelled anonymously; authenticated runs require the same GitHub identity that created them.

```json
{ "cancelled": true }
```

The operation is idempotent while the retained run is already cancelled. Completed or failed runs reject the invalid state transition.

## WebSocket GET /api/solve/stream?runId=...&afterSequence=...

Connect with `runId` from the solve response. On reconnect, provide the last applied `sequence` as `afterSequence`. Every protocol v1 message contains `type`, `protocolVersion`, `runId`, and a monotonic `sequence`.

**First message** (text JSON):

```json
{
  "type": "connected",
  "protocolVersion": 1,
  "runId": "...",
  "sequence": 0,
  "latestSequence": 14
}
```

**Complete state:**

```json
{
  "type": "snapshot",
  "protocolVersion": 1,
  "runId": "...",
  "sequence": 1,
  "state": {
    "step": 1,
    "frontier": [[0, 0]],
    "visited": [[0, 0]],
    "current": [0, 0]
  }
}
```

**Incremental state:**

```json
{
  "type": "delta",
  "protocolVersion": 1,
  "runId": "...",
  "sequence": 2,
  "delta": {
    "step": 2,
    "frontierAdded": [[1, 0]],
    "frontierRemoved": [[0, 0]],
    "visitedAdded": [[1, 0]],
    "current": [1, 0]
  }
}
```

`current` may be omitted. If retained deltas no longer cover `afterSequence`, the server sends a current snapshot. Heartbeats repeat the latest sequence and do not modify visual state.

**Final success:**

```json
{
  "type": "completed",
  "protocolVersion": 1,
  "runId": "...",
  "sequence": 15,
  "path": [[0, 0], [1, 0]],
  "stats": { "visited": 42, "cost": 10, "ms": 1, "peakFrontier": 8 }
}
```

Terminal failure and cancellation messages use `failed` and `cancelled`. A `failed` message contains stable `code` and safe `message` fields. `stream_expired` directs the client to `GET /api/run/:runId` and the persisted replay.

The full schema and resume rules are documented in [Realtime protocol v1](v0.1/realtime-protocol-v1.md).

## GET /api/replay/:runId

Returns the versioned replay JSON (camelCase): `protocolVersion`, `mazeId`, `solver`, `seed`, `events`, `path`, and `stats`. Events are protocol-compatible `snapshot` and `delta` records. Expired replays return `404`.

## GET /api/run/:runId

Returns durable run status, timestamps, safe failure code, and server-computed metrics when complete.

## GET /api/leaderboard

One selector is required: `mazeId`, `dailyDate=YYYY-MM-DD`, or `raceId`. Optional filters are `solver`, `scope=all|personal`, `limit` = 1–100 (default 50), and `offset` = 0–10000. Personal scope requires a valid Bearer token.

**Response:** JSON array of entries:

```json
[
  {
    "rank": 1,
    "tied": false,
    "runId": "...",
    "solver": "ASTAR",
    "cost": 10,
    "ms": 2,
    "visited": 50,
    "displayName": "octocat",
    "avatarUrl": "https://avatars.githubusercontent.com/...",
    "acceptedAt": "2026-09-05T12:00:00Z",
    "isPersonal": true
  }
]
```

Only explicitly accepted submissions are returned. Ordering is stable: cost, time, visited, acceptance time, then run ID. Rank ties are based on equal cost, runtime, and visited metrics. Submissions are limited to 100 per identity in a rolling 24-hour window; duplicates remain idempotent.

## POST /api/leaderboard

Submit a completed run to the leaderboard pipeline.

This endpoint requires `Authorization: Bearer <token>` whenever authentication is enabled. The completed run must already be owned by the same stable GitHub identity.

**Request**:

```json
{
  "runId": "uuid-string"
}
```

**Response**:

```json
{
  "accepted": true,
  "duplicate": false
}
```

The first valid submission returns `201 Created`; an idempotent duplicate returns `200 OK` with `duplicate: true`.

## Error envelope

REST failures use a stable, safe shape and never expose database details:

```json
{
  "error": { "code": "invalid_request", "message": "..." },
  "requestId": "..."
}
```

## GET /api/daily

Returns the UTC daily challenge parameters (camelCase):

```json
{
  "challengeId": "uuid-string",
  "seed": 1234567890,
  "date": "2026-04-04",
  "version": 1,
  "w": 15,
  "h": 15,
  "algo": "KRUSKAL",
  "featurePreset": "classic",
  "secondsUntilReset": 43199,
  "personalBest": null,
  "streak": 0,
  "completed": false
}
```

The definition is stored by UTC date and version. Passing its `challengeId` back to maze generation binds the maze only when every immutable parameter matches. With optional authentication, the response includes the caller's accepted personal best, completion state, and consecutive UTC-day streak.

## Profile, export, and deletion

- `GET /api/profile` upserts the signed-in GitHub identity and returns public profile fields, submission count, streak, versioned server achievements, and the 50 most recent accepted runs.
- `GET /api/profile/export` returns the same portable JSON record for download.
- `DELETE /api/profile` removes achievements and private run ownership, clears the public name/avatar, and rotates the provider subject. Accepted leaderboard rows remain attached only to an anonymized `Deleted player` record so historical ranking integrity is preserved.

All three routes require a valid Bearer token. The GitHub provider requests only `read:user`; anonymous play, solving, replays, and non-personal leaderboards remain available without sign-in.
