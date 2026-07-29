//! Simple Unicode case folding — the codepoint-for-codepoint mapping the
//! fold-on-miss lookup uses to recognize that two differently-cased
//! surfaces name the SAME lexicon entry (Slice D,
//! `.notes/chat-fix-c-build-state.md`: 17,272 of 131,798 loaded WordNet
//! surfaces carry an uppercase letter — "Section Eight", "Turkish bath",
//! "O.K." — and the tokenizer's lowercasing discards that case before the
//! exact-case WordIndex lookup ever runs, a MISS this table's fold-index
//! recovers).
//!
//! Deliberately distinct from Rust's `str::to_lowercase` (a locale-generic
//! DISPLAY lowercasing) — the UCD defines case folding as its own property,
//! specifically for CASELESS MATCHING (Unicode Standard Annex #44 §5.6;
//! Unicode Technical Standard #10 §3.13 "Default Caseless Matching"), and
//! this crate sources it from the loaded table rather than the standard
//! library's algorithm, so the fold is auditable against the same primary
//! citation every other loaded vocabulary in this crate is.
//!
//! Mirrors [`operators`](crate::cognitive::linguistics::lambek::operators) /
//! [`quote_glyphs`](crate::cognitive::linguistics::lambek::quote_glyphs):
//! the committed content-addressed `.prx` is `include_bytes!`-embedded,
//! decoded through the generalized raw-source gate, parsed into a typed
//! table, and cached in a process `OnceLock` (`std`) or rebuilt by value
//! (`no_std`).
//!
//! Source: Unicode Standard Annex #44 (Unicode Character Database),
//! `CaseFolding.txt`, status C (common) + S (simple) — the single-codepoint
//! fold (status F, full folding, can grow the string — e.g. LATIN SMALL
//! LETTER SHARP S → "ss" — and is excluded, since a WordIndex key needs a
//! fixed-width fold; status T, Turkic-locale dotted/dotless I, is excluded
//! per the source file's own default-usage note). Verified against Unicode
//! 18.0.0 (2026-02-03) by direct inspection of the primary data file
//! (`unicode-org/unicodetools:unicodetools/data/ucd/dev/CaseFolding.txt`).
//! Full citation + row format in the loaded TSV
//! (`data/orthography/case-folding.tsv`).

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// One loaded case-folding entry: a codepoint's simple fold target and its
/// UCD name (citation traceability).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseFold {
    /// The source codepoint.
    pub from: char,
    /// Its simple case-folded target.
    pub to: char,
    /// The UCD character name of `from` (e.g. "LATIN CAPITAL LETTER A").
    pub name: String,
}

/// The loaded simple case-folding table — codepoint → its fold entry. A
/// codepoint absent from the table folds to itself (the UCD's own
/// convention: "All code points not listed in this file map to
/// themselves").
#[derive(Debug, Clone, Default)]
pub struct CaseFoldingTable(BTreeMap<char, CaseFold>);

impl CaseFoldingTable {
    /// The simple case fold of one character — itself, unless the table
    /// carries an explicit mapping.
    pub fn fold_char(&self, c: char) -> char {
        self.0.get(&c).map_or(c, |f| f.to)
    }

    /// The simple case fold of a whole string — the per-character fold,
    /// concatenated. Length-preserving in codepoint count (the property that
    /// distinguishes SIMPLE from FULL case folding — see this module's own
    /// citation), so a folded WordIndex key is cheap to derive and compare.
    pub fn fold(&self, s: &str) -> String {
        s.chars().map(|c| self.fold_char(c)).collect()
    }

    /// The loaded record for a codepoint, if it carries a non-identity fold.
    pub fn get(&self, c: char) -> Option<&CaseFold> {
        self.0.get(&c)
    }

    /// Every loaded entry.
    pub fn iter(&self) -> impl Iterator<Item = &CaseFold> {
        self.0.values()
    }

    /// Total number of loaded (non-identity) entries.
    pub fn len(&self) -> Quantity {
        Quantity::from_unit(self.0.len() as f64, &unit::UNITLESS)
    }

