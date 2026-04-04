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
- Also produces a uniform random spanning tree (implementation uses random edge selection from the frontier).

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
