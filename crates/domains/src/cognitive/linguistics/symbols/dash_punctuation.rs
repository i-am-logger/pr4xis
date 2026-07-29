//! The dash-punctuation glyph vocabulary — LOADED data, not a hardcoded char
//! set — the closed inventory of Unicode General_Category=Pd (Dash_
//! Punctuation) code points, fixing a confirmed tokenization bug: the
//! tokenizer's `flush_word` (`cognitive::linguistics::lambek::tokenize`)
//! trimmed leading/trailing punctuation off an accumulated word buffer with
//! a raw `char::is_ascii_punctuation()` check, which is FALSE for every
//! non-ASCII Unicode dash — so real U.S. Code legal text carrying an EM DASH
//! after a defining verb (`the term "developmental disability" means—`,
//! followed by an enumerated list — a common USC drafting convention) left
//! the malformed token `means—` in the stream, which then missed lexicon
//! lookup for the bare word `means`, breaking downstream grammar parsing.
//!
//! Mirrors [`quote_glyphs`](super::super::lambek::quote_glyphs) /
//! [`operators`](super::super::lambek::operators): the committed content-
//! addressed `.prx` is `include_bytes!`-embedded, decoded through the
//! generalized raw-source gate, parsed into a typed vocabulary, and cached
//! in a process `OnceLock` (`std`) or rebuilt by value (`no_std`). Simpler
//! than `QuoteGlyphVocabulary`: Pd carries no pairing/nesting structure (no
//! `role`/`pairs_with` columns), so this is a flat `codepoint → name` map.
//!
//! Lives under `symbols/` (not `lambek/`, unlike its sibling
//! `quote_glyphs`) because it has TWO consumers on opposite sides of the
//! `orthography` ⇄ `lambek` boundary — `orthography::english_writing_system`
//! composes it into the loaded
//! [`WritingSystem`](super::super::orthography::WritingSystem) punctuation
//! table, and `lambek::tokenize::flush_word`
//! queries it directly on the tokenizer's hot path (a per-word call, so it
//! is threaded through as a loaded vocabulary parameter — the same pattern
//! `tokenize` already uses for `OperatorVocabulary`/`QuoteGlyphVocabulary` —
//! rather than through `Language::writing_system()`, which is documented as
//! a COLD, test-only reader that deserializes a whole owned tree on every
//! call). `symbols` is the common ancestor both already depend on, so
//! placing the vocabulary here needs no new dependency edge either way.
//!
//! Source: Unicode Standard Annex #44 (Unicode Character Database),
//! General_Category property — Pd (Dash_Punctuation, 25 members), verified
//! against Unicode 18.0.0 (2026) via compart.com/en/unicode/category/Pd,
//! cross-referenced against the UCD's own DerivedGeneralCategory.txt
//! tabulation. Full citation + row format in the loaded TSV
//! (`data/grammar/dash-punctuation.tsv`).

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use super::punctuation::{Position, PunctuationFunction, PunctuationMark};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// One loaded dash-punctuation glyph: itself plus its UCD character name
/// (citation traceability).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashGlyph {
    /// The glyph itself.
    pub glyph: char,
    /// The UCD character name (e.g. "EM DASH").
    pub name: String,
}

/// The loaded dash-punctuation vocabulary — every Unicode General_Category=Pd
/// code point. Built by [`load`].
#[derive(Debug, Clone, Default)]
pub struct DashPunctuationVocabulary(BTreeMap<char, DashGlyph>);

impl DashPunctuationVocabulary {
    /// Is `c` a loaded dash-punctuation glyph (Unicode General_Category=Pd)?
    pub fn is_dash_glyph(&self, c: char) -> bool {
        self.0.contains_key(&c)
    }

    /// The loaded record for a glyph, if it is a loaded dash glyph.
    pub fn get(&self, c: char) -> Option<&DashGlyph> {
        self.0.get(&c)
    }

    /// Every loaded dash glyph.
    pub fn iter(&self) -> impl Iterator<Item = &DashGlyph> {
        self.0.values()
    }

    /// Total number of loaded glyphs.
    pub fn len(&self) -> Quantity {
        Quantity::from_unit(self.0.len() as f64, &unit::UNITLESS)
    }

    /// True iff no glyph loaded.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every loaded glyph as a [`PunctuationMark`] — `PunctuationFunction::
    /// Dash`, `Position::Between`, matching the shape of the existing
    /// hand-authored [`punctuation::hyphen`](super::punctuation::hyphen)
    /// entry (Dash "indicates a range, pause, or interruption"; Between
    /// "comma, semicolon, colon"-style medial marks). The composition site
    /// (`orthography::english_writing_system`) merges this into the SAME
    /// `Vec<PunctuationMark>` `standard_punctuation()` already builds,
    /// deduping by character so the pre-existing ASCII-hyphen entry is not
    /// duplicated — this method itself stays a pure, total conversion of
    /// the whole loaded set, with no knowledge of that composition policy.
    pub fn punctuation_marks(&self) -> Vec<PunctuationMark> {
        self.0
            .values()
            .map(|g| {
                PunctuationMark::new(
                    g.glyph,
                    &g.name,
                    PunctuationFunction::Dash,
                    Position::Between,
                )
            })
            .collect()
    }
}

