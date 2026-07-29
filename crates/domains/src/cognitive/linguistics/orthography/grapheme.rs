//! The English orthographic grapheme classes — LOADED data, not a hardcoded
//! char set.
//!
//! Fixes a real praxis-law violation: `morphology::english::generation` and
//! `morphology::english::irregular` each carried their own private
//!
//! ```text
//! fn is_vowel(c: char) -> bool { matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') }
//! ```
//!
//! — the English vowel inventory written as a Rust match arm, duplicated
//! across two files, with `y`'s dual status scattered next to it as ad-hoc
//! `|| c == 'y'` tests. The orthographic RULES built on top of it are
//! properly cited (Quirk et al. 1985 §3, §3.21; Spencer 1991 §5.2) but the
//! alphabet they range over was encoded rather than loaded, so nothing in the
//! system could inspect, cite, or reason about it.
//!
//! The vowel/consonant split is NOT a Unicode property — the UCD classifies
//! these code points only as `General_Category=Ll/Lu`; vowel-hood is a
//! language-specific orthographic fact. So this is an AUTHORED table from the
//! descriptive literature (Carney 1994; Quirk et al. 1985), following the
//! same committed-`.prx` idiom as
//! [`dash_punctuation`](super::super::symbols::dash_punctuation): the
//! git-tracked TSV is the source-of-truth, the content-addressed `.prx` is
//! `include_bytes!`-embedded and decoded through the generalized fail-closed
//! raw-source gate, and `std` callers reuse a process `OnceLock`.
//!
//! Lives under `orthography/` because a grapheme inventory is a property of
//! the WRITING SYSTEM, not of the morphology that consumes it — the same
//! placement rule `dash_punctuation`'s own doc states (put the vocabulary at
//! the layer that owns the concept, and let consumers depend inward).
//!
//! On `y`: both cited sources treat it as DUAL rather than assigning it to
//! one class, and the two consuming rules genuinely need different readings —
//! the syllable-peak count wants `y` to count as a vowel ("rhythm" has a
//! peak), while the consonant-before-final-`y` test wants a preceding `y` NOT
//! to count as a consonant ("boy" → "boys", Quirk §3.21). Rather than expose
//! a lossy boolean and make every caller re-derive the exception, the two
//! readings are two named queries — [`EnglishGraphemeClasses::is_syllabic`]
//! and [`EnglishGraphemeClasses::is_consonantal`] — and `Dual` answers
//! `true`/`false` respectively. That is exactly what the old
//! `is_vowel(c) || c == 'y'` and `!is_vowel(c)` pair computed by hand.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

/// Which class the loaded table assigns a letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphemeClass {
    /// A basic vowel letter — `a e i o u`.
    Vowel,
    /// A consonant letter.
    Consonant,
    /// `y`: consonant word/syllable-initially ("yes"), vowel elsewhere
    /// ("baby", "boy"). Carney 1994 Ch.1.
    Dual,
}

impl GraphemeClass {
    fn parse(s: &str) -> Self {
        match s {
            "Vowel" => Self::Vowel,
            "Consonant" => Self::Consonant,
            "Dual" => Self::Dual,
            other => panic!("english-grapheme-classes.tsv: unknown class {other:?}"),
        }
    }
}

/// One loaded row: the letter, its class, and the citation note that grounds
/// the assignment (traceability — the note is carried, not dropped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphemeRow {
    pub letter: char,
    pub class: GraphemeClass,
    pub note: String,
}

/// The loaded English grapheme-class table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishGraphemeClasses(BTreeMap<char, GraphemeRow>);

impl EnglishGraphemeClasses {
    /// The row for `c`, case-insensitively (the table is authored lowercase).
    pub fn row(&self, c: char) -> Option<&GraphemeRow> {
        self.0.get(&c.to_ascii_lowercase())
    }

    /// Is `c` one of the five BASIC vowel letters `a e i o u`? `Dual` (`y`)
    /// answers NO. Distinct from [`Self::is_syllabic`], and both readings are
    /// genuinely needed: the C-V-C doubling environment (Quirk et al. 1985 §3)
    /// is stated over the basic vowels — "gym" does not double to "gymming"
    /// by rule, it is a loaded AGID exception if attested at all — while the
    /// syllable-peak count must see `y` to give "rhythm" a peak.
    pub fn is_basic_vowel(&self, c: char) -> bool {
        matches!(self.row(c).map(|r| r.class), Some(GraphemeClass::Vowel))
    }

