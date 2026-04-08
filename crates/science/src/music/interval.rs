/// A musical interval (distance in semitones).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Interval(pub u8);

impl Interval {
    pub const UNISON: Self = Interval(0);
    pub const MINOR_SECOND: Self = Interval(1);
    pub const MAJOR_SECOND: Self = Interval(2);
    pub const MINOR_THIRD: Self = Interval(3);
    pub const MAJOR_THIRD: Self = Interval(4);
    pub const PERFECT_FOURTH: Self = Interval(5);
    pub const TRITONE: Self = Interval(6);
    pub const PERFECT_FIFTH: Self = Interval(7);
    pub const MINOR_SIXTH: Self = Interval(8);
    pub const MAJOR_SIXTH: Self = Interval(9);
    pub const MINOR_SEVENTH: Self = Interval(10);
    pub const MAJOR_SEVENTH: Self = Interval(11);
    pub const OCTAVE: Self = Interval(12);

    /// Compose two intervals (add semitones).
    pub fn compose(&self, other: Interval) -> Interval {
        Interval(self.0 + other.0)
    }

    /// Inversion: the complement to an octave.
    pub fn invert(&self) -> Interval {
        if self.0 <= 12 {
            Interval(12 - self.0)
        } else {
            Interval(self.0) // no inversion beyond octave
        }
    }

    /// Is this a consonant interval?
    pub fn is_consonant(&self) -> bool {
        matches!(self.0 % 12, 0 | 3 | 4 | 5 | 7 | 8 | 9 | 12)
    }

    /// Is this a dissonant interval?
    pub fn is_dissonant(&self) -> bool {
        !self.is_consonant()
    }

    pub fn name(&self) -> &'static str {
        match self.0 % 12 {
            0 => "unison/octave",
            1 => "minor 2nd",
            2 => "major 2nd",
            3 => "minor 3rd",
            4 => "major 3rd",
            5 => "perfect 4th",
            6 => "tritone",
            7 => "perfect 5th",
            8 => "minor 6th",
            9 => "major 6th",
            10 => "minor 7th",
            11 => "major 7th",
            _ => unreachable!(),
        }
    }
}
