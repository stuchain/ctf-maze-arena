# E2E testing runbook (Playwright)

This runbook explains how to execute the Phase 16 Playwright smoke tests locally and debug failures.

## Prerequisites

- Rust toolchain (backend)
- Node.js 20+ and npm (frontend)
- Browser binaries installed for Playwright

From `web/`:

```bash
npm ci
npx playwright install
```

## Required environment values

- Backend API URL for web/e2e: `NEXT_PUBLIC_API_URL=http://127.0.0.1:8080`
- Backend port: `PORT=8080` (default if unset)
- Auth rollout mode for smoke tests: `AUTH_MODE=anonymous`

For local shell session running tests:

```bash
export NEXT_PUBLIC_API_URL=http://127.0.0.1:8080
```

On PowerShell:

```powershell
$env:NEXT_PUBLIC_API_URL = "http://127.0.0.1:8080"
```

## Two-terminal workflow (recommended)

Terminal 1 (repo root):

```bash
cargo run
```

Terminal 2 (`web/`):

```bash
npm run dev
```

Terminal 3 (`web/`), run tests:

```bash
npm run test:e2e
```

For CI-like local execution:

```bash
npm run build
npm run start
npm run test:e2e:ci
```

## Optional compose workflow

If you prefer containers, start backend and web with:

```bash
docker compose up --build
```

Then run Playwright from host in `web/`:

```bash
npm run test:e2e
```

## Failure diagnostics and traces

- CI/local retries can emit traces on first retry.
- Open traces with:

```bash
npx playwright show-trace test-results/<trace-folder>/trace.zip
```

- Useful artifacts:
  - `web/test-results/`
  - `web/playwright-report/`

## What the smoke suite covers

- Generate maze and confirm grid rendering.
- Solve maze and confirm terminal stream status reaches `finished`.
- Signed-out token bridge behavior (`GET /api/token`) returns `401`.

## Auth-specific local checks

- To test strict auth routing locally, run backend with:
  - `AUTH_MODE=jwt`
  - `JWT_SECRET=<same secret used by web token route>`
- Verify signed-out `POST /api/solve` returns `401`, then sign in via GitHub and re-run to confirm success with Bearer auth.