    /// True iff no entry loaded.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load the bundled case-folding table from `data/orthography/case-folding.tsv`
/// through the generalized raw-source gate.
///
/// Returns the parsed table BY VALUE (the function-words/operators idiom):
/// no global `OnceLock` here, so this stays `no_std`-clean. The bundle ships
/// with praxis, so a parse failure is a build-time invariant and panics
/// rather than silently degrading.
pub fn load() -> CaseFoldingTable {
    const CASE_FOLDING_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/orthography/case-folding.prx"
    ));
    const NAME: &str = "case_folding";
    const VERSION: &str = "2026";

    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, CASE_FOLDING_PRX);

    let mut map: BTreeMap<char, CaseFold> = BTreeMap::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || line.starts_with("codepoint\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let [from_hex, to_hex, name] = cols[..] else {
            panic!("case-folding.tsv row does not carry 3 columns: {line:?}");
        };
        let from_cp = u32::from_str_radix(from_hex, 16)
            .unwrap_or_else(|_| panic!("case-folding.tsv: {from_hex:?} is not hex"));
        let from = char::from_u32(from_cp)
            .unwrap_or_else(|| panic!("case-folding.tsv: {from_hex:?} is not a valid codepoint"));
        let to_cp = u32::from_str_radix(to_hex, 16)
            .unwrap_or_else(|_| panic!("case-folding.tsv: {to_hex:?} is not hex"));
        let to = char::from_u32(to_cp)
            .unwrap_or_else(|| panic!("case-folding.tsv: {to_hex:?} is not a valid codepoint"));
        map.insert(
            from,
            CaseFold {
                from,
                to,
                name: name.to_string(),
            },
        );
    }
    CaseFoldingTable(map)
}

/// The process-cached case-folding table (`std`): [`load`] parses
/// `case-folding.tsv` once behind a `OnceLock`, so callers reuse it instead
/// of re-parsing on every call. On `no_std` there is no global `OnceLock`,
/// so callers use [`load`] by value.
#[cfg(feature = "std")]
pub fn table() -> &'static CaseFoldingTable {
    use std::sync::OnceLock;
    static TABLE: OnceLock<CaseFoldingTable> = OnceLock::new();
    TABLE.get_or_init(load)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ascii_uppercase_folds_to_lowercase() {
        let t = load();
        assert_eq!(t.fold_char('A'), 'a');
        assert_eq!(t.fold_char('Z'), 'z');
        assert_eq!(t.fold_char('a'), 'a', "already-folded chars are stable");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_multiword_capitalized_surface_folds_to_its_lowercase_form() {
        let t = load();
        assert_eq!(t.fold("Section Eight"), "section eight");
        assert_eq!(t.fold("Turkish bath"), "turkish bath");
        assert_eq!(t.fold("O.K."), "o.k.");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn digits_and_punctuation_are_identity() {
        // Case folding is defined over cased letters only — a codepoint
        // with no case distinction is absent from the loaded table and
        // folds to itself.
        let t = load();
        for c in ['0', '9', '.', ' ', '-'] {
            assert_eq!(t.fold_char(c), c);
            assert!(t.get(c).is_none());
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loads_the_full_simple_fold_population() {
        // Status C (1501) + S (32) from the loaded citation's own header —
        // checked against the parsed data, not only stated in a comment.
        let t = load();
        assert_eq!(t.len().value, 1533.0);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_loaded_entry_carries_a_nonempty_ucd_name() {
        let t = load();
        for f in t.iter() {
            assert!(
                !f.name.is_empty(),
                "{:?} ({:#06x}) carries no UCD name",
                f.from,
                f.from as u32
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn fold_is_length_preserving_in_codepoint_count() {
        let t = load();
        for s in ["Section Eight", "O.K.", "STRASSE", "İstanbul"] {
            assert_eq!(
                t.fold(s).chars().count(),
                s.chars().count(),
                "simple folding never changes codepoint count: {s:?}"
            );
        }
    }
}
