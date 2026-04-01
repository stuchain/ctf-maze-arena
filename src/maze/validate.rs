use std::collections::{HashSet, VecDeque};

use super::{Cell, Maze};

fn bfs_reachable(maze: &Maze, from: Cell) -> HashSet<Cell> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([from]);

    while let Some(cell) = queue.pop_front() {
        if !visited.insert(cell) {
            continue;
        }
        for neighbor in maze.neighbors(cell) {
            if !visited.contains(&neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    visited
}

pub fn is_fully_connected(maze: &Maze) -> bool {
    let reached = bfs_reachable(maze, maze.start);
    reached.len() == maze.grid.width * maze.grid.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::gen::{generate_dfs, generate_kruskal, generate_prim};

    #[test]
    fn kruskal_is_fully_connected() {
        let maze = generate_kruskal(10, 10, 12345);
        assert!(is_fully_connected(&maze));
    }

    #[test]
    fn prim_is_fully_connected() {
        let maze = generate_prim(10, 10, 12345);
        assert!(is_fully_connected(&maze));
    }

    #[test]
    fn dfs_is_fully_connected() {
        let maze = generate_dfs(10, 10, 12345);
        assert!(is_fully_connected(&maze));
    }
}
