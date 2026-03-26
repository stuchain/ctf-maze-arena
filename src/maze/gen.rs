use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use super::{Cell, Edge, Maze, Walls};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kruskal_neighbors_within_expected_bounds() {
        let maze = generate_kruskal(10, 10, 42);
        for cell in maze.grid.cells() {
            let degree = maze.neighbors(cell).len();
            assert!((1..=4).contains(&degree));
        }
    }

    #[test]
    fn kruskal_spanning_tree_invariant() {
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
}
