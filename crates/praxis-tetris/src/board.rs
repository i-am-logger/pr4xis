use crate::piece::Piece;

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;

/// The Tetris board: a grid of filled/empty cells.
/// The ontology enforces: no overlaps, no out-of-bounds, gravity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// cells[row][col] — true = filled
    cells: [[bool; WIDTH]; HEIGHT],
    pub lines_cleared: u32,
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[false; WIDTH]; HEIGHT],
            lines_cleared: 0,
        }
    }

    pub fn is_filled(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= WIDTH as i32 || y < 0 || y >= HEIGHT as i32 {
            return true; // walls and floor are "filled" (blocks movement)
        }
        self.cells[y as usize][x as usize]
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        !self.is_filled(x, y)
    }

    /// Check if a piece fits at its current position (no collisions).
    pub fn piece_fits(&self, piece: &Piece) -> bool {
        piece.cells().iter().all(|&(x, y)| {
            x >= 0
                && x < WIDTH as i32
                && y >= 0
                && y < HEIGHT as i32
                && !self.cells[y as usize][x as usize]
        })
    }

    /// Lock a piece onto the board. Returns Err if piece doesn't fit.
    pub fn lock_piece(&mut self, piece: &Piece) -> Result<(), &'static str> {
        if !self.piece_fits(piece) {
            return Err("piece does not fit — cannot lock");
        }
        for (x, y) in piece.cells() {
            self.cells[y as usize][x as usize] = true;
        }
        Ok(())
    }

    /// Clear any full lines. Returns number of lines cleared.
    pub fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0u32;
        let mut new_cells = [[false; WIDTH]; HEIGHT];
        let mut write_row = 0;

        for row in 0..HEIGHT {
            if !self.cells[row].iter().all(|&c| c) {
                new_cells[write_row] = self.cells[row];
                write_row += 1;
            } else {
                cleared += 1;
            }
        }

        self.cells = new_cells;
        self.lines_cleared += cleared;
        cleared
    }

    /// Check if a row is completely filled.
    pub fn row_full(&self, row: usize) -> bool {
        row < HEIGHT && self.cells[row].iter().all(|&c| c)
    }

    /// Count total filled cells.
    pub fn filled_count(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c)
            .count()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
