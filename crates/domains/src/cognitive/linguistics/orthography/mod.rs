#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod channel;
pub mod channel_communication_functor;
pub mod distance;

use super::symbols::character::{Direction, Script};
use super::symbols::numeral::NumeralSystem;
use super::symbols::punctuation::PunctuationMark;

/// A writing system — how a language is written.
///
/// Binds a script (characters), numeral system, punctuation rules,
/// and writing direction into a complete orthographic system.
#[derive(Debug, Clone)]
pub struct WritingSystem {
    pub name: String,
    pub script: Script,
    pub numerals: NumeralSystem,
    pub punctuation: Vec<PunctuationMark>,
    pub direction: Direction,
}

impl WritingSystem {
    pub fn new(name: &str, script: Script, numerals: NumeralSystem) -> Self {
        let direction = script.direction;
        Self {
            name: name.into(),
            script,
            numerals,
            punctuation: Vec::new(),
            direction,
        }
    }

    pub fn with_punctuation(mut self, marks: Vec<PunctuationMark>) -> Self {
        self.punctuation = marks;
        self
    }

    /// Is a character part of this writing system?
    pub fn recognizes(&self, c: char) -> bool {
        self.script.contains(c)
            || self.numerals.contains(c)
            || self.punctuation.iter().any(|p| p.character == c)
    }
}

/// English writing system: Latin script, Arabic numerals, LTR.
pub fn english_writing_system() -> WritingSystem {
    use super::symbols::{character, numeral, punctuation};

    WritingSystem::new("English", character::latin(), numeral::arabic())
        .with_punctuation(punctuation::standard_punctuation())
}

/// Hebrew writing system: Hebrew script, Arabic numerals, RTL.
pub fn hebrew_writing_system() -> WritingSystem {
    use super::symbols::{character, numeral, punctuation};

    WritingSystem::new("Hebrew", character::hebrew(), numeral::arabic())
        .with_punctuation(punctuation::standard_punctuation())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_recognizes_latin() {
        let ws = english_writing_system();
        assert!(ws.recognizes('a'));
        assert!(ws.recognizes('Z'));
        assert!(ws.recognizes('5'));
        assert!(ws.recognizes('.'));
        assert!(!ws.recognizes('\u{05D0}')); // aleph
    }

    #[test]
    fn english_is_ltr() {
        let ws = english_writing_system();
        assert_eq!(ws.direction, Direction::LeftToRight);
    }

    #[test]
    fn hebrew_recognizes_hebrew() {
        let ws = hebrew_writing_system();
        assert!(ws.recognizes('\u{05D0}')); // aleph
        assert!(ws.recognizes('5')); // Arabic numerals shared
        assert!(!ws.recognizes('a')); // Latin not in Hebrew
    }

    #[test]
    fn hebrew_is_rtl() {
        let ws = hebrew_writing_system();
        assert_eq!(ws.direction, Direction::RightToLeft);
    }
}
