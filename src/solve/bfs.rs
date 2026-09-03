use crate::maze::{Cell, Maze};
use crate::solve::{
    cell_to_arr, reconstruct_path, NoopProgress, ProgressSink, SolveError, SolveProgress,
    SolveResult, SolveStats, Solver,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct BfsSolver;

impl Solver for BfsSolver {
    fn name(&self) -> &'static str {
        "BFS"
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
        let mut queue = VecDeque::from([maze.start]);
        let mut t = 0_u32;

        while let Some(cell) = queue.pop_front() {
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
                        queue.push_back(next);
                    }
                }
            }
            t = t.saturating_add(1);
            let mut visited_cells: Vec<_> =
                visited.iter().map(|&value| cell_to_arr(value)).collect();
            visited_cells.sort_unstable();
            progress.progress(SolveProgress {
                step: t,
                frontier: queue.iter().map(|&value| cell_to_arr(value)).collect(),
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

#[cfg(test)]
mod tests {
    use super::BfsSolver;
    use crate::maze::{generate, GeneratorAlgo, Maze};
    use crate::solve::Solver;

    #[test]
    fn bfs_path_valid() {
        let maze = generate(5, 5, 42, GeneratorAlgo::Kruskal);
        let result = BfsSolver.solve(&maze);
        assert!(!result.path.is_empty());
        assert_eq!(result.path[0], maze.start);
        assert_eq!(result.path.last().copied(), Some(maze.goal));
        for i in 1..result.path.len() {
            let (a, b) = (result.path[i - 1], result.path[i]);
            assert!(maze.neighbors(a).contains(&b));
        }
    }

    #[test]
    fn bfs_start_equals_goal_returns_single_cell() {
        let mut maze = Maze::new(1, 1);
        maze.start = crate::maze::Cell::new(0, 0);
        maze.goal = crate::maze::Cell::new(0, 0);
        let result = BfsSolver.solve(&maze);
        assert_eq!(result.path, vec![maze.start]);
        assert_eq!(result.stats.cost, 0);
    }

    #[test]
    fn bfs_emits_live_progress() {
        let maze = generate(5, 5, 7, GeneratorAlgo::Kruskal);
        struct Counter(usize);
        impl crate::solve::ProgressSink for Counter {
            fn progress(&mut self, _: crate::solve::SolveProgress) {
                self.0 += 1;
            }
        }
        let mut counter = Counter(0);
        BfsSolver
            .solve_with_progress(
                &maze,
                &mut counter,
                &std::sync::atomic::AtomicBool::new(false),
            )
            .unwrap();
        assert!(counter.0 > 1);
    }
}
