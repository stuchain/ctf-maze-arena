use crate::maze::{Cell, Maze};
use crate::solve::{
    cell_to_arr, reconstruct_path, NoopProgress, ProgressSink, SolveError, SolveProgress,
    SolveResult, SolveStats, Solver,
};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct DfsSolver;

impl Solver for DfsSolver {
    fn name(&self) -> &'static str {
        "DFS"
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
        let mut stack = vec![maze.start];
        let mut step = 0_u32;

        while let Some(cell) = stack.pop() {
            if cancelled.load(Ordering::Acquire) {
                return Err(SolveError::Cancelled);
            }
            if !visited.insert(cell) {
                continue;
            }
            let reached_goal = cell == maze.goal;
            if !reached_goal {
                for next in maze.neighbors(cell) {
                    if !visited.contains(&next) && !parent.contains_key(&next) {
                        parent.insert(next, cell);
                        stack.push(next);
                    }
                }
            }
            step = step.saturating_add(1);
            let mut visited_cells: Vec<_> =
                visited.iter().map(|&value| cell_to_arr(value)).collect();
            visited_cells.sort_unstable();
            progress.progress(SolveProgress {
                step,
                frontier: stack.iter().map(|&value| cell_to_arr(value)).collect(),
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
