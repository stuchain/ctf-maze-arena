# API Reference

Base URL: `http://localhost:8080` (or set `NEXT_PUBLIC_API_URL` in the Next.js app).

All REST routes below are under `/api`.

## Authentication model

- The web app signs users in with GitHub via NextAuth.
- `GET /api/token` (web route) mints short-lived API JWTs (10 minute TTL) from the authenticated web session.
- API JWT middleware validates `HS256` signature, `exp`, and `iat` with `JWT_CLOCK_SKEW_SECS` tolerance.
- `AUTH_MODE` controls enforcement:
  - `anonymous`: no JWT required.
  - `optional_jwt`: JWT accepted when present.
  - `jwt`: JWT required on protected identity routes (`POST /api/leaderboard`). Anonymous solves remain available.

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
  "algo": "KRUSKAL"
}
```

`algo` is one of `KRUSKAL`, `PRIM`, `DFS` (see backend validation).

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

## WebSocket GET /api/solve/stream?runId=...

Connect with query parameter `runId` matching the solve response.

**First message** (text JSON):

```json
{ "type": "connected", "runId": "..." }
```

**Then** (zero or more):

```json
{
  "type": "frame",
  "data": {
    "t": 0,
    "frontier": [[0, 0]],
    "visited": [[0, 0]],
    "current": [0, 0]
  }
}
```

(`current` may be omitted.)

**Final success:**

```json
{
  "type": "finished",
  "path": [[0, 0], [1, 0]],
  "stats": { "visited": 42, "cost": 10, "ms": 1 }
}
```

**Error** (e.g. unknown run):

```json
{ "type": "error", "error": "unknown or completed runId" }
```

## GET /api/replay/:runId

Returns stored replay JSON (camelCase): `mazeId`, `solver`, `seed`, `frames`, `path`, `stats`.

## GET /api/run/:runId

Returns durable run status, timestamps, safe failure code, and server-computed metrics when complete.

## GET /api/leaderboard?mazeId=...

Query: `mazeId` (camelCase) = maze UUID, `limit` = 1–100 (default 50), and `offset` = 0–10000.

**Response:** JSON array of entries:

```json
[
  {
    "runId": "...",
    "solver": "ASTAR",
    "cost": 10,
    "ms": 2,
    "visited": 50,
    "displayName": "octocat",
    "avatarUrl": "https://avatars.githubusercontent.com/..."
  }
]
```

Only explicitly accepted submissions are returned. Ordering is stable: cost, time, visited, acceptance time, then run ID.

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
  "seed": 1234567890,
  "date": "2026-04-04",
  "w": 15,
  "h": 15
}
```

The `seed` is derived from the date string; same calendar day (UTC) yields the same seed.
