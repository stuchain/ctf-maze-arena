use crate::maze::Maze;
use crate::solve::{SolveResult, SolveStats, Solver};

pub struct BfsSolver;

impl Solver for BfsSolver {
    fn name(&self) -> &'static str {
        "BFS"
    }

    fn solve(&self, _maze: &Maze) -> SolveResult {
        SolveResult {
            path: vec![],
            stats: SolveStats {
                visited: 0,
                cost: 0,
                ms: 0,
            },
        }
    }
}
