use crate::maze::{neighbors_all, Cell, Maze};
use crate::solve::{SolveResult, SolveStats, Solver};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DpState {
    pub cell: Cell,
    pub keys: u32,
}

impl DpState {
    pub fn initial(start: Cell) -> Self {
        Self {
            cell: start,
            keys: 0,
        }
    }

    pub fn with_key(&self, key_id: u8) -> Self {
        Self {
            cell: self.cell,
            keys: self.keys | (1 << key_id),
        }
    }

    pub fn has_key(&self, key_id: u8) -> bool {
        (self.keys & (1 << key_id)) != 0
    }
}

pub struct DpKeysSolver;

impl Solver for DpKeysSolver {
    fn name(&self) -> &'static str {
        "DP_KEYS"
    }

    fn solve(&self, maze: &Maze) -> SolveResult {
        let start_time = std::time::Instant::now();
        let mut visited = HashSet::new();
        let mut parent: HashMap<DpState, DpState> = HashMap::new();
        let init = DpState::initial(maze.start);
        let mut queue = VecDeque::from([init]);
        let mut goal_state = None;

        while let Some(state) = queue.pop_front() {
            if !visited.insert(state) {
                continue;
            }
            if state.cell == maze.goal {
                goal_state = Some(state);
                break;
            }
            for next_cell in neighbors_all(state.cell, maze.grid.width, maze.grid.height) {
                if !maze.can_move(state.cell, next_cell, state.keys) {
                    continue;
                }
                let mut next_state = DpState {
                    cell: next_cell,
                    keys: state.keys,
                };
                if let Some(kid) = maze.has_key_at(next_cell) {
                    next_state = next_state.with_key(kid);
                }
                if !visited.contains(&next_state) && !parent.contains_key(&next_state) {
                    parent.insert(next_state, state);
                    queue.push_back(next_state);
                }
            }
        }

        let path = match goal_state {
            Some(final_state) => reconstruct_dp_path(&parent, init, final_state),
            None => vec![],
        };
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

fn reconstruct_dp_path(
    parent: &HashMap<DpState, DpState>,
    start: DpState,
    goal: DpState,
) -> Vec<Cell> {
    let mut path = Vec::new();
    let mut cur = goal;
    loop {
        path.push(cur.cell);
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

#[cfg(test)]
mod tests {
    use super::{DpKeysSolver, DpState};
    use crate::maze::{Cell, Edge, Maze};
    use crate::solve::Solver;

    #[test]
    fn state_bitmask_tracks_keys() {
        let start = Cell::new(0, 0);
        let s = DpState::initial(start).with_key(0);
        assert!(s.has_key(0));
        assert!(!s.has_key(1));
    }

    fn make_keys_maze() -> Maze {
        let mut maze = Maze::with_all_walls(3, 3);
        maze.start = Cell::new(0, 0);
        maze.goal = Cell::new(2, 2);
        maze.walls.remove_wall(Cell::new(0, 0), Cell::new(1, 0));
        maze.walls.remove_wall(Cell::new(1, 0), Cell::new(1, 1));
        maze.walls.remove_wall(Cell::new(1, 1), Cell::new(2, 1));
        maze.walls.remove_wall(Cell::new(2, 1), Cell::new(2, 2));
        maze.keys.insert(Cell::new(1, 1), 0);
        maze.doors
            .insert(Edge::normalized(Cell::new(2, 1), Cell::new(2, 2)), 0);
        maze
    }

    #[test]
    fn dp_collects_key_before_door() {
        let maze = make_keys_maze();
        let r = DpKeysSolver.solve(&maze);
        assert!(!r.path.is_empty());
        assert!(r.path.contains(&Cell::new(1, 1)));
        let key_idx = r.path.iter().position(|&c| c == Cell::new(1, 1)).unwrap();
        let door_idx = r.path.iter().position(|&c| c == Cell::new(2, 2)).unwrap();
        assert!(key_idx < door_idx);
    }

    #[test]
    fn dp_returns_empty_when_impossible() {
        let mut maze = Maze::with_all_walls(2, 1);
        maze.start = Cell::new(0, 0);
        maze.goal = Cell::new(1, 0);
        maze.walls.remove_wall(Cell::new(0, 0), Cell::new(1, 0));
        maze.doors
            .insert(Edge::normalized(Cell::new(0, 0), Cell::new(1, 0)), 0);
        let r = DpKeysSolver.solve(&maze);
        assert!(r.path.is_empty());
    }
}
