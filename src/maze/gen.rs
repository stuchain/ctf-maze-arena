use rand::rngs::StdRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::SeedableRng;
use std::collections::HashSet;

use super::{neighbors_all, Cell, Edge, Maze, Walls};

/// Union-Find (Disjoint Set Union) used by randomized Kruskal.
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] != i {
            self.parent[i] = self.find(self.parent[i]);
        }
        self.parent[i]
    }

    fn union(&mut self, a: usize, b: usize) -> bool {
        let (pa, pb) = (self.find(a), self.find(b));
        if pa == pb {
            return false;
        }
        self.parent[pa] = pb;
        true
    }
}

/// Generate a perfect maze using randomized Kruskal.
///
/// Starts with all walls present and removes a wall iff it joins
/// two previously disconnected components.
pub fn generate_kruskal(width: usize, height: usize, seed: u64) -> Maze {
    let mut maze = Maze::with_all_walls(width, height);
    let n = width * height;
    let mut uf = UnionFind::new(n);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut edges = Walls::all_edges(width, height);
    edges.shuffle(&mut rng);

    for edge in edges {
        let Edge(a, b) = edge;
        let ia = maze.grid.index(a).unwrap();
        let ib = maze.grid.index(b).unwrap();
        if uf.union(ia, ib) {
            maze.walls.remove_wall(a, b);
        }
    }

    maze
}

/// Generate a perfect maze using randomized Prim.
///
/// Starts with all walls present and grows a tree by repeatedly
/// selecting a random frontier edge that adds a new cell.
pub fn generate_prim(width: usize, height: usize, seed: u64) -> Maze {
    let mut maze = Maze::with_all_walls(width, height);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut in_tree = HashSet::new();
    let start = Cell::new(
        (0..width).choose(&mut rng).unwrap_or(0),
        (0..height).choose(&mut rng).unwrap_or(0),
    );
    in_tree.insert(start);

    // Frontier edges (from_tree, to_candidate).
    let mut frontier: Vec<(Cell, Cell)> = Vec::new();
    for n in neighbors_all(start, width, height) {
        frontier.push((start, n));
    }

    while !frontier.is_empty() {
        let idx = (0..frontier.len()).choose(&mut rng).unwrap();
        let (from, to) = frontier.swap_remove(idx);
        if in_tree.contains(&to) {
            continue;
        }
        maze.walls.remove_wall(from, to);
        in_tree.insert(to);
        for n in neighbors_all(to, width, height) {
            if !in_tree.contains(&n) {
                frontier.push((to, n));
            }
        }
    }

    maze
}

/// Generate a perfect maze using iterative DFS backtracker.
///
/// Starts with all walls present, then performs randomized DFS and
/// removes a wall whenever moving to a previously unvisited neighbor.
pub fn generate_dfs(width: usize, height: usize, seed: u64) -> Maze {
    let mut maze = Maze::with_all_walls(width, height);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut visited = HashSet::new();
    let mut stack = vec![Cell::new(0, 0)];

    while let Some(cell) = stack.pop() {
        if !visited.insert(cell) {
            continue;
        }
        let mut neighbors = neighbors_all(cell, width, height);
        neighbors.shuffle(&mut rng);
        for next in neighbors {
            if !visited.contains(&next) {
                maze.walls.remove_wall(cell, next);
                stack.push(next);
            }
        }
    }

    maze
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::validate::is_fully_connected;

    #[test]
    fn kruskal_neighbors_within_expected_bounds() {
        let maze = generate_kruskal(10, 10, 42);
        for cell in maze.grid.cells() {
            let degree = maze.neighbors(cell).len();
            assert!((1..=4).contains(&degree));
        }
    }

    #[test]
    fn kruskal_connectivity() {
        let maze = generate_kruskal(10, 10, 12345);
        assert!(is_fully_connected(&maze), "all cells reachable from start");
        assert!(maze.neighbors(maze.start).len() >= 1);
    }

    #[test]
    fn kruskal_spanning_tree() {
        let width = 5;
        let height = 5;
        let maze = generate_kruskal(width, height, 999);
        let total_possible_edges = Walls::all_edges(width, height).len();
        let passages = total_possible_edges - maze.walls.wall_count();
        assert_eq!(passages, width * height - 1);
    }

    #[test]
    fn kruskal_same_seed_produces_same_structure() {
        let a = generate_kruskal(8, 8, 12345);
        let b = generate_kruskal(8, 8, 12345);
        for y in 0..a.grid.height {
            for x in 0..a.grid.width {
                let c = Cell::new(x, y);
                if x + 1 < a.grid.width {
                    let right = Cell::new(x + 1, y);
                    assert_eq!(a.walls.has_wall(c, right), b.walls.has_wall(c, right));
                }
                if y + 1 < a.grid.height {
                    let down = Cell::new(x, y + 1);
                    assert_eq!(a.walls.has_wall(c, down), b.walls.has_wall(c, down));
                }
            }
        }
    }

    #[test]
    fn prim_connectivity() {
        let maze = generate_prim(10, 10, 12345);
        assert!(is_fully_connected(&maze), "all cells reachable from start");
        assert!(maze.neighbors(maze.start).len() >= 1);
    }

    #[test]
    fn prim_spanning_tree() {
        let width = 5;
        let height = 5;
        let maze = generate_prim(width, height, 999);
        let total_possible_edges = Walls::all_edges(width, height).len();
        let passages = total_possible_edges - maze.walls.wall_count();
        assert_eq!(passages, width * height - 1);
    }

    #[test]
    fn prim_same_seed_produces_same_structure() {
        let a = generate_prim(8, 8, 12345);
        let b = generate_prim(8, 8, 12345);
        for y in 0..a.grid.height {
            for x in 0..a.grid.width {
                let c = Cell::new(x, y);
                if x + 1 < a.grid.width {
                    let right = Cell::new(x + 1, y);
                    assert_eq!(a.walls.has_wall(c, right), b.walls.has_wall(c, right));
                }
                if y + 1 < a.grid.height {
                    let down = Cell::new(x, y + 1);
                    assert_eq!(a.walls.has_wall(c, down), b.walls.has_wall(c, down));
                }
            }
        }
    }

    #[test]
    fn dfs_spanning_tree() {
        let width = 5;
        let height = 5;
        let maze = generate_dfs(width, height, 999);
        let total_possible_edges = Walls::all_edges(width, height).len();
        let passages = total_possible_edges - maze.walls.wall_count();
        assert_eq!(passages, width * height - 1);
    }

    #[test]
    fn dfs_same_seed_produces_same_structure() {
        let a = generate_dfs(8, 8, 12345);
        let b = generate_dfs(8, 8, 12345);
        for y in 0..a.grid.height {
            for x in 0..a.grid.width {
                let c = Cell::new(x, y);
                if x + 1 < a.grid.width {
                    let right = Cell::new(x + 1, y);
                    assert_eq!(a.walls.has_wall(c, right), b.walls.has_wall(c, right));
                }
                if y + 1 < a.grid.height {
                    let down = Cell::new(x, y + 1);
                    assert_eq!(a.walls.has_wall(c, down), b.walls.has_wall(c, down));
                }
            }
        }
    }
}
