# Realtime protocol v1

**Status:** implemented

**Transport:** one solve run per WebSocket at `/api/solve/stream`

## Envelope and ordering

Every server message contains:

- `type`: message discriminator.
- `protocolVersion`: always `1` for this schema.
- `runId`: UUID of the subscribed run.
- `sequence`: monotonic run sequence.

State-changing messages use strictly increasing sequence numbers. `connected` reports the accepted resume position and `latestSequence`; `heartbeat` repeats the latest published sequence without changing state. Clients ignore messages for another protocol version or run and do not reapply a state-changing sequence they have already accepted.

## Client subscription

The client supplies `runId` and optionally `afterSequence`, the last state-changing sequence it applied. The server subscribes the socket to live publication before reading retained history, closing the fast-completion race.

If the bounded retained history covers the requested gap, the server sends the missing messages. Otherwise it sends a full `snapshot` at the sequence represented by that state, followed by a retained terminal message when applicable. A missing in-memory stream produces `failed` with code `stream_expired`; the client then reads durable run state and, for completed runs, the persisted replay.

## Messages

### `connected`

```json
{"type":"connected","protocolVersion":1,"runId":"uuid","sequence":12,"latestSequence":18}
```

### `snapshot`

`state` contains `step`, complete `frontier` and `visited` cell arrays, and optional `current`.

```json
{"type":"snapshot","protocolVersion":1,"runId":"uuid","sequence":13,"state":{"step":25,"frontier":[[2,3]],"visited":[[0,0],[1,0]],"current":[1,0]}}
```

### `delta`

`delta` contains the new `step`, `frontierAdded`, `frontierRemoved`, `visitedAdded`, and optional `current`. Cell collections are canonicalized to stable coordinate order.

```json
{"type":"delta","protocolVersion":1,"runId":"uuid","sequence":14,"delta":{"step":27,"frontierAdded":[[3,3]],"frontierRemoved":[[2,3]],"visitedAdded":[[2,3]],"current":[2,3]}}
```

### Terminal messages

- `completed` includes the final `path` and authoritative `stats` (`visited`, `cost`, `ms`).
- `failed` includes a stable `code` and user-safe `message`.
- `cancelled` acknowledges explicit cancellation or shutdown interruption.

Only the first terminal publication wins. Terminal state remains subscribable for the configured retention window.

### `heartbeat`

Heartbeats provide transport liveness and do not advance or mutate solve state.

## Bounds and defaults

| Setting | Default | Purpose |
|---|---:|---|
| `MAX_CONCURRENT_SOLVES` | 1 | Global CPU-bound solve limit |
| `MAX_ACTIVE_SOLVES_PER_ACTOR` | 2 | Queued/running limit per GitHub identity or source IP |
| `STREAM_HISTORY_CAPACITY` | 256 | Retained messages per active run |
| `STREAM_CLIENT_CAPACITY` | 32 | Per-subscriber broadcast buffer |
| `STREAM_SAMPLE_EVERY` | 2 | Solver steps per published progress sample |
| `STREAM_SNAPSHOT_EVERY` | 32 | Solver steps between canonical snapshots |
| `MAX_REPLAY_EVENTS` | 2048 | Persisted replay event cap |
| `STREAM_RETENTION_SECS` | 30 | In-memory terminal retention |
| `STREAM_HEARTBEAT_SECS` | 10 | WebSocket heartbeat interval |

Persisted replay payloads are capped at 8 MiB and expire after seven days. Startup cleanup removes expired rows.

## Client state machine

The browser exposes `waking`, `connecting`, `live`, `reconnecting`, `completed`, `failed`, and `cancelled`. It retains only the latest live visual state, reconnects with capped exponential backoff and jitter, and resumes from its last applied sequence. Replay pages reconstruct the same snapshot/delta model deterministically.
