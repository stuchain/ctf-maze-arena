use crate::maze::{Cell, Maze};
use crate::solve::{
    cell_to_arr, reconstruct_path, NoopProgress, ProgressSink, SolveError, SolveProgress,
    SolveResult, SolveStats, Solver,
};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

pub struct AstarSolver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Item {
    f: usize,
    cell: Cell,
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.cell.x.cmp(&self.cell.x))
            .then_with(|| other.cell.y.cmp(&self.cell.y))
    }
}

impl Solver for AstarSolver {
    fn name(&self) -> &'static str {
        "ASTAR"
    }

    fn solve(&self, maze: &Maze) -> SolveResult {
        self.solve_with_progress(maze, &mut NoopProgress, &AtomicBool::new(false))
            .expect("unlimited solve cannot cancel")
    }

    fn solve_with_progress(
        &self,
        maze: &Maze,
        progress: &mut dyn ProgressSink,
        cancelled: &AtomicBool,
    ) -> Result<SolveResult, SolveError> {
        let start_time = std::time::Instant::now();
        let mut visited = HashSet::new();
        let mut parent: HashMap<Cell, Cell> = HashMap::new();
        let mut g: HashMap<Cell, usize> = HashMap::new();
        g.insert(maze.start, 0);

        let h = |c: Cell| c.x.abs_diff(maze.goal.x) + c.y.abs_diff(maze.goal.y);
        let mut heap = BinaryHeap::from([Item {
            f: h(maze.start),
            cell: maze.start,
        }]);
        let mut step = 0_u32;

        while let Some(Item { cell, .. }) = heap.pop() {
            if cancelled.load(AtomicOrdering::Acquire) {
                return Err(SolveError::Cancelled);
            }
            if !visited.insert(cell) {
                continue;
            }
            let reached_goal = cell == maze.goal;
            if !reached_goal {
                let g_cur = *g.get(&cell).unwrap_or(&usize::MAX);
                if g_cur == usize::MAX {
                    continue;
                }
                for next in maze.neighbors(cell) {
                    let g_next = g_cur + 1;
                    if g.get(&next).is_none_or(|&old| g_next < old) {
                        g.insert(next, g_next);
                        parent.insert(next, cell);
                        heap.push(Item {
                            f: g_next + h(next),
                            cell: next,
                        });
                    }
                }
            }
            step = step.saturating_add(1);
            let mut frontier: Vec<_> = heap.iter().map(|item| cell_to_arr(item.cell)).collect();
            frontier.sort_unstable();
            let mut visited_cells: Vec<_> =
                visited.iter().map(|&value| cell_to_arr(value)).collect();
            visited_cells.sort_unstable();
            progress.progress(SolveProgress {
                step,
                frontier,
                visited: visited_cells,
                current: Some(cell_to_arr(cell)),
            });
            if reached_goal {
                break;
            }
        }

        let path = reconstruct_path(&parent, maze.start, maze.goal);
        let cost = path.len().saturating_sub(1);
        let ms = start_time.elapsed().as_millis() as u64;
        Ok(SolveResult {
            path,
            stats: SolveStats {
                visited: visited.len(),
                cost,
                ms,
            },
        })
    }
}
