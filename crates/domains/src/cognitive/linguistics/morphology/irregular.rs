//! Irregular-form types — Pinker (1991) dual-route model. Surface
//! inflections that are *not* predictable from
//! [`super::MorphologicalRule`]s live in a lookup table per
//! language, registered in `super::<lang>::irregular`.
//!
//! This module hosts only the **universal** types — the
//! [`IrregularForm`] record and the [`IrregularKind`] taxonomy —
//! plus the generic [`lookup_in`] helper. The language-specific
//! data tables live in `super::english::irregular`, sibling
//! `super::latin::irregular`, etc.
//!
//! # Why a table is the right shape
//!
//! Regular forms compose mechanically: `walk` + `-ed` → `walked`.
//! Irregular forms do not: `go` + `<past>` → `went` (suppletion);
//! `child` + `<plural>` → `children` (umlaut + suffix); `mouse` +
//! `<plural>` → `mice` (umlaut). The forms are listed in the
//! mental lexicon, not generated. Praxis encodes this with an
//! explicit registry the lemmatizer consults *before* attempting
//! rule-based inversion.
//!
//! # Praxis "bottom-up loaded" rule
//!
//! Lexical data should be loaded from a registered authoritative
//! source. The per-language tables currently hold the role only
//! until AGID (Atkinson 2003) or WordNet's morph-exception files
//! are wired up via `praxis.toml`. The types in this file stay;
//! the language-specific data moves.
//!
//! # Literature
//!
//! - **Pinker, Steven (1991)** "Rules of Language", *Science* 253,
//!   530–535 — dual-route model.
//! - **Pinker, Steven (1999)** *Words and Rules: The Ingredients
//!   of Language*, Basic Books — full book-length treatment.
//! - **Quirk, Greenbaum, Leech & Svartvik (1985)** *A
//!   Comprehensive Grammar of the English Language*, Longman,
//!   §3.21–3.26 (irregular nouns) and §3.40–3.59 (irregular
//!   verbs) — English language reference.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// What kind of inflectional change the surface form represents.
///
/// Mirrors [`super::SemanticEffect`] but specialised to the
/// inflectional sub-set — irregular forms only appear for
/// inflectional, not derivational, morphology
/// (Bauer 1983 *English Word-formation* §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrregularKind {
    /// Plural noun: child → children, mouse → mice.
    PluralNoun,
    /// Past-tense verb: go → went, run → ran.
    PastTense,
    /// Past participle: go → gone, run → run, write → written.
    PastParticiple,
    /// Comparative adjective: good → better.
    Comparative,
    /// Superlative adjective: good → best.
    Superlative,
}

impl Concept for IrregularKind {
    fn variants() -> Vec<Self> {
        vec![
            Self::PluralNoun,
            Self::PastTense,
            Self::PastParticiple,
            Self::Comparative,
            Self::Superlative,
        ]
    }
}

/// One irregular-form entry: surface form, its lemma, and what
/// inflectional change the surface represents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IrregularForm {
    pub surface: String,
    pub lemma: String,
    pub kind: IrregularKind,
}

impl IrregularForm {
    pub fn new(surface: &str, lemma: &str, kind: IrregularKind) -> Self {
        Self {
            surface: surface.to_string(),
            lemma: lemma.to_string(),
            kind,
        }
    }
}

/// Generic case-insensitive lookup over an irregular-forms table.
/// Returns every entry whose surface matches `needle` (ASCII
/// case-folded).
pub fn lookup_in(needle: &str, table: &[IrregularForm]) -> Vec<IrregularForm> {
    let lower = needle.to_ascii_lowercase();
    table
        .iter()
        .filter(|f| f.surface == lower)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_variants_complete() {
        let kinds = IrregularKind::variants();
        assert_eq!(kinds.len(), 5);
    }

    #[test]
    fn lookup_in_is_case_insensitive() {
        let table = vec![IrregularForm::new(
            "children",
            "child",
            IrregularKind::PluralNoun,
        )];
        for needle in &["children", "Children", "CHILDREN"] {
            let hits = lookup_in(needle, &table);
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].lemma, "child");
        }
    }

    #[test]
    fn lookup_in_empty_table_returns_empty() {
        let hits = lookup_in("children", &[]);
        assert!(hits.is_empty());
    }

    #[test]
    fn lookup_in_unknown_word_returns_empty() {
        let table = vec![IrregularForm::new(
            "children",
            "child",
            IrregularKind::PluralNoun,
        )];
        let hits = lookup_in("xyzzy", &table);
        assert!(hits.is_empty());
    }
}
