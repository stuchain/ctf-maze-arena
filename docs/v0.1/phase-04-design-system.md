# Phase 4 — Design system and application shell

**Status:** ready

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

- [ ] Produce low-fidelity layout and state inventory before styling.
- [ ] Define semantic tokens and both themes.
- [ ] Build/test the minimal component set.
- [ ] Implement responsive application shell and navigation.
- [ ] Create branded metadata, iconography, copy voice, and theme persistence.
- [ ] Design skeleton, API wake-up, reconnect, empty, failure, and not-found states.
- [ ] Add accessibility automation and representative visual snapshots.

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

- [ ] Application shell is coherent at all target viewport classes.
- [ ] Both themes and all core product states are intentionally designed.
- [ ] Core navigation and controls are keyboard-complete.
- [ ] Representative accessibility and visual tests pass.
- [ ] No raw browser-default product controls or scaffolding copy remains.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| — | Not implemented | — |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-02 | Premium dark algorithm lab is primary; light theme remains first-class. | All tokens and visual tests require two-theme coverage. |
