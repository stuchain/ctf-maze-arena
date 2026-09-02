# Phase 1 — Engineering foundations

**Status:** complete

**Depends on:** Phase 0

**Purpose:** make repository quality gates honest before architectural or visual expansion

## Problem

The backend checks are healthy, but full frontend lint fails and CI passes by disabling the failing rules. A tracked `.env`, default metadata, placeholder code, obsolete phase comments, and raw internal IDs weaken trust. Later phases should not build on hidden quality debt.

## Goals

- One canonical local/CI verification path with no correctness-rule exceptions.
- Safe configuration and secret hygiene.
- Clean, purposeful repository and browser metadata.
- Typed frontend API boundary foundations.
- A documented contribution and change process.

## Non-goals

- No product redesign.
- No database migration or API restructuring.
- No feature additions beyond developer-facing diagnostics required by this phase.

## Design

### Quality commands

Create canonical scripts for Rust formatting, Clippy, tests, frontend lint, typecheck, production build, and E2E. CI must call the same commands developers call locally. Accessibility rules may be added, but existing lint failures may not be hidden with command-line overrides.

### Configuration

- Remove `.env` from tracking and ignore `.env*` except committed examples.
- Keep root and web environment examples explicit and secret-free.
- Validate required production variables at startup with actionable errors.
- Document which variables are build-time public values and which are runtime secrets.
- Never log secrets, tokens, connection strings, or OAuth credentials.

### Frontend boundary

Replace `any`, unchecked JSON casts, and placeholder Zod usage with shared request/response schemas and a small typed API client. Transport errors should normalize into a stable application error type carrying a safe message, status, and request ID when available.

### Repository polish

Replace create-next-app metadata and README, remove obsolete comments and raw UUID display, establish consistent naming, and add contribution guidance. Keep historical docs, but make `docs/v0.1` the active design source.

## Work checklist

- [x] Stop tracking `.env`; update ignore rules and secret guidance.
- [x] Fix all `npm run lint` errors without suppressing rules.
- [x] Make CI run canonical lint, typecheck, build, Rust checks, and tests.
- [x] Replace placeholder `web/lib/api.ts` with typed API/error infrastructure.
- [x] Replace scaffold metadata, frontend README, obsolete comments, and debug IDs.
- [x] Add runtime environment validation for backend and Next.js server routes.
- [x] Add dependency/container scanning with a zero-cost GitHub-native approach.
- [x] Add `CONTRIBUTING.md` and document the design-led phase workflow.

## Test strategy

- Existing tests plus configuration validation tests remain green (62 Rust tests total).
- Full ESLint, TypeScript, and production build pass.
- Add unit tests for environment parsing and frontend response/error parsing.
- Confirm a fresh clone can start from documented example files.
- Run a repository secret scan and inspect tracked environment files.

## Risks

- Tightening lint may expose architectural React-effect issues rather than mechanical fixes. Refactor state ownership instead of disabling rules.
- Environment cleanup can break local startup. Preserve safe defaults only where behavior is unambiguous.
- Schema introduction can duplicate backend types. Keep the initial boundary small; generated contracts are considered later.

## Exit criteria

- [x] All canonical checks pass locally and CI uses the same commands without rule suppression.
- [x] No secret-bearing environment file is tracked.
- [x] No scaffolding metadata, placeholder API code, obsolete phase comment, or raw-ID product copy remains.
- [x] Configuration failures are explicit and documented.
- [x] Implementation evidence and any deviations are recorded below.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| 2026-09-02 | Canonical verification | `./scripts/verify.ps1 -Scope all`: formatting, Clippy, 62 Rust tests, ESLint, TypeScript, 8 frontend unit tests, and production build passed. |
| 2026-09-02 | Browser and container verification | Both production images built; all 3 Playwright flows passed against those containers. |
| 2026-09-02 | Security and hygiene | `npm audit` and configured `cargo audit` passed; Trivy reported 0 high/critical findings in both production images; `.env` is ignored and absent from the tracked-file set. |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-02 | Added Vitest for focused boundary tests and made the PowerShell verification script the shared local/CI entry point. | Frontend parsing and environment behavior now fail before deployment, while CI cannot silently diverge from local checks. |
| 2026-09-02 | Upgraded Next.js, NextAuth, SQLx, Criterion, and vulnerable transitive packages during the security pass. | Audited versions are locked; Criterion benchmarks use `std::hint::black_box`; the API image now pins Rust 1.94 to match dependency MSRV. |
| 2026-09-02 | Replaced SQLx's compile-time migration macro with the runtime migrator. | The release build no longer needs macro code generation, while startup still applies versioned migrations. |
| 2026-09-02 | Accepted `RUSTSEC-2023-0071` only for SQLx's disabled MySQL backend. | The vulnerable RSA package is recorded in the lockfile but absent from every compiled target; `.cargo/audit.toml` documents the narrow exception because no fixed release exists. |
| 2026-09-02 | Anonymous mode disables identity routes and session polling instead of invoking partially configured OAuth. | Local and public anonymous environments start cleanly; GitHub/JWT modes fail fast when required secrets are missing. |
| 2026-09-02 | Hardened production images and excluded local build artifacts and environment files from the web build context. | Containers run as non-root users, omit unused package managers, receive patched OS packages, and scan clean at high/critical severity. |
