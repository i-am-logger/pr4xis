use crate::interval::Interval;
use crate::note::Note;
use praxis_category::Entity;
use praxis_ontology::Quality;

// Note pitch classes (0-11) are the entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PitchClass(pub u8);

impl Entity for PitchClass {
    fn variants() -> Vec<Self> {
        (0..12).map(PitchClass).collect()
    }
}

/// Quality: note name for a pitch class.
#[derive(Debug, Clone)]
pub struct NoteName;

impl Quality for NoteName {
    type Individual = PitchClass;
    type Value = String;

    fn get(&self, pc: &PitchClass) -> Option<String> {
        Some(Note(pc.0 + 60).name().to_string()) // use octave 4 for naming
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

    #[test]
    fn test_12_pitch_classes() {
        assert_eq!(PitchClass::variants().len(), 12);
    }

    #[test]
    fn test_note_name_quality() {
        let quality = NoteName;
        assert_eq!(quality.get(&PitchClass(0)), Some("C".to_string()));
        assert_eq!(quality.get(&PitchClass(9)), Some("A".to_string()));
    }

    #[test]
    fn test_natural_quality() {
        let quality = IsNatural;
        assert_eq!(quality.get(&PitchClass(0)), Some(true)); // C
        assert_eq!(quality.get(&PitchClass(1)), Some(false)); // C#
    }

    #[test]
    fn test_consonant_quality() {
        let quality = IsConsonant;
        assert_eq!(quality.get(&PitchClass(7)), Some(true)); // perfect fifth
        assert_eq!(quality.get(&PitchClass(6)), Some(false)); // tritone
    }
}