/// Load the bundled dash-punctuation vocabulary from
/// `data/grammar/dash-punctuation.tsv` through the generalized raw-source
/// gate.
///
/// Returns the parsed vocabulary BY VALUE (the function-words/operators/
/// quote-glyphs idiom): no global `OnceLock` here, so this stays `no_std`-
/// clean. The bundle ships with praxis, so a parse failure is a build-time
/// invariant and panics rather than silently degrading.
pub fn load() -> DashPunctuationVocabulary {
    // The committed dash-punctuation `.prx` — the content-addressed envelope
    // carrying the authored TSV bytes. The raw `.tsv` is the git-tracked
    // source-of-truth but is EXCLUDED from the published crate; only this
    // `.prx` ships, loaded through the generalized fail-closed
    // `[compact_archive_signatures]` gate.
    const DASH_PUNCTUATION_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/grammar/dash-punctuation.prx"
    ));
    const NAME: &str = "dash_punctuation";
    const VERSION: &str = "2026";

    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, DASH_PUNCTUATION_PRX);

    let mut map: BTreeMap<char, DashGlyph> = BTreeMap::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || line.starts_with("codepoint\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let [codepoint, glyph_col, name] = cols[..] else {
            panic!("dash-punctuation.tsv row does not carry 3 columns: {line:?}");
        };
        let cp = u32::from_str_radix(codepoint, 16)
            .unwrap_or_else(|_| panic!("dash-punctuation.tsv: {codepoint:?} is not hex"));
        let glyph = char::from_u32(cp).unwrap_or_else(|| {
            panic!("dash-punctuation.tsv: {codepoint:?} is not a valid codepoint")
        });
        // Build-time invariant: the glyph column and the codepoint column
        // must name the SAME character — catches a hand-transcription slip.
        let glyph_col_char = glyph_col.chars().next().unwrap_or_else(|| {
            panic!("dash-punctuation.tsv: row for {codepoint:?} has an empty glyph column")
        });
        assert_eq!(
            glyph, glyph_col_char,
            "dash-punctuation.tsv: codepoint {codepoint:?} does not match its glyph column {glyph_col:?}"
        );
        map.insert(
            glyph,
            DashGlyph {
                glyph,
                name: name.to_string(),
            },
        );
    }
    DashPunctuationVocabulary(map)
}

/// The process-cached dash-punctuation vocabulary (`std`): [`load`] parses
/// `dash-punctuation.tsv` once behind a `OnceLock`, so the tokenizer reuses
/// it instead of re-parsing on every call. On `no_std` there is no global
/// `OnceLock`, so callers use [`load`] by value.
#[cfg(feature = "std")]
pub fn vocabulary() -> &'static DashPunctuationVocabulary {
    use std::sync::OnceLock;
    static VOCAB: OnceLock<DashPunctuationVocabulary> = OnceLock::new();
    VOCAB.get_or_init(load)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn vocabulary_loads_the_full_glyph_set() {
        // 25 Pd (Dash_Punctuation) members, per the UCD count cited in the
        // loaded table's own header.
        let v = load();
        assert_eq!(v.len().value, 25.0, "the full dash-punctuation set loads");
        for g in [
            '-', '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}', '\u{2015}',
        ] {
            assert!(v.is_dash_glyph(g), "missing dash glyph {g:?}");
        }
        // Non-dash punctuation is NOT a loaded dash glyph.
        for g in ['"', '\'', '?', '.', ',', 'a', '1'] {
            assert!(!v.is_dash_glyph(g), "{g:?} is not a dash glyph");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn em_dash_is_loaded_with_its_ucd_name() {
        // The specific glyph the confirmed bug report names: U+2014 EM DASH,
        // the mark ending "the term X means—" in real USC drafting.
        let v = load();
        let g = v.get('\u{2014}').expect("em dash is loaded");
        assert_eq!(g.name, "EM DASH");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn punctuation_marks_are_all_dash_between() {
        let v = load();
        let marks = v.punctuation_marks();
        assert_eq!(marks.len(), 25, "one PunctuationMark per loaded glyph");
        for m in &marks {
            assert_eq!(m.function, PunctuationFunction::Dash);
            assert_eq!(m.position, Position::Between);
            assert!(v.is_dash_glyph(m.character));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_loaded_glyph_carries_a_nonempty_ucd_name() {
        // Coverage over the WHOLE bundle, not a hand-picked sample -- every
        // row's citation-traceability name is present.
        let v = load();
        for g in v.iter() {
            assert!(
                !g.name.is_empty(),
                "{:?} ({:#06x}) carries no UCD name",
                g.glyph,
                g.glyph as u32
            );
        }
    }
}
