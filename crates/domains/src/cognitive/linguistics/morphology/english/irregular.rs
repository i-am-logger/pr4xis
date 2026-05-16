//! English irregular forms — the dual-route lookup table.
//!
//! Hand-coded against Quirk et al. (1985) §3.21–3.59 + Pinker
//! (1991) until the AGID / WordNet morph-exception data source
//! is wired up in `praxis.toml`. See module-level docs.

#[allow(unused_imports)]
use alloc::{string::ToString, vec, vec::Vec};

use super::super::irregular::{IrregularForm, IrregularKind};

/// High-frequency English irregular forms.
///
/// Coverage: the closed-class irregulars (be/have/do/say/get) plus
/// the most common irregular plurals and strong verbs. The
/// long-form list is intentional repetition for review against
/// Quirk et al. (1985) §3.21–3.59; future replacement by an
/// AGID-loaded source preserves the same data shape.
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

/// English-specific irregular lookup — case-insensitive match
/// against [`english_irregulars`].
pub fn lookup_irregular(surface: &str) -> Vec<IrregularForm> {
    super::super::irregular::lookup_in(surface, &english_irregulars())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // ── Property-based laws for english_irregulars() ──────────────

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn property_every_surface_resolves(_ in 0..1) {
            // For every entry in the table, lookup(surface) must
            // return at least that entry. Pure structural law.
            for entry in english_irregulars() {
                let hits = lookup_irregular(&entry.surface);
                prop_assert!(
                    hits.iter().any(|h| h.lemma == entry.lemma && h.kind == entry.kind),
                    "entry {:?} not findable via its own surface",
                    entry
                );
            }
        }

        #[test]
        fn property_lookup_is_case_insensitive_for_all(_ in 0..1) {
            for entry in english_irregulars() {
                let upper = lookup_irregular(&entry.surface.to_uppercase());
                let mixed = {
                    let mut s = entry.surface.clone();
                    if let Some(first) = s.get_mut(0..1) {
                        first.make_ascii_uppercase();
                    }
                    lookup_irregular(&s)
                };
                prop_assert!(!upper.is_empty(), "uppercase fails for {}", entry.surface);
                prop_assert!(!mixed.is_empty(), "mixed-case fails for {}", entry.surface);
            }
        }

        #[test]
        fn property_lookup_unknown_returns_empty(s in "[a-z]{8,16}") {
            // Random 8-16 char lowercase strings are extremely
            // unlikely to be English irregular surfaces.
            let known: alloc::collections::BTreeSet<String> =
                english_irregulars().into_iter().map(|f| f.surface).collect();
            if !known.contains(&s) {
                let hits = lookup_irregular(&s);
                prop_assert!(hits.is_empty(), "random string {s} matched: {hits:?}");
            }
        }

        #[test]
        fn property_lemmas_never_have_whitespace_or_punctuation(_ in 0..1) {
            for entry in english_irregulars() {
                for c in entry.lemma.chars() {
                    prop_assert!(
                        c.is_ascii_alphabetic(),
                        "lemma `{}` has non-alpha char `{c}`",
                        entry.lemma
                    );
                }
                for c in entry.surface.chars() {
                    prop_assert!(
                        c.is_ascii_alphabetic(),
                        "surface `{}` has non-alpha char `{c}`",
                        entry.surface
                    );
                }
            }
        }
    }
}
