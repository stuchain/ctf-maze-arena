use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Cell coordinate. (row, col) or (y, x) — pick one and stick to it.
/// Using (x, y) for consistency with JSON [x,y] arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// Flat grid: index = y * width + x. Dimensions (width, height).
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    /// Optional: store cell data. For now, just dimensions.
    _data: Vec<()>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            _data: vec![(); width * height],
        }
    }

    pub fn index(&self, cell: Cell) -> Option<usize> {
        if cell.x < self.width && cell.y < self.height {
            Some(cell.y * self.width + cell.x)
        } else {
            None
        }
    }

    pub fn cell_from_index(&self, i: usize) -> Option<Cell> {
        if i < self.width * self.height {
            Some(Cell {
                x: i % self.width,
                y: i / self.width,
            })
        } else {
            None
        }
    }

    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.width * self.height).filter_map(move |i| self.cell_from_index(i))
    }
}

/// Directed edge (a, b) means wall between a and b.
/// Normalize: always store (min_cell, max_cell) so (A,B) and (B,A) map to same key.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Edge(pub Cell, pub Cell);

impl Edge {
    pub fn normalized(a: Cell, b: Cell) -> Self {
        let (min, max) = if (a.x, a.y) < (b.x, b.y) {
            (a, b)
        } else {
            (b, a)
        };
        Edge(min, max)
    }
}

/// Walls: set of edges. No wall = passage.
#[derive(Debug, Clone, Default)]
pub struct Walls {
    inner: HashSet<Edge>,
}

impl Walls {
    pub fn new() -> Self {
        Self { inner: HashSet::new() }
    }

    pub fn has_wall(&self, a: Cell, b: Cell) -> bool {
        self.inner.contains(&Edge::normalized(a, b))
    }

    pub fn set_wall(&mut self, a: Cell, b: Cell, present: bool) {
        let e = Edge::normalized(a, b);
        if present {
            self.inner.insert(e);
        } else {
            self.inner.remove(&e);
        }
    }

    pub fn remove_wall(&mut self, a: Cell, b: Cell) {
        self.set_wall(a, b, false);
    }

    pub fn wall_count(&self) -> usize {
        self.inner.len()
    }

    /// All possible edges in a grid (for Kruskal: shuffle these).
    pub fn all_edges(width: usize, height: usize) -> Vec<Edge> {
        let mut edges = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let c = Cell::new(x, y);
                if x + 1 < width {
                    edges.push(Edge::normalized(c, Cell::new(x + 1, y)));
                }
                if y + 1 < height {
                    edges.push(Edge::normalized(c, Cell::new(x, y + 1)));
                }
            }
        }
        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_index_roundtrip() {
        let g = Grid::new(5, 5);
        let c = Cell::new(2, 3);
        let i = g.index(c).unwrap();
        assert_eq!(g.cell_from_index(i), Some(c));
    }

    #[test]
    fn walls_all_edges_2x2() {
        let edges = Walls::all_edges(2, 2);
        // 2×2 grid: 2 horizontal edges per row (2 rows) + 2 vertical edges per col (2 cols) = 2*2 + 2*2 = 4? No.
        // For each (x,y): east edge if x+1<2, south edge if y+1<2.
        // (0,0): east (0,0)-(1,0), south (0,0)-(0,1). (1,0): south (1,0)-(1,1). (0,1): east (0,1)-(1,1). (1,1): none.
        // So 4 edges total.
        assert_eq!(edges.len(), 4);
    }

    #[test]
    fn walls_has_wall_symmetric() {
        let mut w = Walls::new();
        w.set_wall(Cell::new(0, 0), Cell::new(1, 0), true);
        assert!(w.has_wall(Cell::new(0, 0), Cell::new(1, 0)));
        assert!(w.has_wall(Cell::new(1, 0), Cell::new(0, 0)));
    }
}
