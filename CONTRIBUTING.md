# Contributing to CTF Maze Arena

## Design-led workflow

The active roadmap lives in [`docs/v0.1`](docs/v0.1/README.md). Before changing a planned area:

1. Read the roadmap and the applicable phase document.
2. Keep the phase status and checklist current in the same change as the implementation.
3. Record material design deviations before or alongside the code.
4. Add verification evidence and do not mark a phase complete until its exit criteria pass.

## Local setup

1. Install stable Rust, Node.js 20 or newer, npm, and PowerShell 7.
2. Copy `.env.example` to `.env` and `web/.env.example` to `web/.env.local`.
3. Run `npm ci` in `web`.
4. Start the API with `cargo run` and the frontend with `npm run dev` from `web`.

Local environment files are ignored. Never commit credentials, OAuth secrets, JWT secrets, database connection strings, or provider tokens.

## Verification

Run every non-E2E check from the repository root:

```powershell
./scripts/verify.ps1
```

Use `-Scope rust` or `-Scope web` for a targeted run. Run Playwright after starting the API and production frontend as described in `docs/e2e-runbook.md`:

```powershell
Set-Location web
npm run test:e2e
```

## Commits

Use one-line Conventional Commit messages, such as:

- `feat(ui): add replay speed controls`
- `fix(api): reject duplicate leaderboard submissions`
- `docs: update realtime protocol design`

Keep each commit focused and do not include generated build output.
