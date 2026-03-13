use serde::{Deserialize, Serialize};

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
}
