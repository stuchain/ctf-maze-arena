# CTF Maze Arena web

The Next.js frontend for CTF Maze Arena. It generates and displays mazes, streams solver progress, shows replays and results, and provides GitHub-backed identity features when authentication is enabled.

## Setup

```powershell
Copy-Item .env.example .env.local
npm ci
npm run dev
```

The local application runs at <http://localhost:3000> and expects the Rust API at `NEXT_PUBLIC_API_URL`.

`NEXT_PUBLIC_API_URL` and `NEXT_PUBLIC_AUTH_MODE` are public values embedded in the browser bundle at build time. `AUTH_MODE`, GitHub OAuth credentials, `NEXTAUTH_SECRET`, and `JWT_SECRET` are server-only runtime values. Anonymous mode needs no secrets; `jwt` and `optional_jwt` modes fail fast unless every server-only authentication value in `.env.example` is configured.

## Commands

- `npm run lint` — ESLint with warnings treated as failures.
- `npm run typecheck` — strict TypeScript validation.
- `npm run test:unit` — Vitest unit tests.
- `npm run build` — production Next.js build.
- `npm run check` — all four checks above.
- `npm run test:e2e` — Playwright end-to-end suite.

The active design roadmap is in [`../docs/v0.1`](../docs/v0.1/README.md).
