use crate::interval::Interval;
use crate::note::Note;
use crate::scale::Scale;

/// Chord kinds with their interval structures (from root).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordKind {
    Major,       // root, M3, P5
    Minor,       // root, m3, P5
    Diminished,  // root, m3, tritone
    Augmented,   // root, M3, m6
    Major7,      // root, M3, P5, M7
    Minor7,      // root, m3, P5, m7
    Dominant7,   // root, M3, P5, m7
    Diminished7, // root, m3, tritone, M6
    Sus2,        // root, M2, P5
    Sus4,        // root, P4, P5
}

impl ChordKind {
    /// Intervals from root for each chord tone.
    pub fn intervals(&self) -> Vec<Interval> {
        match self {
            ChordKind::Major => vec![Interval(0), Interval(4), Interval(7)],
            ChordKind::Minor => vec![Interval(0), Interval(3), Interval(7)],
            ChordKind::Diminished => vec![Interval(0), Interval(3), Interval(6)],
            ChordKind::Augmented => vec![Interval(0), Interval(4), Interval(8)],
            ChordKind::Major7 => vec![Interval(0), Interval(4), Interval(7), Interval(11)],
            ChordKind::Minor7 => vec![Interval(0), Interval(3), Interval(7), Interval(10)],
            ChordKind::Dominant7 => vec![Interval(0), Interval(4), Interval(7), Interval(10)],
            ChordKind::Diminished7 => vec![Interval(0), Interval(3), Interval(6), Interval(9)],
            ChordKind::Sus2 => vec![Interval(0), Interval(2), Interval(7)],
            ChordKind::Sus4 => vec![Interval(0), Interval(5), Interval(7)],
        }
    }

    pub fn all() -> Vec<ChordKind> {
        vec![
            ChordKind::Major,
            ChordKind::Minor,
            ChordKind::Diminished,
            ChordKind::Augmented,
            ChordKind::Major7,
            ChordKind::Minor7,
            ChordKind::Dominant7,
            ChordKind::Diminished7,
            ChordKind::Sus2,
            ChordKind::Sus4,
        ]
    }
}

/// Result of chord validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChordResult {
    /// All notes are in the scale.
    Diatonic,
    /// Some notes are outside the scale (chromatic).
    Chromatic { out_of_scale: Vec<Note> },
}

/// A chord: root + kind.
#[derive(Debug, Clone)]
pub struct Chord {
    pub root: Note,
    pub kind: ChordKind,
}

impl Chord {
    pub fn new(root: Note, kind: ChordKind) -> Self {
        Self { root, kind }
    }

    /// Generate the notes of this chord.
    pub fn notes(&self) -> Vec<Note> {
        self.kind
            .intervals()
            .iter()
            .filter_map(|interval| self.root.transpose(interval.0 as i16))
            .collect()
    }

    /// Validate this chord against a scale.
    pub fn validate_against(&self, scale: &Scale) -> ChordResult {
        let out_of_scale: Vec<Note> = self
            .notes()
            .into_iter()
            .filter(|n| !scale.contains(*n))
            .collect();

        if out_of_scale.is_empty() {
            ChordResult::Diatonic
        } else {
            ChordResult::Chromatic { out_of_scale }
        }
    }

    /// Does this chord contain a tritone (dissonance)?
    pub fn has_tritone(&self) -> bool {
        let notes = self.notes();
        for i in 0..notes.len() {
            for j in i + 1..notes.len() {
                let interval = (notes[j].0 as i16 - notes[i].0 as i16).unsigned_abs() as u8 % 12;
                if interval == 6 {
                    return true;
                }
            }
        }
        false
    }

    /// Number of notes in the chord.
    pub fn note_count(&self) -> usize {
        self.kind.intervals().len()
    }

    pub fn name(&self) -> String {
        let kind_str = match self.kind {
            ChordKind::Major => "",
            ChordKind::Minor => "m",
            ChordKind::Diminished => "dim",
            ChordKind::Augmented => "aug",
            ChordKind::Major7 => "maj7",
            ChordKind::Minor7 => "m7",
            ChordKind::Dominant7 => "7",
            ChordKind::Diminished7 => "dim7",
            ChordKind::Sus2 => "sus2",
            ChordKind::Sus4 => "sus4",
        };
        format!("{}{}", self.root.name(), kind_str)
    }
}
