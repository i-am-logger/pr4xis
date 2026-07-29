#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod case_folding;
pub mod channel;
pub mod channel_communication_functor;
pub mod distance;
pub mod grapheme;

use super::symbols::character::{Direction, Script};
use super::symbols::numeral::NumeralSystem;
use super::symbols::punctuation::{PunctuationFunction, PunctuationMark};

/// A writing system — how a language is written.
///
/// Binds a script (characters), numeral system, punctuation rules,
/// and writing direction into a complete orthographic system.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Is `c` a loaded mark whose FUNCTION signals an INCOMPLETE, continued
    /// enumeration — [`PunctuationFunction::Dash`] ("indicates a range,
    /// pause, or interruption") or [`PunctuationFunction::Introducer`]
    /// ("introduces elaboration, definition, or a list. Creates an
    /// expectation of what follows")?
    ///
    /// The USLM legislative-drafting "sandwich" convention this detects: a
    /// chapeau paragraph deliberately ends its sentence in an em dash or
    /// colon ("the term X means—" / "means the following:"), with the
    /// definiens continuing in enumerated child subparagraphs — never a
    /// hardcoded `matches!(c, '—' | ':')`, but a query over whichever loaded
    /// marks this writing system's own punctuation table classifies under
    /// either function (the Unicode Pd dash-punctuation set — 25 members,
    /// [`super::symbols::dash_punctuation`] — UNIONED with the single
    /// hand-authored [`super::symbols::punctuation::colon`] entry, both
    /// already composed into [`english_writing_system`]'s table).
    pub fn is_enumeration_introducer(&self, c: char) -> bool {
        self.punctuation.iter().any(|p| {
            p.character == c
                && matches!(
                    p.function,
                    PunctuationFunction::Dash | PunctuationFunction::Introducer
                )
        })
    }
}

/// English writing system: Latin script, Arabic numerals, LTR.
///
/// Punctuation is the COMPOSITION of the hand-authored 12-mark
/// `standard_punctuation()` table (each mark's OTHER functional role —
/// StatementTerminator, Separator, Quotation, … — is not derivable from
/// Unicode category alone, so those stay authored) with the loaded Unicode
/// General_Category=Pd (Dash_Punctuation) glyph set (25 members) — ADDING
/// Dash-category coverage, never replacing the table. Deduped by character:
/// `standard_punctuation()`'s own ASCII-hyphen `hyphen()` entry (also Pd)
/// wins over the loaded set's duplicate, so composing is idempotent and the
/// authored entry's identity is preserved.
pub fn english_writing_system() -> WritingSystem {
    use super::symbols::{character, dash_punctuation, numeral, punctuation};

    #[cfg(feature = "std")]
    let dashes = dash_punctuation::vocabulary();
    #[cfg(not(feature = "std"))]
    let owned_dashes = dash_punctuation::load();
    #[cfg(not(feature = "std"))]
    let dashes = &owned_dashes;

    let mut marks = punctuation::standard_punctuation();
    for mark in dashes.punctuation_marks() {
        if !marks.iter().any(|m| m.character == mark.character) {
            marks.push(mark);
        }
    }

    WritingSystem::new("English", character::latin(), numeral::arabic()).with_punctuation(marks)
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn english_recognizes_latin() {
        let ws = english_writing_system();
        assert!(ws.recognizes('a'));
        assert!(ws.recognizes('Z'));
        assert!(ws.recognizes('5'));
        assert!(ws.recognizes('.'));
        assert!(!ws.recognizes('\u{05D0}')); // aleph
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn english_recognizes_the_loaded_unicode_dash_set() {
        // The composition this module's own doc comment claims: the loaded
        // Pd (Dash_Punctuation) set is ADDED to the hand-authored table, not
        // just the single ASCII hyphen `standard_punctuation()` already
        // carries. U+2014 EM DASH is the confirmed-bug glyph
        // (`lambek::tokenize::flush_word`'s "means—" regression).
        let ws = english_writing_system();
        assert!(ws.recognizes('-'), "ASCII hyphen still recognized");
        assert!(ws.recognizes('\u{2014}'), "em dash now recognized");
        assert!(ws.recognizes('\u{2013}'), "en dash now recognized");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn english_punctuation_has_no_duplicate_ascii_hyphen() {
        // The dedup this module's doc comment claims: composing the loaded
        // Pd set with `standard_punctuation()` must not double-insert the
        // ASCII hyphen both tables carry.
        let ws = english_writing_system();
        let hyphen_count = ws.punctuation.iter().filter(|m| m.character == '-').count();
        assert_eq!(hyphen_count, 1, "ASCII hyphen appears exactly once");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_enumeration_introducer_recognizes_dash_and_colon_not_ordinary_punctuation() {
        // The two loaded USLM "sandwich"-chapeau signals: the em dash (the
        // confirmed-bug glyph, Dash function) and the colon (Introducer
        // function, "introduces ... a list"). Real USC drafting uses both
        // ("the term X means—" / "means the following:").
        let ws = english_writing_system();
        assert!(
            ws.is_enumeration_introducer('\u{2014}'),
            "em dash signals a continued enumeration"
        );
        assert!(
            ws.is_enumeration_introducer(':'),
            "colon signals a continued enumeration"
        );
        // A period ends a COMPLETE sentence (StatementTerminator, not
        // Dash/Introducer) — never a false positive.
        assert!(!ws.is_enumeration_introducer('.'));
        // A comma (Separator) is not an enumeration introducer either.
        assert!(!ws.is_enumeration_introducer(','));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn english_is_ltr() {
        let ws = english_writing_system();
        assert_eq!(ws.direction, Direction::LeftToRight);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hebrew_recognizes_hebrew() {
        let ws = hebrew_writing_system();
        assert!(ws.recognizes('\u{05D0}')); // aleph
        assert!(ws.recognizes('5')); // Arabic numerals shared
        assert!(!ws.recognizes('a')); // Latin not in Hebrew
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hebrew_is_rtl() {
        let ws = hebrew_writing_system();
        assert_eq!(ws.direction, Direction::RightToLeft);
    }
}
