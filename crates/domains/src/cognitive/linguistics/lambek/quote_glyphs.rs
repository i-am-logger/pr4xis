//! The quotation-mark glyph vocabulary — LOADED data, not a hardcoded char
//! set — the closed inventory of Unicode characters that open or close a
//! quoted span (Slice B's quoted-mention NP recognizer,
//! `.notes/chat-fix-c-build-state.md`: "what does {adverb} mean" needs
//! `deadly` in `what does "deadly" mean` to type as `NP` — quoting a bare
//! content word licenses that reading, Quine 1940:26 via SEP "Quotation"
//! §3.1).
//!
//! Mirrors [`operators`](super::operators): the committed content-addressed
//! `.prx` is `include_bytes!`-embedded, decoded through the generalized
//! raw-source gate, parsed into a typed vocabulary, and cached in a process
//! `OnceLock` (`std`) or rebuilt by value (`no_std`).
//!
//! Source: Unicode Standard Annex #44 (Unicode Character Database),
//! General_Category property — Pi (Initial_Punctuation, 12 members) and Pf
//! (Final_Punctuation, 10 members), verified against Unicode 18.0.0 by
//! direct inspection of `DerivedGeneralCategory.txt`; plus the two
//! direction-ambiguous ASCII quote marks (U+0022, U+0027 — General_Category
//! Po), carried separately per the UCD's own `Quotation_Mark` property
//! listing. Full citation + row format in the loaded TSV
//! (`data/grammar/quote-glyphs.tsv`).

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// Whether a glyph opens, closes, or ambiguously either opens or closes a
/// quoted span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteRole {
    /// Unicode General_Category Pi — opens a quoted span.
    Initial,
    /// Unicode General_Category Pf — closes a quoted span.
    Final,
    /// Unicode General_Category Po, the two ASCII quote marks (`"`, `'`):
    /// ASCII does not encode quote directionality, so either glyph can open
    /// OR close a span depending on position.
    Ambiguous,
}

/// One loaded quote glyph: its role, its paired counterpart (if the UCD
/// pairs one), and the UCD character name for citation traceability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteGlyph {
    /// The glyph itself.
    pub glyph: char,
    /// Initial / Final / Ambiguous.
    pub role: QuoteRole,
    /// For an `Initial` glyph, its paired `Final` counterpart, when the UCD
    /// pairs one (10 of the 12 Pi members do; U+201B and U+201F do not —
    /// real, unpaired code points, not a gap in the loaded table). `None`
    /// for every `Final` or `Ambiguous` glyph — the pairing is recorded once,
    /// on the `Initial` side; [`QuoteGlyphVocabulary::closes`] answers the
    /// reverse question.
    pub pairs_with: Option<char>,
    /// The UCD character name (e.g. "LEFT SINGLE QUOTATION MARK").
    pub name: String,
}

/// The loaded quote-glyph vocabulary — glyph → its role/pairing/name. Built
/// by [`load`].
#[derive(Debug, Clone, Default)]
pub struct QuoteGlyphVocabulary(BTreeMap<char, QuoteGlyph>);

impl QuoteGlyphVocabulary {
    /// Is `c` a loaded quote glyph (opener, closer, or ambiguous)?
    pub fn is_quote_glyph(&self, c: char) -> bool {
        self.0.contains_key(&c)
    }

    /// Does `c` open a quoted span — an `Initial` glyph, or an `Ambiguous`
    /// one (which can serve as an opener)?
    pub fn opens(&self, c: char) -> bool {
        matches!(
            self.0.get(&c).map(|g| g.role),
            Some(QuoteRole::Initial | QuoteRole::Ambiguous)
        )
    }

