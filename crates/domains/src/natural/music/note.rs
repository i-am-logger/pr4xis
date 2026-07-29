#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// A note represented as a MIDI number (0-127).
/// Middle C = 60, A4 = 69 (440Hz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Note(pub u8);

impl Note {
    pub const C4: Self = Note(60);
    pub const D4: Self = Note(62);
    pub const E4: Self = Note(64);
    pub const F4: Self = Note(65);
    pub const G4: Self = Note(67);
    pub const A4: Self = Note(69);
    pub const B4: Self = Note(71);

    /// Pitch class (0-11): C=0, C#=1, D=2, ... B=11. A dimensionless count,
    /// not a physical quantity — see [`Note::octave`] for the same
    /// reasoning.
    pub fn pitch_class(&self) -> Quantity {
        Quantity::from_unit((self.0 % 12) as f64, &unit::UNITLESS)
    }

    /// Octave number (-1 to 9 in standard MIDI). A dimensionless count, not
    /// a physical quantity — carried as `Quantity` (unit `UNITLESS`) so it
    /// composes with the rest of the codebase's typed-quantity boundary,
    /// matching `formal::mereology::counting::ontology::cardinality`'s
    /// precedent for dimensionless counts.
    pub fn octave(&self) -> Quantity {
        Quantity::from_unit(((self.0 / 12) as i8 - 1) as f64, &unit::UNITLESS)
    }

    /// Transpose by semitones. Returns None if out of MIDI range.
    pub fn transpose(&self, semitones: i16) -> Option<Note> {
        // Widen to i32 so the add cannot overflow before the range test.
        let new = self.0 as i32 + semitones as i32;
        if (0..=127).contains(&new) {
            Some(Note(new as u8))
        } else {
            None
        }
    }

    /// Name of the pitch class.
    pub fn name(&self) -> &'static str {
        match self.pitch_class().value as u8 {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            n => panic!("pitch_class returned {n}, expected 0..=11"),
        }
    }

    /// Distance in semitones to another note. A dimensionless interval
    /// count — see [`Note::octave`] for the same reasoning.
    pub fn distance_to(&self, other: Note) -> Quantity {
        Quantity::from_unit((other.0 as i16 - self.0 as i16) as f64, &unit::UNITLESS)
    }

    /// Are these notes enharmonic (same pitch class)?
    pub fn is_enharmonic(&self, other: Note) -> bool {
        self.pitch_class() == other.pitch_class()
    }
}
