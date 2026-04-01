use crate::maze::{Cell, Maze};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveStats {
    pub visited: usize,
    pub cost: usize,
    pub ms: u64,
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub path: Vec<Cell>,
    pub stats: SolveStats,
}

pub trait Solver: Send + Sync {
    fn name(&self) -> &'static str;
    fn solve(&self, maze: &Maze) -> SolveResult;
}

pub struct StubSolver;

impl Solver for StubSolver {
    fn name(&self) -> &'static str {
        "STUB"
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

#[cfg(test)]
mod tests {
    use super::{Solver, StubSolver};
    use crate::maze::Maze;

    #[test]
    fn solver_trait_object_compiles_and_runs() {
        let maze = Maze::new(2, 2);
        let s: Box<dyn Solver> = Box::new(StubSolver);
        let r = s.solve(&maze);
        assert!(r.path.is_empty());
        assert_eq!(r.stats.visited, 0);
        assert_eq!(r.stats.cost, 0);
        assert_eq!(r.stats.ms, 0);
    }
}
