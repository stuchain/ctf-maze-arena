# Phase 4 — Design system and application shell

**Status:** complete

**Depends on:** Phase 1

**Purpose:** establish the premium visual and interaction foundation before feature-heavy UI work

## Problem

The current application is a centered column of unrelated controls and default Tailwind treatments. Hierarchy, branding, responsive composition, state feedback, and component consistency are insufficient for a showcase product.

## Goals

- Create a distinctive premium dark algorithm-lab identity with complete light mode.
- Build accessible, reusable primitives and semantic design tokens.
- Create a responsive application shell optimized for the maze workspace.
- Design every loading, empty, error, disabled, success, and cold-start state.
- Establish visual-regression and accessibility gates.

## Non-goals

- No maze rendering rewrite; Phase 5 owns the stage internals.
- No large animation library unless native CSS/Web Animations cannot meet requirements.
- No decorative 3D, glassmorphism, or effects that reduce clarity or performance.

## Information architecture

### Global shell

- Header: CTF Maze Arena identity, concise product descriptor, GitHub repository link, theme control, account control.
- Primary workspace: configuration rail, maze stage, live inspector.
- Secondary content: results/comparison, educational detail, leaderboard, achievements.
- Mobile: configuration and inspector become accessible sheets/tabs; maze remains primary.

### Core user flow

1. Land and immediately understand the project.
2. Use a curated preset, daily challenge, or advanced configuration.
3. Generate a maze and choose one solver or Algorithm Race.
4. Watch, control, and understand the run.
5. Inspect results, replay, share, or sign in to submit.

## Design tokens

Define semantic tokens rather than component-specific colors:

- Surfaces: canvas, base, raised, overlay, interactive.
- Text: primary, secondary, muted, inverse.
- Borders: subtle, default, strong, focus.
- Actions: primary, secondary, destructive.
- Maze states: unvisited, frontier, visited, current, path, start, goal, key, door.
- Status: info, success, warning, danger.
- Typography, spacing, radii, control heights, layout widths, layering, and motion durations/easing.

Maze colors must remain distinguishable in both themes and common color-vision deficiencies. Color is paired with shape, label, pattern, or position where state is important.

## Component system

Build only components required by planned screens: button, icon button, field, select, segmented control, slider, switch, card/panel, tabs, dialog/sheet, tooltip, popover, toast, badge, table, skeleton, empty/error state, and visually-hidden/live-region utilities. Components expose consistent sizes, focus behavior, disabled/loading states, and test selectors based on semantics rather than styling.

## Responsive design

- 320–639px: stage-first single column; controls in bottom sheet or tabs.
- 640–1023px: two-region layout with collapsible inspector.
- 1024px and above: three-region lab workspace.
- Use container-aware stage sizing and avoid fixed pixel maze dimensions.
- Support 200% zoom without loss of core function.

## Motion

Motion communicates causality: panel transition, state change, solve progress, path completion, and achievement feedback. Use transform/opacity where possible. Reduced-motion mode replaces continuous movement with immediate or short cross-fade state changes and retains playback controls.

## Work checklist

- [x] Produce low-fidelity layout and state inventory before styling.
- [x] Define semantic tokens and both themes.
- [x] Build/test the minimal component set.
- [x] Implement responsive application shell and navigation.
- [x] Create branded metadata, iconography, copy voice, and theme persistence.
- [x] Design skeleton, API wake-up, reconnect, empty, failure, and not-found states.
- [x] Add accessibility automation and representative visual snapshots.

## Test strategy

- Component interaction tests for keyboard, focus, disabled/loading, and screen-reader naming.
- Automated axe checks on representative states.
- Visual snapshots at mobile, tablet, desktop, dark, light, and reduced-motion settings.
- Manual keyboard-only, 200% zoom, and screen-reader smoke passes.
- Monitor bundle impact of any component or icon dependency.

## Risks

- “Top notch” can become subjective churn. Approve tokens, layout, and representative components before full conversion.
- Building a generic design system can consume the project. Only planned components are in scope.
- Dark technical styling can become cliché or inaccessible. Restraint and semantic contrast are acceptance criteria.

## Exit criteria

- [x] Application shell is coherent at all target viewport classes.
- [x] Both themes and all core product states are intentionally designed.
- [x] Core navigation and controls are keyboard-complete.
- [x] Representative accessibility and visual tests pass.
- [x] No raw browser-default product controls or scaffolding copy remains.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| 2026-09-03 | Semantic token system, dark/light themes, responsive three-region shell, reusable controls, branded metadata/iconography, and intentional route/run/replay states | `npm run lint`; `npm run typecheck`; `npm run test:unit`; `npm run build` |
| 2026-09-03 | Keyboard, reduced-motion, 320px mobile, tablet, desktop, 200% zoom equivalent, cancellation dialog, complete/missing replays, and dark/light axe coverage | `npm run test:e2e` — 11 passed against the production frontend, real Rust API, and PostgreSQL; representative screenshots captured as test attachments |
| 2026-09-03 | Cross-stack regression gate | `./scripts/verify.ps1 -Scope all` — Rust format/Clippy, 68 Rust tests, 11 frontend unit tests, lint, TypeScript, and production build passed |
| 2026-09-03 | Manual visual and console smoke | Dark and light desktop renders reviewed; no browser warnings, errors, page errors, or horizontal overflow remained |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-02 | Premium dark algorithm lab is primary; light theme remains first-class. | All tokens and visual tests require two-theme coverage. |
| 2026-09-03 | Use semantic CSS tokens and a small in-repository primitive layer rather than a general-purpose UI package. | The shell remains distinctive and dependency cost stays limited to test-only axe automation. |
| 2026-09-03 | Keep configuration and inspector in document flow on mobile with a sticky section navigator. | Stage-first reading order and 200% zoom remain robust without JavaScript-owned drawer state. |
| 2026-09-03 | Treat screenshot attachments plus geometric assertions as the portable visual gate for Phase 4. | CI retains reviewable visual evidence without introducing operating-system-sensitive pixel baselines; strict rendering baselines may be added with a standardized Linux image later. |
| 2026-09-03 | Return an empty session payload in anonymous mode while keeping every other disabled auth route unavailable. | NextAuth no longer produces expected-but-noisy browser errors, and anonymous mode remains closed to sign-in operations. |
