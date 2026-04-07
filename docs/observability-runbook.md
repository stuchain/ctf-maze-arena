# Observability and tracing runbook

This runbook is for day-1 operations on the API without full APM tooling.
Primary correlation key: `X-Request-Id` / `request_id`.

## Quick triage flow

1. Ask reporter for the response header value from `X-Request-Id`.
2. Filter logs for that ID.
3. Check request path, HTTP status, and latency.
4. If `429`, follow the rate-limit section below.
5. If solve behavior looks wrong, verify the same run via `/api/replay/{runId}`.

## Capture and reuse request IDs

- Every HTTP response includes `X-Request-Id`.
- Clients may provide `X-Request-Id`; safe values are propagated.
- Unsafe or missing values are replaced with a generated UUID.

Example request/response:

```bash
curl -i -H "X-Request-Id: support-case-42" http://localhost:8080/api/health
```

Use the returned `X-Request-Id` value when searching logs.

## Log lookup by request ID

### Pretty logs (`LOG_FORMAT=pretty`)

Typical request completion line includes:

- `request completed`
- `status=<code>`
- `latency_ms=<ms>`
- span fields with `method`, `path`, `request_id`

Example grep-like search:

```bash
rg "request_id\":\"support-case-42|request_id=support-case-42" .
```

### JSON logs (`LOG_FORMAT=json`)

Each log line is JSON and includes fields like `timestamp`, `level`, `message`, `status`, `latency_ms`, and span data with `request_id`, `method`, `path`.

Example:

```bash
docker logs <container> | jq -c 'select(.span.request_id == "support-case-42")'
```

If `jq` is unavailable locally, install it or filter raw lines first with `rg`.

## Normal solve completion signals

For a healthy solve flow:

1. `POST /api/solve` returns `200` with a `runId`.
2. Request completion log shows `path=/api/solve` and `status=200`.
3. WebSocket stream for `/api/solve/stream?runId=...` ends with a `{"type":"finished", ...}` message.
4. `GET /api/replay/{runId}` returns persisted replay data.

Useful checks:

```bash
curl -s -X POST http://localhost:8080/api/solve -H "content-type: application/json" -d "{\"mazeId\":\"<maze-id>\",\"solver\":\"ASTAR\"}"
curl -s "http://localhost:8080/api/replay/<run-id>"
```

## 429 rate-limit interpretation

`429 Too Many Requests` means the per-IP limiter rejected the request.

- Baseline limits apply to general routes.
- Stricter limits apply to expensive routes:
  - `POST /api/solve`
  - `POST /api/maze/generate`

When investigating `429`:

1. Confirm request path and frequency.
2. Check current limiter env:
   - `RATE_LIMIT_PER_SECOND`
   - `RATE_LIMIT_BURST`
   - `RATE_LIMIT_EXPENSIVE_PER_SECOND`
   - `RATE_LIMIT_EXPENSIVE_BURST`
3. If behind a proxy, verify `TRUST_PROXY` is correct for your platform.
4. Distinguish abuse from legitimate bursts before changing limits.

Related deployment guidance: `docs/deployment-runbook.md`.