    /// Can `c` head a syllable peak? `Vowel` yes, `Dual` (`y`) YES — "rhythm"
    /// and "gym" have peaks spelled only by `y`. This is the reading the
    /// syllable-counting rule needs (Quirk et al. 1985 §3, the monosyllabic
    /// C-V-C doubling environment).
    pub fn is_syllabic(&self, c: char) -> bool {
        matches!(
            self.row(c).map(|r| r.class),
            Some(GraphemeClass::Vowel | GraphemeClass::Dual)
        )
    }

    /// Does `c` count as a CONSONANT for the orthographic rules? `Consonant`
    /// yes, `Dual` (`y`) NO — Quirk et al. 1985 §3.21 turns on exactly this:
    /// "boy" → "boys" (not "boies") *because* the `y` follows a vowel, and
    /// the preceding letter in "baby" is a consonant while in "boy" it is
    /// not. A letter absent from the table (a digit, an accented form) is not
    /// a consonant.
    pub fn is_consonantal(&self, c: char) -> bool {
        matches!(self.row(c).map(|r| r.class), Some(GraphemeClass::Consonant))
    }

    /// Rows, for tests and introspection.
    pub fn rows(&self) -> impl Iterator<Item = &GraphemeRow> {
        self.0.values()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Parse the committed `english-grapheme-classes.tsv` through the generalized
/// raw-source gate. Returns BY VALUE so the module stays `no_std`-clean (the
/// dash-punctuation / quote-glyphs idiom); `std` callers should prefer
/// [`classes`], which caches.
///
/// The bundle ships with praxis, so a parse failure is a build-time invariant
/// and panics rather than silently degrading into a wrong alphabet.
pub fn load() -> EnglishGraphemeClasses {
    const ENGLISH_GRAPHEME_CLASSES_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/grammar/english-grapheme-classes.prx"
    ));
    const NAME: &str = "english_grapheme_classes";
    const VERSION: &str = "1994";

    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, ENGLISH_GRAPHEME_CLASSES_PRX);

    let mut map: BTreeMap<char, GraphemeRow> = BTreeMap::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || line.starts_with("letter\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let [letter_col, class_col, note] = cols[..] else {
            panic!("english-grapheme-classes.tsv row does not carry 3 columns: {line:?}");
        };
        let mut chars = letter_col.chars();
        let (Some(letter), None) = (chars.next(), chars.next()) else {
            panic!("english-grapheme-classes.tsv: {letter_col:?} is not a single letter");
        };
        map.insert(
            letter,
            GraphemeRow {
                letter,
                class: GraphemeClass::parse(class_col),
                note: note.to_string(),
            },
        );
    }
    EnglishGraphemeClasses(map)
}

/// The process-cached table (`std`): [`load`] parses once behind a
/// `OnceLock`, so the morphology hot path reuses it rather than re-parsing
/// per call. On `no_std` there is no global `OnceLock`, so callers use
/// [`load`] by value.
#[cfg(feature = "std")]
pub fn classes() -> &'static EnglishGraphemeClasses {
    use std::sync::OnceLock;
    static CLASSES: OnceLock<EnglishGraphemeClasses> = OnceLock::new();
    CLASSES.get_or_init(load)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn the_full_basic_latin_alphabet_loads_with_a_class() {
        let t = load();
        assert_eq!(t.len(), 26, "all 26 basic-Latin letters carry a class");
        for c in 'a'..='z' {
            assert!(t.row(c).is_some(), "{c} has a loaded class");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_five_vowel_letters_are_exactly_aeiou_and_y_is_dual() {
        let t = load();
        let vowels: Vec<char> = t
            .rows()
            .filter(|r| r.class == GraphemeClass::Vowel)
            .map(|r| r.letter)
            .collect();
        assert_eq!(vowels, vec!['a', 'e', 'i', 'o', 'u']);
        assert_eq!(t.row('y').map(|r| r.class), Some(GraphemeClass::Dual));
    }

    /// The two readings `y` needs, which the previous hardcoded
    /// `is_vowel(c) || c == 'y'` / `!is_vowel(c)` pair computed by hand.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn y_is_syllabic_but_not_consonantal() {
        let t = load();
        assert!(t.is_syllabic('y'), "\"rhythm\" has a peak spelled by y");
        assert!(
            !t.is_consonantal('y'),
            "Quirk 1985 §3.21: \"boy\" -> \"boys\" because y is not a consonant here"
        );
        assert!(t.is_syllabic('a') && !t.is_consonantal('a'));
        assert!(t.is_consonantal('b') && !t.is_syllabic('b'));
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn a_non_letter_is_neither_syllabic_nor_consonantal() {
        let t = load();
        for c in ['1', '-', ' ', 'é'] {
            assert!(!t.is_syllabic(c) && !t.is_consonantal(c), "{c:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_is_case_insensitive() {
        let t = load();
        assert!(t.is_syllabic('A') && t.is_consonantal('B'));
    }
}
