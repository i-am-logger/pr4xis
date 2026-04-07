/// The 6 faces of a Rubik's cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    U, // Up (top)
    D, // Down (bottom)
    F, // Front
    B, // Back
    L, // Left
    R, // Right
}

impl Face {
    pub fn all() -> [Face; 6] {
        [Face::U, Face::D, Face::F, Face::B, Face::L, Face::R]
    }

    /// The opposite face.
    pub fn opposite(&self) -> Face {
        match self {
            Face::U => Face::D,
            Face::D => Face::U,
            Face::F => Face::B,
            Face::B => Face::F,
            Face::L => Face::R,
            Face::R => Face::L,
        }
    }
}

/// The 6 colors (one per face in solved state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Yellow,
    Green,
    Blue,
    Red,
    Orange,
}

impl Color {
    /// Default color for each face in solved state.
    pub fn of_face(face: Face) -> Self {
        match face {
            Face::U => Color::White,
            Face::D => Color::Yellow,
            Face::F => Color::Green,
            Face::B => Color::Blue,
            Face::L => Color::Orange,
            Face::R => Color::Red,
        }
    }
}
