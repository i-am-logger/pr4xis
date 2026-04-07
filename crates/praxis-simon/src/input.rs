use crate::color::SimonColor;

/// A player input: pressing a color button at a specific position in the sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    pub color: SimonColor,
    pub position: usize, // which position in the sequence (0-indexed)
}

impl Input {
    pub fn new(color: SimonColor, position: usize) -> Self {
        Self { color, position }
    }
}
