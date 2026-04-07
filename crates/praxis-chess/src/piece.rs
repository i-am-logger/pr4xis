use crate::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opponent(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Pawn forward direction: +1 for white, -1 for black.
    pub fn pawn_direction(&self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    /// Starting rank for pawns.
    pub fn pawn_rank(&self) -> u8 {
        match self {
            Color::White => 1,
            Color::Black => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color) -> Self {
        Self { kind, color }
    }

    /// Raw movement deltas for this piece kind (ignoring board state).
    /// For sliding pieces, returns direction vectors (repeated up to 7 times).
    /// For pawns, includes forward moves and diagonal captures.
    pub fn movement_deltas(&self) -> Vec<(i8, i8)> {
        match self.kind {
            PieceKind::King => vec![
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ],
            PieceKind::Knight => vec![
                (-2, -1),
                (-2, 1),
                (-1, -2),
                (-1, 2),
                (1, -2),
                (1, 2),
                (2, -1),
                (2, 1),
            ],
            PieceKind::Rook => Self::sliding_deltas(&[(1, 0), (-1, 0), (0, 1), (0, -1)]),
            PieceKind::Bishop => Self::sliding_deltas(&[(1, 1), (1, -1), (-1, 1), (-1, -1)]),
            PieceKind::Queen => Self::sliding_deltas(&[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ]),
            PieceKind::Pawn => {
                let d = self.color.pawn_direction();
                vec![(0, d), (-1, d), (1, d), (0, 2 * d)]
            }
        }
    }

    fn sliding_deltas(directions: &[(i8, i8)]) -> Vec<(i8, i8)> {
        let mut deltas = Vec::new();
        for &(df, dr) in directions {
            for dist in 1..8i8 {
                deltas.push((df * dist, dr * dist));
            }
        }
        deltas
    }

    /// Squares this piece could reach from `from`, ignoring other pieces.
    pub fn reachable_squares(&self, from: Square) -> Vec<Square> {
        self.movement_deltas()
            .iter()
            .filter_map(|&(df, dr)| from.offset(df, dr))
            .collect()
    }
}
