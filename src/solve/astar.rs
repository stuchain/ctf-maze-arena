use crate::maze::{Cell, Maze};
use crate::solve::{reconstruct_path, SolveResult, SolveStats, Solver};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

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
    }
}

impl Solver for AstarSolver {
    fn name(&self) -> &'static str {
        "ASTAR"
    }

    fn solve(&self, maze: &Maze) -> SolveResult {
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

        while let Some(Item { cell, .. }) = heap.pop() {
            if !visited.insert(cell) {
                continue;
            }
            if cell == maze.goal {
                break;
            }
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
            frames: vec![],
        }
    }
}
