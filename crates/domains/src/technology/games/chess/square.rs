use praxis::category::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub file: u8, // 0-7 (a-h)
    pub rank: u8, // 0-7 (1-8)
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        assert!(file < 8 && rank < 8, "square out of bounds");
        Self { file, rank }
    }

    pub fn name(&self) -> String {
        let f = (b'a' + self.file) as char;
        let r = (b'1' + self.rank) as char;
        format!("{}{}", f, r)
    }

    /// Try to offset by (df, dr). Returns None if out of bounds.
    pub fn offset(&self, df: i8, dr: i8) -> Option<Self> {
        let f = self.file as i8 + df;
        let r = self.rank as i8 + dr;
        if (0..8).contains(&f) && (0..8).contains(&r) {
            Some(Self::new(f as u8, r as u8))
        } else {
            None
        }
    }
}

impl Entity for Square {
    fn variants() -> Vec<Self> {
        let mut v = Vec::with_capacity(64);
        for file in 0..8u8 {
            for rank in 0..8u8 {
                v.push(Square::new(file, rank));
            }
        }
        v
    }
}