    /// Does `close` legitimately close a span opened by `open`? `Ambiguous`
    /// glyphs close any `Ambiguous`-opened span (ASCII does not encode
    /// directionality); an `Initial` glyph's span closes only with its own
    /// loaded [`QuoteGlyph::pairs_with`] counterpart, or with NO counterpart
    /// at all for the two unpaired Pi glyphs (U+201B, U+201F) — which
    /// therefore never legitimately close, honestly reflecting the loaded
    /// data rather than guessing a pairing the UCD does not assert.
    pub fn closes(&self, open: char, close: char) -> bool {
        let Some(opener) = self.0.get(&open) else {
            return false;
        };
        match opener.role {
            QuoteRole::Ambiguous => self
                .0
                .get(&close)
                .is_some_and(|g| g.role == QuoteRole::Ambiguous),
            QuoteRole::Initial => opener.pairs_with == Some(close),
            QuoteRole::Final => false,
        }
    }

    /// The loaded record for a glyph, if it is a loaded quote glyph.
    pub fn get(&self, c: char) -> Option<&QuoteGlyph> {
        self.0.get(&c)
    }

    /// Every loaded quote glyph.
    pub fn iter(&self) -> impl Iterator<Item = &QuoteGlyph> {
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
}

/// Load the bundled quote-glyph vocabulary from
/// `data/grammar/quote-glyphs.tsv` through the generalized raw-source gate.
///
/// Returns the parsed vocabulary BY VALUE (the function-words/operators
/// idiom): no global `OnceLock` here, so this stays `no_std`-clean. The
/// bundle ships with praxis, so a parse failure is a build-time invariant
/// and panics rather than silently degrading.
pub fn load() -> QuoteGlyphVocabulary {
    // The committed quote-glyph `.prx` — the content-addressed envelope
    // carrying the authored TSV bytes. The raw `.tsv` is the git-tracked
    // source-of-truth but is EXCLUDED from the published crate; only this
    // `.prx` ships, loaded through the generalized fail-closed
    // `[compact_archive_signatures]` gate.
    const QUOTE_GLYPHS_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/grammar/quote-glyphs.prx"
    ));
    const NAME: &str = "quote_glyphs";
    const VERSION: &str = "2026";

    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, QUOTE_GLYPHS_PRX);

    let mut map: BTreeMap<char, QuoteGlyph> = BTreeMap::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || line.starts_with("codepoint\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let [codepoint, glyph_col, role_col, pairs_with_col, name] = cols[..] else {
            panic!("quote-glyphs.tsv row does not carry 5 columns: {line:?}");
        };
        let cp = u32::from_str_radix(codepoint, 16)
            .unwrap_or_else(|_| panic!("quote-glyphs.tsv: {codepoint:?} is not hex"));
        let glyph = char::from_u32(cp)
            .unwrap_or_else(|| panic!("quote-glyphs.tsv: {codepoint:?} is not a valid codepoint"));
        // Build-time invariant: the glyph column and the codepoint column
        // must name the SAME character — catches a hand-transcription slip
        // (the two easily-confused curly-quote glyphs, for instance).
        let glyph_col_char = glyph_col.chars().next().unwrap_or_else(|| {
            panic!("quote-glyphs.tsv: row for {codepoint:?} has an empty glyph column")
        });
        assert_eq!(
            glyph, glyph_col_char,
            "quote-glyphs.tsv: codepoint {codepoint:?} does not match its glyph column {glyph_col:?}"
        );
        let role = match role_col {
            "initial" => QuoteRole::Initial,
            "final" => QuoteRole::Final,
            "ambiguous" => QuoteRole::Ambiguous,
            other => panic!("quote-glyphs.tsv: unknown role {other:?} for {codepoint:?}"),
        };
        let pairs_with = match pairs_with_col {
            "-" => None,
            hex => Some(
                u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .unwrap_or_else(|| {
                        panic!("quote-glyphs.tsv: {hex:?} is not a valid pairs_with codepoint")
                    }),
            ),
        };
        map.insert(
            glyph,
            QuoteGlyph {
                glyph,
                role,
                pairs_with,
                name: name.to_string(),
            },
        );
    }
    QuoteGlyphVocabulary(map)
}

