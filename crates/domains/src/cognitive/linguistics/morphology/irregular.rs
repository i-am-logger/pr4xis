//! Irregular forms — surface inflections not predictable from
//! [`super::MorphologicalRule`]s. Pinker (1991) "dual-route" model:
//! regular inflection goes through a rule, irregular inflection
//! goes through a lookup table.
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
//! # Coverage and the praxis "bottom-up loaded" rule
//!
//! Praxis convention is that lexical data is loaded from a
//! registered authoritative source (e.g. WordNet, AGID) rather
//! than hand-coded. The high-frequency English irregular table
//! below carries that role only until a praxis.toml-registered
//! inflection dictionary is wired up — at which point the
//! ontology stays, the data moves. Each entry cites Pinker (1991)
//! as the theoretical home and Quirk et al. (1985) §3.21–3.26 for
//! the canonical English-language list.
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
//!   verbs) — canonical English-language inflection tables.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// What kind of inflectional change the surface form represents.
///
/// Mirrors [`super::SemanticEffect`] but specialised to the
/// inflectional sub-set — irregular forms only appear for
/// inflectional, not derivational, morphology.
/// (Bauer 1983 *English Word-formation* §2.3.)
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

/// High-frequency English irregular forms.
///
/// Coverage: the closed-class irregulars (be/have/do/say/get) plus
/// the most common irregular plurals and strong verbs from Quirk et
/// al. (1985) §3.21–3.59. Not exhaustive — a praxis.toml-registered
/// inflection dictionary will replace this list (see module docs).
pub fn english_irregulars() -> Vec<IrregularForm> {
    use IrregularKind::*;
    vec![
        // ── Irregular plural nouns (Quirk et al. 1985 §3.21–3.26) ──
        IrregularForm::new("children", "child", PluralNoun),
        IrregularForm::new("men", "man", PluralNoun),
        IrregularForm::new("women", "woman", PluralNoun),
        IrregularForm::new("people", "person", PluralNoun),
        IrregularForm::new("feet", "foot", PluralNoun),
        IrregularForm::new("teeth", "tooth", PluralNoun),
        IrregularForm::new("geese", "goose", PluralNoun),
        IrregularForm::new("mice", "mouse", PluralNoun),
        IrregularForm::new("oxen", "ox", PluralNoun),
        IrregularForm::new("data", "datum", PluralNoun),
        IrregularForm::new("criteria", "criterion", PluralNoun),
        IrregularForm::new("phenomena", "phenomenon", PluralNoun),
        IrregularForm::new("analyses", "analysis", PluralNoun),
        IrregularForm::new("bases", "basis", PluralNoun),
        IrregularForm::new("crises", "crisis", PluralNoun),
        IrregularForm::new("hypotheses", "hypothesis", PluralNoun),
        IrregularForm::new("theses", "thesis", PluralNoun),
        IrregularForm::new("indices", "index", PluralNoun),
        IrregularForm::new("matrices", "matrix", PluralNoun),
        IrregularForm::new("vertices", "vertex", PluralNoun),
        IrregularForm::new("appendices", "appendix", PluralNoun),
        IrregularForm::new("formulae", "formula", PluralNoun),
        // ── Highly-frequent strong / irregular verbs ──
        // be (suppletion)
        IrregularForm::new("am", "be", PastTense),
        IrregularForm::new("is", "be", PastTense),
        IrregularForm::new("are", "be", PastTense),
        IrregularForm::new("was", "be", PastTense),
        IrregularForm::new("were", "be", PastTense),
        IrregularForm::new("been", "be", PastParticiple),
        IrregularForm::new("being", "be", PastTense),
        // have
        IrregularForm::new("has", "have", PastTense),
        IrregularForm::new("had", "have", PastTense),
        IrregularForm::new("having", "have", PastTense),
        // do
        IrregularForm::new("does", "do", PastTense),
        IrregularForm::new("did", "do", PastTense),
        IrregularForm::new("done", "do", PastParticiple),
        IrregularForm::new("doing", "do", PastTense),
        // say
        IrregularForm::new("said", "say", PastTense),
        // go (suppletion: went)
        IrregularForm::new("went", "go", PastTense),
        IrregularForm::new("gone", "go", PastParticiple),
        IrregularForm::new("going", "go", PastTense),
        // get
        IrregularForm::new("got", "get", PastTense),
        IrregularForm::new("gotten", "get", PastParticiple),
        // make
        IrregularForm::new("made", "make", PastTense),
        IrregularForm::new("making", "make", PastTense),
        // take
        IrregularForm::new("took", "take", PastTense),
        IrregularForm::new("taken", "take", PastParticiple),
        IrregularForm::new("taking", "take", PastTense),
        // give
        IrregularForm::new("gave", "give", PastTense),
        IrregularForm::new("given", "give", PastParticiple),
        IrregularForm::new("giving", "give", PastTense),
        // see
        IrregularForm::new("saw", "see", PastTense),
        IrregularForm::new("seen", "see", PastParticiple),
        IrregularForm::new("seeing", "see", PastTense),
        // come
        IrregularForm::new("came", "come", PastTense),
        IrregularForm::new("coming", "come", PastTense),
        // know
        IrregularForm::new("knew", "know", PastTense),
        IrregularForm::new("known", "know", PastParticiple),
        // think
        IrregularForm::new("thought", "think", PastTense),
        // bring
        IrregularForm::new("brought", "bring", PastTense),
        // buy
        IrregularForm::new("bought", "buy", PastTense),
        // catch
        IrregularForm::new("caught", "catch", PastTense),
        // teach
        IrregularForm::new("taught", "teach", PastTense),
        // find
        IrregularForm::new("found", "find", PastTense),
        // tell
        IrregularForm::new("told", "tell", PastTense),
        // sell
        IrregularForm::new("sold", "sell", PastTense),
        // hold
        IrregularForm::new("held", "hold", PastTense),
        // keep
        IrregularForm::new("kept", "keep", PastTense),
        // leave
        IrregularForm::new("left", "leave", PastTense),
        IrregularForm::new("leaving", "leave", PastTense),
        // lose
        IrregularForm::new("lost", "lose", PastTense),
        // mean
        IrregularForm::new("meant", "mean", PastTense),
        // read (unchanged orthographically; phonologically different)
        IrregularForm::new("read", "read", PastTense),
        // send
        IrregularForm::new("sent", "send", PastTense),
        // spend
        IrregularForm::new("spent", "spend", PastTense),
        // build
        IrregularForm::new("built", "build", PastTense),
        // run
        IrregularForm::new("ran", "run", PastTense),
        // write
        IrregularForm::new("wrote", "write", PastTense),
        IrregularForm::new("written", "write", PastParticiple),
        IrregularForm::new("writing", "write", PastTense),
        // speak
        IrregularForm::new("spoke", "speak", PastTense),
        IrregularForm::new("spoken", "speak", PastParticiple),
        // break
        IrregularForm::new("broke", "break", PastTense),
        IrregularForm::new("broken", "break", PastParticiple),
        // choose
        IrregularForm::new("chose", "choose", PastTense),
        IrregularForm::new("chosen", "choose", PastParticiple),
        // begin
        IrregularForm::new("began", "begin", PastTense),
        IrregularForm::new("begun", "begin", PastParticiple),
        // ── Irregular comparatives / superlatives ──
        IrregularForm::new("better", "good", Comparative),
        IrregularForm::new("best", "good", Superlative),
        IrregularForm::new("worse", "bad", Comparative),
        IrregularForm::new("worst", "bad", Superlative),
        IrregularForm::new("more", "much", Comparative),
        IrregularForm::new("most", "much", Superlative),
        IrregularForm::new("less", "little", Comparative),
        IrregularForm::new("least", "little", Superlative),
        IrregularForm::new("further", "far", Comparative),
        IrregularForm::new("furthest", "far", Superlative),
    ]
}

