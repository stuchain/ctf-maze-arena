use crate::maze::{Cell, Maze};
use crate::solve::{SolveResult, SolveStats, Solver};
use std::collections::{HashMap, HashSet};

pub struct DfsSolver;

impl Solver for DfsSolver {
    fn name(&self) -> &'static str {
        "DFS"
    }

    fn solve(&self, maze: &Maze) -> SolveResult {
        let start_time = std::time::Instant::now();
        let mut visited = HashSet::new();
        let mut parent: HashMap<Cell, Cell> = HashMap::new();
        let mut stack = vec![maze.start];

        while let Some(cell) = stack.pop() {
            if !visited.insert(cell) {
                continue;
            }
            if cell == maze.goal {
                break;
            }
            for next in maze.neighbors(cell) {
                if !visited.contains(&next) && !parent.contains_key(&next) {
                    parent.insert(next, cell);
                    stack.push(next);
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
        }
    }
}

fn reconstruct_path(parent: &HashMap<Cell, Cell>, start: Cell, goal: Cell) -> Vec<Cell> {
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
