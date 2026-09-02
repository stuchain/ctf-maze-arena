# Phase 7 — Identity and community

**Status:** not-started

**Depends on:** Phases 2, 4, and 6

**Purpose:** add durable identity and replayability without blocking anonymous visitors

## Problem

GitHub authentication exists, but the displayed leaderboard is not tied to submission and achievements live only in browser storage. Daily challenges, identity, ranking, and replay sharing are therefore disconnected.

## Goals

- Keep all core play available anonymously.
- Use GitHub sign-in for profile, legitimate submission, persistent achievements, streaks, and personal history.
- Make daily challenges and leaderboards trustworthy and useful.
- Provide privacy, abuse, and deletion behavior appropriate for a public demo.

## Non-goals

- No general social network, comments, messaging, teams, or custom avatars.
- No password authentication.
- No prize-bearing competition or anti-cheat guarantees beyond server-authoritative runs.

## Identity flow

NextAuth authenticates with GitHub using minimal scopes. The web server mints a short-lived API JWT containing a stable provider subject and required claims. The API validates algorithm, expiry, issuer, audience, and clock skew, then upserts the user only for identity-sensitive actions. Anonymous runs can later be submitted only if ownership was bound while authenticated; arbitrary anonymous run claiming is forbidden.

## Community model

### Leaderboards

- Filters: daily, maze/race, solver, and personal.
- Stable ranking: path cost or challenge score, then compute/runtime metric where valid, visited nodes, accepted timestamp, and stable ID.
- Paginated queries and clear tie presentation.
- Only accepted `leaderboard_submissions` appear.

### Daily challenge

Generate a server-defined versioned challenge from UTC date plus challenge version. Store the challenge definition so later algorithm/config changes do not rewrite history. Show countdown, personal best, completion state, and streak based on accepted completions.

### Achievements

Achievements are server-defined, versioned, and awarded from authoritative run facts. Existing local achievements may be displayed as legacy-local only or discarded; they are never silently promoted to server achievements.

### Profiles and privacy

Store only provider subject, public display name/avatar as needed, and product records. Provide sign-out, an explanation of stored data, and a deletion path that removes or anonymizes personal records while preserving leaderboard integrity according to the documented policy.

## Work checklist

- [ ] Harden JWT issuer/audience/algorithm configuration and auth error behavior.
- [ ] Implement user upsert/profile and authenticated ownership binding.
- [ ] Connect explicit score submission to the corrected leaderboard.
- [ ] Implement versioned daily challenges, personal best, and streak.
- [ ] Implement server-backed versioned achievements.
- [ ] Add leaderboard filters, pagination, ties, empty/loading/error states.
- [ ] Add replay/result share metadata and signed-in/out conversion flow.
- [ ] Add privacy copy, data export/deletion path, and abuse limits.

## Test strategy

- Auth unit/integration tests for signature, issuer, audience, expiry, clock skew, missing claims, and modes.
- Ownership and submission tests prevent claiming another run or duplicate scoring.
- Daily challenge tests across UTC boundaries and version changes.
- Achievement idempotency and historical-version tests.
- E2E anonymous play, sign-in boundary, authenticated submit, profile, and sign-out.
- Accessibility tests for account menus, dialogs, tables, and live feedback.

## Risks

- GitHub OAuth cannot be fully exercised in normal CI. Keep token validation integration tests and a documented production smoke check.
- Public names/avatars introduce privacy and content concerns. Display provider-controlled public identity only, with conservative fallback and deletion behavior.
- Gamification can distract from the algorithm showcase. Keep community panels secondary to the lab and race.

## Exit criteria

- [ ] Anonymous visitors complete every core learning/play flow.
- [ ] GitHub identity reliably enables only authorized persistent features.
- [ ] Leaderboards contain valid, unique, server-authoritative submissions.
- [ ] Daily challenge history and achievements are durable and versioned.
- [ ] Privacy, deletion, abuse, and empty/error behavior are documented and verified.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| — | Not implemented | — |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-02 | GitHub sign-in is optional for play and required for persistent community features. | Core routes and UI must always preserve an anonymous path. |