/// The process-cached quote-glyph vocabulary (`std`): [`load`] parses
/// `quote-glyphs.tsv` once behind a `OnceLock`, so the tokenizer reuses it
/// instead of re-parsing on every call. On `no_std` there is no global
/// `OnceLock`, so callers use [`load`] by value.
#[cfg(feature = "std")]
pub fn vocabulary() -> &'static QuoteGlyphVocabulary {
    use std::sync::OnceLock;
    static VOCAB: OnceLock<QuoteGlyphVocabulary> = OnceLock::new();
    VOCAB.get_or_init(load)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn vocabulary_loads_the_full_glyph_set() {
        let v = load();
        // 12 Pi + 10 Pf + 2 ASCII Po = 24, per the UCD counts cited in the
        // loaded table's own header.
        assert_eq!(v.len().value, 24.0, "the full quote-glyph set loads");
        for g in [
            '"', '\'', '«', '»', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}',
        ] {
            assert!(v.is_quote_glyph(g), "missing quote glyph {g:?}");
        }
        // Non-quote punctuation is NOT a loaded quote glyph.
        for g in ['?', '.', ',', '-', 'a', '1'] {
            assert!(!v.is_quote_glyph(g), "{g:?} is not a quote glyph");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pi_count_is_twelve_and_pf_count_is_ten() {
        // The exact UCD counts (General_Category Pi / Pf), verified against
        // Unicode 18.0.0 -- the loaded table's own citation claim, checked
        // against the parsed data rather than only stated in a comment.
        let v = load();
        let initial = v.iter().filter(|g| g.role == QuoteRole::Initial).count();
        let final_ = v.iter().filter(|g| g.role == QuoteRole::Final).count();
        let ambiguous = v.iter().filter(|g| g.role == QuoteRole::Ambiguous).count();
        assert_eq!(initial, 12, "Pi (Initial_Punctuation) has 12 members");
        assert_eq!(final_, 10, "Pf (Final_Punctuation) has 10 members");
        assert_eq!(ambiguous, 2, "the two ASCII quote marks");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn curly_double_quotes_pair_and_close() {
        let v = load();
        assert!(v.opens('\u{201C}'), "left double quote opens");
        assert!(!v.opens('\u{201D}'), "right double quote does not open");
        assert!(
            v.closes('\u{201C}', '\u{201D}'),
            "left double quote closes with right double quote"
        );
        assert!(
            !v.closes('\u{201C}', '\u{2019}'),
            "left double quote does NOT close with right SINGLE quote"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn the_two_unpaired_pi_glyphs_never_close() {
        // U+201B and U+201F are genuinely unpaired in the UCD -- the
        // vocabulary must not invent a pairing for them.
        let v = load();
        for unpaired in ['\u{201B}', '\u{201F}'] {
            let g = v.get(unpaired).expect("loaded");
            assert_eq!(g.role, QuoteRole::Initial);
            assert_eq!(
                g.pairs_with, None,
                "{unpaired:?} has no Pf counterpart in the UCD"
            );
            for c in v.iter().map(|g| g.glyph) {
                assert!(
                    !v.closes(unpaired, c),
                    "{unpaired:?} must not close with anything (honest, not guessed)"
                );
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn ascii_quotes_are_ambiguous_and_only_close_each_other() {
        let v = load();
        assert!(v.opens('"'), "ASCII double quote can open");
        assert!(v.opens('\''), "ASCII single quote can open");
        assert!(
            v.closes('"', '"'),
            "an ambiguous glyph closes a span another ambiguous glyph opened"
        );
        assert!(
            v.closes('\'', '"'),
            "any ambiguous glyph closes any ambiguous-opened span (ASCII has no directionality)"
        );
        assert!(
            !v.closes('"', '\u{201D}'),
            "an ASCII opener does not close with a directional Pf glyph"
        );
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