/// Look up an irregular form. Returns every registered entry that
/// matches `surface` (case-insensitive); a surface like `read` may
/// have several entries (different inflectional kinds).
pub fn lookup_irregular(surface: &str) -> Vec<IrregularForm> {
    let needle = surface.to_ascii_lowercase();
    english_irregulars()
        .into_iter()
        .filter(|f| f.surface == needle)
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
    fn high_frequency_irregulars_present() {
        let table = english_irregulars();
        let surfaces: Vec<&str> = table.iter().map(|f| f.surface.as_str()).collect();
        for w in &[
            "children", "men", "women", "feet", "mice", "data", "was", "were", "been", "had",
            "did", "said", "went", "got", "made", "took", "gave", "saw", "came", "knew", "thought",
            "found", "told", "left", "ran", "wrote", "spoke", "broke", "chose", "began", "better",
            "best", "worse", "worst",
        ] {
            assert!(
                surfaces.contains(w),
                "missing high-frequency irregular: {w}"
            );
        }
    }

    #[test]
    fn every_entry_well_formed() {
        for entry in english_irregulars() {
            assert!(!entry.surface.is_empty(), "empty surface");
            assert!(!entry.lemma.is_empty(), "empty lemma in entry {:?}", entry);
            assert!(
                entry.surface != entry.lemma || entry.surface == "read",
                "surface == lemma for non-`read` entry {:?}",
                entry
            );
        }
    }

    #[test]
    fn lookup_returns_all_kinds_for_polysemous_surface() {
        // "read" has both PastTense (read /rɛd/) and is the lemma
        // (PresentTense — read /riːd/). Only the past-tense entry is in
        // the table; the present is the lemma itself.
        let entries = lookup_irregular("read");
        assert!(!entries.is_empty(), "read should be in irregulars");
        for e in &entries {
            assert_eq!(e.lemma, "read");
        }
    }

    #[test]
    fn lookup_case_insensitive() {
        let lower = lookup_irregular("children");
        let upper = lookup_irregular("CHILDREN");
        let mixed = lookup_irregular("Children");
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
        assert!(!lower.is_empty());
    }

    #[test]
    fn lookup_unknown_returns_empty() {
        let entries = lookup_irregular("nonsenseword");
        assert!(entries.is_empty());
    }

    #[test]
    fn children_maps_to_child() {
        let entries = lookup_irregular("children");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].lemma, "child");
        assert_eq!(entries[0].kind, IrregularKind::PluralNoun);
    }

    #[test]
    fn went_maps_to_go() {
        let entries = lookup_irregular("went");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].lemma, "go");
        assert_eq!(entries[0].kind, IrregularKind::PastTense);
    }

    #[test]
    fn better_maps_to_good_comparative() {
        let entries = lookup_irregular("better");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].lemma, "good");
        assert_eq!(entries[0].kind, IrregularKind::Comparative);
    }
}
