#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::interval::Interval;
use pr4xis::category::{Concept, FinitelyGenerated};
use pr4xis::ontology::Quality;

// Note pitch classes (0-11) are the entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PitchClass(pub u8);

impl Concept for PitchClass {}
impl FinitelyGenerated for PitchClass {
    fn variants() -> Vec<Self> {
        (0..12).map(PitchClass).collect()
    }
}

/// The 12 pitch-class names of Western equal temperament, spelled with
/// sharps (integer pitch-class notation 0=C..11=B) — a closed taxonomy,
/// not a formatted string.
///
/// Source: Forte, A. (1973) *The Structure of Atonal Music*, Yale
/// University Press — the pitch-class integer notation (0=C, 1=C♯, ...,
/// 11=B) this enum mirrors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PitchName {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

/// Quality: note name for a pitch class.
#[derive(Debug, Clone)]
pub struct NoteName;

impl Quality for NoteName {
    type Individual = PitchClass;
    type Value = PitchName;

    fn get(&self, pc: &PitchClass) -> Option<PitchName> {
        Some(match pc.0 % 12 {
            0 => PitchName::C,
            1 => PitchName::CSharp,
            2 => PitchName::D,
            3 => PitchName::DSharp,
            4 => PitchName::E,
            5 => PitchName::F,
            6 => PitchName::FSharp,
            7 => PitchName::G,
            8 => PitchName::GSharp,
            9 => PitchName::A,
            10 => PitchName::ASharp,
            _ => PitchName::B,
        })
    }
}

/// Quality: is this pitch class a natural (no sharp/flat)?
#[derive(Debug, Clone)]
pub struct IsNatural;

impl Quality for IsNatural {
    type Individual = PitchClass;
    type Value = bool;

    fn get(&self, pc: &PitchClass) -> Option<bool> {
        Some(matches!(pc.0, 0 | 2 | 4 | 5 | 7 | 9 | 11)) // C D E F G A B
    }
}

/// Quality: is this interval consonant?
#[derive(Debug, Clone)]
pub struct IsConsonant;

impl Quality for IsConsonant {
    type Individual = PitchClass;
    type Value = bool;

    fn get(&self, pc: &PitchClass) -> Option<bool> {
        Some(Interval(pc.0).is_consonant())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_12_pitch_classes() {
        assert_eq!(PitchClass::variants().len(), 12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_note_name_quality() {
        let quality = NoteName;
        assert_eq!(quality.get(&PitchClass(0)), Some(PitchName::C));
        assert_eq!(quality.get(&PitchClass(9)), Some(PitchName::A));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_natural_quality() {
        let quality = IsNatural;
        assert_eq!(quality.get(&PitchClass(0)), Some(true)); // C
        assert_eq!(quality.get(&PitchClass(1)), Some(false)); // C#
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_consonant_quality() {
        let quality = IsConsonant;
        assert_eq!(quality.get(&PitchClass(7)), Some(true)); // perfect fifth
        assert_eq!(quality.get(&PitchClass(6)), Some(false)); // tritone
    }
}
