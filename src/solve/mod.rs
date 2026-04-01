use crate::maze::{Cell, Maze};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod bfs;
pub mod astar;
pub mod dfs;

use astar::AstarSolver;
use bfs::BfsSolver;
use dfs::DfsSolver;

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

pub type SolverRegistry = HashMap<String, Arc<dyn Solver>>;

pub fn default_registry() -> SolverRegistry {
    let mut r = SolverRegistry::new();
    r.insert("BFS".to_string(), Arc::new(BfsSolver));
    r.insert("DFS".to_string(), Arc::new(DfsSolver));
    r.insert("ASTAR".to_string(), Arc::new(AstarSolver));
    r
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
    use super::{default_registry, Solver, StubSolver};
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

    #[test]
    fn bfs_registered_in_default_registry() {
        let maze = Maze::new(2, 2);
        let registry = default_registry();
        let solver = registry.get("BFS").expect("BFS solver missing");
        let _ = solver.solve(&maze);
    }
}
