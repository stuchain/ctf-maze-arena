# Algorithm Notes

## Maze generation

### Kruskal

- Each cell starts in its own component (union–find).
- All internal walls between adjacent cells are collected and shuffled (seeded RNG).
- For each wall, if its two cells are in different components, remove the wall and merge the components.
- Result: a spanning tree of the grid — exactly one simple path between any two cells (a *perfect* maze).

### Prim

- Grow a tree from a starting cell. The *frontier* is edges from the tree to cells not yet in the tree.
- Repeatedly pick a random frontier edge (seeded), add the outside cell, and remove the wall between them.
- Produces a spanning tree by choosing uniformly from the current frontier edges. This local choice does **not** make the result a uniform sample over all grid spanning trees.

### DFS backtracker

- Start at a corner (e.g. `(0,0)`). Shuffle neighbor order (seeded).
- For each neighbor separated by a wall, if unvisited, remove the wall, recurse, then backtrack.
- Still a spanning tree; tends to produce longer, winding corridors than Kruskal/Prim.

## Solvers

Grid moves are **4-directional** (no diagonals) in this project.

### BFS

- Explores in waves of increasing distance from the start.
- On an unweighted grid with unit step cost, the first time the goal is reached yields a **shortest path** in number of steps.

### DFS

- Explores deeply before backtracking (stack order).
- **Does not** guarantee a shortest path; often fast but path length can be large.

### A*

- Uses `f = g + h` where `g` is cost from start and `h` is a heuristic.
- The implementation uses **Manhattan distance** to the goal, which is admissible for this 4-neighbor grid with unit cost, so the path is optimal with respect to step count.
- Typically expands fewer cells than BFS when a good path exists.

### DP (keys / doors)

- State is `(cell, keys_bitmask)` where each bit marks holding a given key id.
- Search (BFS-style expansion) is performed in this expanded graph: moves respect walls and doors (need the required key in the bitmask); picking up a key sets the corresponding bit.
- Finds a shortest path in the state graph when a solution exists.

## Race metrics

- **Path cost:** number of unit moves in the returned path.
- **Visited:** states expanded before reaching the goal. For DP Keys this counts `(cell, key-set)` states, so it is not directly comparable with cell-only solvers.
- **Peak frontier:** largest number of queued/open states observed at one time; a practical indicator of search memory pressure, not total memory allocation.
- **Compute runtime:** wall-clock time inside one backend solver execution. Race solvers run sequentially through the bounded compute gate, avoiding deliberate inter-solver CPU contention, but infrastructure noise still makes this unsuitable as a microbenchmark.
- **Playback time:** user-controlled visualization time. It is never used as a compute-performance result.

For classic unit-cost mazes, BFS and A* must return equal optimal path costs. DFS remains complete on the finite maze graph but does not guarantee an optimal path. A* uses Manhattan distance, which is consistent and admissible for four-directional unit-cost movement. DP Keys is complete and optimal in its expanded state graph.
