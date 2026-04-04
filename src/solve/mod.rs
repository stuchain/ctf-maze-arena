use crate::maze::{Cell, Maze};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod bfs;
pub mod astar;
pub mod dfs;
pub mod dp;

#[cfg(test)]
mod tests;

pub use astar::AstarSolver;
pub use bfs::BfsSolver;
pub use dfs::DfsSolver;
pub use dp::DpKeysSolver;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveStats {
    pub visited: usize,
    pub cost: usize,
    pub ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveFrame {
    pub t: u32,
    pub frontier: Vec<[u32; 2]>,
    pub visited: Vec<[u32; 2]>,
    pub current: Option<[u32; 2]>,
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub path: Vec<Cell>,
    pub stats: SolveStats,
    pub frames: Vec<SolveFrame>,
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
    r.insert("DP_KEYS".to_string(), Arc::new(DpKeysSolver));
    r
}

pub(crate) fn reconstruct_path(parent: &HashMap<Cell, Cell>, start: Cell, goal: Cell) -> Vec<Cell> {
    if start == goal {
        return vec![start];
    }

    let mut path = Vec::new();
    let mut cur = goal;
    loop {
        path.push(cur);
        if cur == start {
            break;
        }
        match parent.get(&cur) {
            Some(&p) => cur = p,
            None => return vec![],
        }
    }
    path.reverse();
    path
}

pub(crate) fn cell_to_arr(c: Cell) -> [u32; 2] {
    [c.x as u32, c.y as u32]
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
            frames: vec![],
        }
    }
}

