#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::note::Note;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// Scale kinds with their interval patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleKind {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Pentatonic,
    Blues,
    Chromatic,
    WholeTone,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
}

impl ScaleKind {
    /// The interval pattern (semitone steps between consecutive notes).
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            ScaleKind::Major => vec![2, 2, 1, 2, 2, 2, 1],
            ScaleKind::NaturalMinor => vec![2, 1, 2, 2, 1, 2, 2],
            ScaleKind::HarmonicMinor => vec![2, 1, 2, 2, 1, 3, 1],
            ScaleKind::MelodicMinor => vec![2, 1, 2, 2, 2, 2, 1],
            ScaleKind::Pentatonic => vec![2, 2, 3, 2, 3],
            ScaleKind::Blues => vec![3, 2, 1, 1, 3, 2],
            ScaleKind::Chromatic => vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ScaleKind::WholeTone => vec![2, 2, 2, 2, 2, 2],
            ScaleKind::Dorian => vec![2, 1, 2, 2, 2, 1, 2],
            ScaleKind::Phrygian => vec![1, 2, 2, 2, 1, 2, 2],
            ScaleKind::Lydian => vec![2, 2, 2, 1, 2, 2, 1],
            ScaleKind::Mixolydian => vec![2, 2, 1, 2, 2, 1, 2],
        }
    }

    /// All scale kinds.
    pub fn all() -> Vec<ScaleKind> {
        vec![
            ScaleKind::Major,
            ScaleKind::NaturalMinor,
            ScaleKind::HarmonicMinor,
            ScaleKind::MelodicMinor,
            ScaleKind::Pentatonic,
            ScaleKind::Blues,
            ScaleKind::Chromatic,
            ScaleKind::WholeTone,
            ScaleKind::Dorian,
            ScaleKind::Phrygian,
            ScaleKind::Lydian,
            ScaleKind::Mixolydian,
        ]
    }
}

/// A scale: root note + scale kind.
#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    pub root: Note,
    pub kind: ScaleKind,
}

impl Scale {
    pub fn new(root: Note, kind: ScaleKind) -> Self {
        Self { root, kind }
    }

    /// Generate all notes in this scale (one octave from root).
    pub fn notes(&self) -> Vec<Note> {
        let mut notes = vec![self.root];
        let mut current = self.root;
        for &step in &self.kind.intervals() {
            if let Some(next) = current.transpose(step as i16) {
                notes.push(next);
                current = next;
            } else {
                break;
            }
        }
        notes
    }

    /// Is a given note in this scale?
    pub fn contains(&self, note: Note) -> bool {
        let scale_pcs: Vec<u8> = self
            .notes()
            .iter()
            .map(|n| n.pitch_class().value as u8)
            .collect();
        scale_pcs.contains(&(note.pitch_class().value as u8))
    }

    /// Total semitones in the scale (should be 12 for a complete octave scale).
    /// A dimensionless semitone count — see [`super::note::Note::octave`].
    pub fn total_semitones(&self) -> Quantity {
        let total: u8 = self.kind.intervals().iter().sum();
        Quantity::from_unit(total as f64, &unit::UNITLESS)
    }

    /// Number of notes in the scale (excluding the repeated octave). A
    /// dimensionless count — see [`super::note::Note::octave`].
    pub fn degree_count(&self) -> Quantity {
        Quantity::from_unit(self.kind.intervals().len() as f64, &unit::UNITLESS)
    }
}
