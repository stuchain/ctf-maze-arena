# Phase 4 layout and state inventory

**Status:** implemented

**Purpose:** record the low-fidelity structure and product-state contract approved for the Phase 4 application shell before visual styling.

## Desktop structure

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Brand / descriptor                    Repository  Theme  Account     │
├────────────────┬────────────────────────────────┬────────────────────┤
│ 01 Configure   │ 02 Live Arena                  │ 03 Inspect         │
│ Daily seed     │ Run status                     │ Stream health      │
│ Dimensions     │                                │ Visited / cost     │
│ Seed           │          Maze stage            │ Runtime / solver   │
│ Generator      │                                │ Score action       │
│ Solver         │ Legend             Run action  │ Achievements       │
├────────────────┴───────────────────┬─────────────┴────────────────────┤
│ Leaderboard                        │ Algorithm note                  │
└────────────────────────────────────┴──────────────────────────────────┘
```

At tablet width, the configuration rail and stage remain side by side while the inspector spans the next row. On mobile, the visual order is stage, configuration, inspector; a sticky section navigator provides direct access without hiding controls in JavaScript-only drawers.

## State inventory

| Surface | Empty/cold | Busy/transitional | Success | Failure/interruption | Disabled |
|---|---|---|---|---|---|
| Arena | Branded generate prompt | Generate/start labels and stream status | Completed badge, path, telemetry | Actionable API/stream notice | Solve unavailable until a maze exists |
| API | Standby | Waking and connecting badges | Live or completed | Reconnecting warning with fallback copy | — |
| Replay | Skeleton summary and stage | Playing badge and frame counter | Complete badge and final path | Branded unavailable state with arena link | Contextual play, pause, and reset states |
| Leaderboard | Ranked empty-state invitation | Score submission label | Submission confirmation | Actionable score error | Ranked action absent for guests |
| Navigation | — | Account loading label | Signed-in identity | Anonymous guest mode | — |
| Destructive run action | — | Cancelling label | Cancelled stream state | Actionable cancel error | Confirmation prevents accidental activation |
| Route | — | Route-level skeleton | Requested screen | Branded 404 and error boundaries | — |

## Interaction contract

- Every control has a persistent accessible name and visible keyboard focus.
- Status changes use polite live regions; blocking failures use alerts.
- Cancellation requires confirmation and remains operable with Escape.
- Theme follows system preference on first visit and persists an explicit choice.
- Motion is limited to state communication and collapses under reduced-motion preference.
- Color is reinforced with text, symbols, or position for meaningful maze and status states.

## Phase boundary

This inventory establishes the shell, tokens, and reusable states. Maze wall geometry, render-performance work, richer replay semantics, and canvas alternatives remain Phase 5 responsibilities.
