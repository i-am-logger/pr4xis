#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

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

    /// Pitch class (0-11): C=0, C#=1, D=2, ... B=11.
    pub fn pitch_class(&self) -> u8 {
        self.0 % 12
    }

    /// Octave number (-1 to 9 in standard MIDI).
    pub fn octave(&self) -> i8 {
        (self.0 / 12) as i8 - 1
    }

    /// Transpose by semitones. Returns None if out of MIDI range.
    pub fn transpose(&self, semitones: i16) -> Option<Note> {
        let new = self.0 as i16 + semitones;
        if (0..=127).contains(&new) {
            Some(Note(new as u8))
        } else {
            None
        }
    }

    /// Name of the pitch class.
    pub fn name(&self) -> &'static str {
        match self.pitch_class() {
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

    /// Distance in semitones to another note.
    pub fn distance_to(&self, other: Note) -> i16 {
        other.0 as i16 - self.0 as i16
    }

    /// Are these notes enharmonic (same pitch class)?
    pub fn is_enharmonic(&self, other: Note) -> bool {
        self.pitch_class() == other.pitch_class()
    }
}
