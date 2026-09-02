# Phase 9 — Portfolio launch

**Status:** not-started

**Depends on:** Phases 6, 7, and 8

**Purpose:** package the finished engineering and product story for public evaluation

## Problem

A technically strong repository can still fail as a portfolio piece when the README, screenshots, architecture story, live link, setup, and tradeoffs are unclear. Current project and frontend READMEs understate the system and retain scaffold/history language.

## Goals

- Make the live demo and signature value obvious in the first screen of the repository.
- Explain architecture, algorithms, quality practices, deployment constraints, and tradeoffs concisely.
- Provide compelling media and a reproducible setup.
- Complete a cross-discipline release audit and tag v0.1.

## README design

The root README order:

1. Product name, one-sentence pitch, hero image/GIF, live Vercel link, and primary badges.
2. Signature features, led by Algorithm Race.
3. Short “try this” path with a deterministic example challenge.
4. Architecture diagram and request/solve sequence.
5. Algorithm comparison and correctness guarantees.
6. Technology choices and tradeoffs, including free-tier cold starts.
7. Local setup for native and Docker workflows.
8. Testing, accessibility, security, performance, and operations evidence.
9. API/docs links, roadmap/history, license, and author/contact.

Badges must represent meaningful current signals and not become decorative noise.

## Media plan

- Hero still at repository/social-preview dimensions.
- Short optimized GIF or video: configure deterministic maze → start Algorithm Race → inspect result → share replay.
- Mobile and light-theme screenshots.
- Architecture and realtime sequence diagrams rendered clearly on GitHub.
- Use real production UI and data, not mockups, for final media.

## Engineering case study

Document the most valuable decisions:

- Why Rust/Axum and Next.js are separated.
- How deterministic generation and solver guarantees are tested.
- How incremental deltas, backpressure, reconnect, and replay interact.
- Why Postgres replaced SQLite for free cloud hosting.
- How completely free deployment changes cold-start and availability design.
- How accessibility and performance affected visualization architecture.

Include measured results with environment and methodology. Avoid unsupported “production-grade,” “blazing fast,” or benchmark claims.

## Repository presentation

- Accurate description, topics, social preview, license, and release notes.
- Useful issue and pull-request templates without enterprise ceremony.
- Link current design docs and archive historical plans clearly.
- Remove stale or duplicate documentation and confirm every internal link.
- Publish a v0.1 changelog/release describing user-visible and engineering outcomes.

## Final audit

- Functional: anonymous, auth, daily, race, replay, share, leaderboard, failure recovery.
- Browser/device: current Chromium, Firefox, WebKit; mobile and desktop.
- Accessibility: automated and manual keyboard/screen-reader/zoom/reduced-motion.
- Performance: production bundle, load, interaction, large maze, race, API cold/warm behavior.
- Security: secrets, auth boundaries, CORS/origin, input limits, dependencies, container, data deletion.
- Operations: migrations, redeploy, cold start, reconnect, rollback, retention.
- Documentation: fresh clone, links, screenshots, API/algorithm accuracy.

## Work checklist

- [ ] Capture final hero, demo video/GIF, mobile, and light-theme media.
- [ ] Rewrite root README and remove/redirect the scaffold frontend README.
- [ ] Add architecture and realtime sequence diagrams.
- [ ] Publish measured algorithm/performance evidence with methodology.
- [ ] Complete cross-browser, accessibility, performance, security, and operational audits.
- [ ] Verify setup from a fresh clone on documented prerequisites.
- [ ] Configure repository description, topics, social preview, templates, and links.
- [ ] Write changelog/release notes and tag v0.1.

## Test strategy

Use the final audit as a release checklist stored with evidence. All automated checks run against the release commit, and production smoke checks run after deployment. Any waiver includes owner, reason, impact, and follow-up.

## Risks

- Media and README can become stale immediately. Capture from the tagged release and keep versioned assets.
- Excessive documentation can hide the live link. Optimize for a one-minute reviewer first, deep technical readers second.
- Free-tier downtime can harm first impressions. Include excellent media and honest wake-up handling so the repository still communicates value.

## Exit criteria

- [ ] A reviewer can understand and launch the project from the first README screen.
- [ ] Demo media showcases Algorithm Race and matches the deployed release.
- [ ] Fresh-clone setup and all documentation links are verified.
- [ ] Final audit has no unresolved critical issue.
- [ ] Public v0.1 release and Vercel link are live.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| — | Not implemented | — |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| — | None | — |
