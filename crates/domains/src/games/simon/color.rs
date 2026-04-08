/// The four Simon Says colors/buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimonColor {
    Red,
    Blue,
    Green,
    Yellow,
}

impl SimonColor {
    pub fn all() -> [SimonColor; 4] {
        [
            SimonColor::Red,
            SimonColor::Blue,
            SimonColor::Green,
            SimonColor::Yellow,
        ]
    }

    /// Generate a deterministic color from a seed (for reproducible games).
    pub fn from_seed(seed: u64) -> Self {
        SimonColor::all()[(seed % 4) as usize]
    }
}
