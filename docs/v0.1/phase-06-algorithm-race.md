# Phase 6 — Algorithm Race

**Status:** not-started

**Depends on:** Phase 5

**Purpose:** deliver the signature portfolio feature

## Problem

Selecting and running one solver demonstrates functionality but does not clearly expose algorithm tradeoffs. Metrics lack context, and the current educational material is separate from the experience.

## Goals

- Compare multiple solvers fairly on the same immutable maze.
- Make exploration behavior, guarantees, and tradeoffs visually obvious.
- Provide synchronized and independent inspection modes.
- Produce deterministic, shareable challenge configurations.
- Connect measured behavior to accurate educational explanations.

## Non-goals

- No claim that wall-clock runtime from a shared concurrent race is a rigorous microbenchmark.
- No user-supplied executable solver code in v0.1.
- No large catalog of algorithms before the four existing solvers are presented excellently.

## Fairness model

Every competitor receives the same immutable maze representation and solver configuration. For comparable unweighted mazes, BFS and A* optimality is checked against path cost; DFS is explicitly labeled non-optimal. DP Keys appears when the maze contains the relevant key/door state or as an advanced preset.

Record two timing concepts separately:

- **Compute runtime:** measured by the backend under documented conditions.
- **Playback time:** user-controlled visualization duration, never presented as compute performance.

Concurrent execution can contend on the free host. Default backend measurement may run solvers sequentially while the client presents synchronized logical playback. If true concurrent execution is offered, label it as an experience mode rather than a benchmark.

## Experience modes

### Race overview

One maze with compact competitor lanes/statuses, synchronized progress, live visited/frontier counts, finish order, and a final comparison panel.

### Side-by-side inspection

Two to four synchronized maze stages for visually comparing exploration shapes. On constrained screens, use tabs or a selected pair rather than unreadably small grids.

### Results analysis

Compare path cost, visited cells, peak frontier, compute runtime, completion status, and guarantees. Provide a short generated explanation based on deterministic rules—not AI—such as why A* explored less or why DFS returned a longer path.

## Shareable challenge contract

Encode a versioned, validated configuration in URL parameters: width, height, seed, generator, maze feature preset, selected solvers, and display mode. The server remains authoritative and rejects unsupported sizes/values. A canonical URL omits defaults and supports Open Graph metadata on replay/result pages.

## Educational content

For each generator and solver, document mechanism, time/space complexity, completeness, optimality, and characteristic behavior. Correct the existing statement that randomized Prim produces a uniform random spanning tree. Link explanations to visible metrics and states.

## Work checklist

- [ ] Define race request/result and shareable configuration schemas.
- [ ] Implement fair orchestration and resource/concurrency policy.
- [ ] Build overview, side-by-side, and results-analysis modes.
- [ ] Add synchronized playback and independent inspection controls.
- [ ] Add peak-frontier and relevant metrics with clear definitions.
- [ ] Add deterministic result explanations and algorithm education panels.
- [ ] Add canonical share URLs and result/replay integration.
- [ ] Correct and expand algorithm documentation.

## Test strategy

- Property tests for optimality claims across many deterministic mazes.
- Contract tests ensure all solvers receive identical maze inputs.
- Unit tests for metrics, result explanations, and URL canonicalization.
- Responsive visual tests for one through four competitors.
- E2E test creates, completes, shares, and reloads a race.
- Benchmark orchestration overhead separately from solver runtime.

## Risks

- Four animated grids can overload low-end devices. Dynamically reduce detail/frame rate and prefer overview mode on small screens.
- Runtime rankings can be noisy on free shared infrastructure. Emphasize structural metrics and label timing limitations.
- DP Keys is not directly comparable on mazes without stateful obstacles. Gate it to compatible presets and explain the expanded state space.

## Exit criteria

- [ ] A visitor can run and understand a multi-solver comparison without documentation.
- [ ] Fairness, timing, and algorithm guarantees are accurate and tested.
- [ ] Shared race URLs recreate the same configuration and results semantics.
- [ ] The experience remains responsive across target viewports and maze sizes.
- [ ] Algorithm documentation matches implementation and accepted theory.

## Verification record

| Date | Change | Evidence |
|---|---|---|
| — | Not implemented | — |

## Decision and deviation log

| Date | Decision or deviation | Consequence |
|---|---|---|
| 2026-09-02 | Algorithm Race is the non-negotiable signature feature. | Lower-priority community scope may be reduced before this phase is compromised. |
