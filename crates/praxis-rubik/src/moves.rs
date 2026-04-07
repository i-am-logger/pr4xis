use crate::face::Face;

/// A Rubik's cube move: a face rotation.
/// Only 18 possible moves (6 faces × 3 rotations: CW, CCW, 180°).
/// This is the complete set — no other state changes are possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    /// Clockwise 90°
    U,
    D,
    F,
    B,
    L,
    R,
    /// Counter-clockwise 90° (prime moves)
    Ui,
    Di,
    Fi,
    Bi,
    Li,
    Ri,
    /// 180° (double moves)
    U2,
    D2,
    F2,
    B2,
    L2,
    R2,
}

impl Move {
    pub fn all() -> [Move; 18] {
        [
            Move::U,
            Move::D,
            Move::F,
            Move::B,
            Move::L,
            Move::R,
            Move::Ui,
            Move::Di,
            Move::Fi,
            Move::Bi,
            Move::Li,
            Move::Ri,
            Move::U2,
            Move::D2,
            Move::F2,
            Move::B2,
            Move::L2,
            Move::R2,
        ]
    }

    /// The face this move rotates.
    pub fn face(&self) -> Face {
        match self {
            Move::U | Move::Ui | Move::U2 => Face::U,
            Move::D | Move::Di | Move::D2 => Face::D,
            Move::F | Move::Fi | Move::F2 => Face::F,
            Move::B | Move::Bi | Move::B2 => Face::B,
            Move::L | Move::Li | Move::L2 => Face::L,
            Move::R | Move::Ri | Move::R2 => Face::R,
        }
    }

    /// Number of 90° CW rotations (1, 2, or 3 for CCW).
    pub fn rotation_count(&self) -> u8 {
        match self {
            Move::U | Move::D | Move::F | Move::B | Move::L | Move::R => 1,
            Move::U2 | Move::D2 | Move::F2 | Move::B2 | Move::L2 | Move::R2 => 2,
            Move::Ui | Move::Di | Move::Fi | Move::Bi | Move::Li | Move::Ri => 3,
        }
    }

    /// The inverse of this move.
    pub fn inverse(&self) -> Move {
        match self {
            Move::U => Move::Ui,
            Move::Ui => Move::U,
            Move::D => Move::Di,
            Move::Di => Move::D,
            Move::F => Move::Fi,
            Move::Fi => Move::F,
            Move::B => Move::Bi,
            Move::Bi => Move::B,
            Move::L => Move::Li,
            Move::Li => Move::L,
            Move::R => Move::Ri,
            Move::Ri => Move::R,
            Move::U2 => Move::U2,
            Move::D2 => Move::D2,
            Move::F2 => Move::F2,
            Move::B2 => Move::B2,
            Move::L2 => Move::L2,
            Move::R2 => Move::R2,
        }
    }

    /// Standard notation string.
    pub fn notation(&self) -> &'static str {
        match self {
            Move::U => "U",
            Move::D => "D",
            Move::F => "F",
            Move::B => "B",
            Move::L => "L",
            Move::R => "R",
            Move::Ui => "U'",
            Move::Di => "D'",
            Move::Fi => "F'",
            Move::Bi => "B'",
            Move::Li => "L'",
            Move::Ri => "R'",
            Move::U2 => "U2",
            Move::D2 => "D2",
            Move::F2 => "F2",
            Move::B2 => "B2",
            Move::L2 => "L2",
            Move::R2 => "R2",
        }
    }
}
