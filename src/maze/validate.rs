use std::collections::{HashSet, VecDeque};

use super::{neighbors_all, Cell, Maze};

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

pub fn walls_are_symmetric(maze: &Maze) -> bool {
    for cell in maze.grid.cells() {
        for n in neighbors_all(cell, maze.grid.width, maze.grid.height) {
            if maze.walls.has_wall(cell, n) != maze.walls.has_wall(n, cell) {
                return false;
            }
        }
    }
    true
}

pub fn start_reachable_from_goal(maze: &Maze) -> bool {
    bfs_reachable(maze, maze.start).contains(&maze.goal)
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

    #[test]
    fn generated_mazes_have_symmetric_walls() {
        let kruskal = generate_kruskal(10, 10, 12345);
        let prim = generate_prim(10, 10, 12345);
        let dfs = generate_dfs(10, 10, 12345);
        assert!(walls_are_symmetric(&kruskal));
        assert!(walls_are_symmetric(&prim));
        assert!(walls_are_symmetric(&dfs));
    }

    #[test]
    fn direct_has_wall_symmetry_sanity() {
        let maze = generate_kruskal(5, 5, 99);
        for cell in maze.grid.cells() {
            for n in neighbors_all(cell, maze.grid.width, maze.grid.height) {
                assert_eq!(maze.walls.has_wall(cell, n), maze.walls.has_wall(n, cell));
            }
        }
    }

    #[test]
    fn goal_reachable_in_generated_maze() {
        let maze = generate_kruskal(10, 10, 12345);
        assert!(start_reachable_from_goal(&maze));
    }

    #[test]
    fn goal_unreachable_in_fully_walled_maze() {
        let maze = Maze::with_all_walls(3, 3);
        assert!(!start_reachable_from_goal(&maze));
    }

    #[test]
    fn goal_unreachable_when_only_start_component_is_open() {
        let mut maze = Maze::with_all_walls(3, 3);
        // Open a tiny component near start, keep goal isolated.
        maze.walls.remove_wall(Cell::new(0, 0), Cell::new(1, 0));
        maze.walls.remove_wall(Cell::new(1, 0), Cell::new(1, 1));
        assert!(!start_reachable_from_goal(&maze));
    }
}
