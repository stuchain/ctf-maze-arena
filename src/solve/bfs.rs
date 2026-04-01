use crate::maze::{Cell, Maze};
use crate::solve::{cell_to_arr, reconstruct_path, SolveFrame, SolveResult, SolveStats, Solver};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct BfsSolver;

impl Solver for BfsSolver {
    fn name(&self) -> &'static str {
        "BFS"
    }

    fn solve(&self, maze: &Maze) -> SolveResult {
        let start_time = std::time::Instant::now();
        let mut visited = HashSet::new();
        let mut parent: HashMap<Cell, Cell> = HashMap::new();
        let mut queue = VecDeque::from([maze.start]);
        let mut frames = Vec::new();
        let mut t = 0_u32;

        while let Some(cell) = queue.pop_front() {
            frames.push(SolveFrame {
                t,
                frontier: queue
                    .iter()
                    .map(|&c| cell_to_arr(c))
                    .collect(),
                visited: visited.iter().map(|&c| cell_to_arr(c)).collect(),
                current: Some(cell_to_arr(cell)),
            });
            t = t.saturating_add(1);
            if !visited.insert(cell) {
                continue;
            }
            if cell == maze.goal {
                break;
            }
            for next in maze.neighbors(cell) {
                if !visited.contains(&next) && !parent.contains_key(&next) {
                    parent.insert(next, cell);
                    queue.push_back(next);
                }
            }
        }

        let path = reconstruct_path(&parent, maze.start, maze.goal);
        let cost = path.len().saturating_sub(1);
        let ms = start_time.elapsed().as_millis() as u64;
        SolveResult {
            path,
            stats: SolveStats {
                visited: visited.len(),
                cost,
                ms,
            },
            frames,
        }
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
    fn bfs_collects_animation_frames() {
        let maze = generate(5, 5, 7, GeneratorAlgo::Kruskal);
        let result = BfsSolver.solve(&maze);
        assert!(!result.frames.is_empty());
    }
}

