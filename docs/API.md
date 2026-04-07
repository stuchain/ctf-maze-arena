# API Reference

Base URL: `http://localhost:8080` (or set `NEXT_PUBLIC_API_URL` in the Next.js app).

All REST routes below are under `/api`.

## GET /api/health

Returns build-aware JSON:

```json
{
  "status": "ok",
  "version": "0.1.0",
  "gitSha": "a1b2c3d4e5f6"
}
```

## POST /api/maze/generate

Generate a new maze and persist it.

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

## GET /api/leaderboard?mazeId=...

Query: `mazeId` (camelCase) = maze uuid.

**Response:** JSON array of entries:

```json
[
  {
    "runId": "...",
    "solver": "ASTAR",
    "cost": 10,
    "ms": 2,
    "visited": 50
  }
]
```

Sorted by cost, then time, then visited (best first), capped at 50.

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
