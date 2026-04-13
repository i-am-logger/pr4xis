use pr4xis::category::Entity;

/// The 7 standard Tetris tetrominoes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PieceKind {
    I, // ████
    O, // ██
    // ██
    T, //  █
    // ███
    S, //  ██
    // ██
    Z, // ██
    //  ██
    J, // █
    // ███
    L, //   █
       // ███
}

impl PieceKind {
    pub fn all() -> [PieceKind; 7] {
        [
            PieceKind::I,
            PieceKind::O,
            PieceKind::T,
            PieceKind::S,
            PieceKind::Z,
            PieceKind::J,
            PieceKind::L,
        ]
    }
}

/// Rotation state (0°, 90°, 180°, 270°).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    pub fn cw(&self) -> Self {
        match self {
            Rotation::R0 => Rotation::R90,
            Rotation::R90 => Rotation::R180,
            Rotation::R180 => Rotation::R270,
            Rotation::R270 => Rotation::R0,
        }
    }

    pub fn ccw(&self) -> Self {
        match self {
            Rotation::R0 => Rotation::R270,
            Rotation::R90 => Rotation::R0,
            Rotation::R180 => Rotation::R90,
            Rotation::R270 => Rotation::R180,
        }
    }
}

/// A piece with position and rotation on the board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub rotation: Rotation,
    pub x: i32, // column (can be negative during wall kicks)
    pub y: i32, // row (0 = bottom)
}

impl Piece {
    pub fn new(kind: PieceKind) -> Self {
        Self {
            kind,
            rotation: Rotation::R0,
            x: 3,  // center of 10-wide board
            y: 18, // near top of 20-high board
        }
    }

    /// The cells this piece occupies (relative offsets from origin).
    pub fn cells(&self) -> Vec<(i32, i32)> {
        let base = base_cells(self.kind);
        let rotated: Vec<(i32, i32)> = base
            .iter()
            .map(|&(dx, dy)| match self.rotation {
                Rotation::R0 => (dx, dy),
                Rotation::R90 => (dy, -dx),
                Rotation::R180 => (-dx, -dy),
                Rotation::R270 => (-dy, dx),
            })
            .collect();
        rotated
            .iter()
            .map(|&(dx, dy)| (self.x + dx, self.y + dy))
            .collect()
    }

    pub fn moved(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self.clone()
        }
    }

    pub fn rotated_cw(&self) -> Self {
        Self {
            rotation: self.rotation.cw(),
            ..self.clone()
        }
    }

    pub fn rotated_ccw(&self) -> Self {
        Self {
            rotation: self.rotation.ccw(),
            ..self.clone()
        }
    }
}

/// Base cell offsets for each piece kind (rotation R0).
fn base_cells(kind: PieceKind) -> [(i32, i32); 4] {
    match kind {
        PieceKind::I => [(0, 0), (1, 0), (2, 0), (3, 0)],
        PieceKind::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
        PieceKind::T => [(0, 0), (1, 0), (2, 0), (1, 1)],
        PieceKind::S => [(0, 0), (1, 0), (1, 1), (2, 1)],
        PieceKind::Z => [(1, 0), (2, 0), (0, 1), (1, 1)],
        PieceKind::J => [(0, 0), (0, 1), (1, 0), (2, 0)],
        PieceKind::L => [(0, 0), (1, 0), (2, 0), (2, 1)],
    }
}
