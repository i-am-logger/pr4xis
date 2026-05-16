//! Lemmatizer — given a surface word form, produce the candidate
//! lemmas (canonical citation forms).
//!
//! Pipeline (Plisson, Lavrac & Mladenic 2004 — "A Rule Based
//! Approach to Word Lemmatization", *Proc. IS-2004*):
//!
//! 1. **Identity** — `surface` is itself a candidate lemma. If
//!    `surface` is already canonical (e.g. `right`, `walk`), the
//!    direct WordNet lookup downstream will succeed and we skip
//!    rule application.
//! 2. **Irregular lookup** — consult
//!    [`super::irregular::lookup_irregular`]. Suppletive and
//!    umlaut-driven inflection (`went` → `go`, `children` → `child`)
//!    is data, not derivable from rule.
//! 3. **Rule inversion** — for every
//!    [`super::MorphologicalRule`] in [`super::english_rules`],
//!    invoke [`super::MorphologicalRule::invert`] on `surface`. The
//!    rule's allomorphy patches handle silent-e, doubled-consonant
//!    and y/i alternations.
//! 4. **Deduplication and lower-casing** — the final candidate set
//!    is normalised so case-only variants collapse and the surface
//!    itself appears at most once.
//!
//! The output is a list of typed [`Form`]s (ontolex:Form, BCP 47
//! language-tagged) — never bare `String` — so downstream consumers
//! (the statute → English adjunction) preserve typing through the
//! whole pipeline.
//!
//! # Literature
//!
//! - **Plisson, J., Lavrac, N. & Mladenic, D. (2004)** "A Rule
//!   Based Approach to Word Lemmatization", *Proc. IS-2004*
//!   — the lookup-then-rules ordering.
//! - **Beesley, K. & Karttunen, L. (2003)** *Finite-State
//!   Morphology*, CSLI — finite-state framing of analyse/generate.
//! - **Pinker, S. (1991)** "Rules of Language", *Science* 253 —
//!   dual-route motivation for keeping the irregular table.
//! - **Manning, C. & Schütze, H. (1999)** *Foundations of
//!   Statistical Natural Language Processing*, MIT Press, Ch. 4
//!   — practical NLP lemmatisation pipelines.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use alloc::collections::BTreeSet;

use super::{english_rules, irregular};
use crate::cognitive::linguistics::lemon::lexicon::Form;

/// Supported lemmatisation languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// English. Uses [`super::english_rules`] +
    /// [`super::irregular::english_irregulars`].
    English,
}

impl Language {
    /// BCP 47 tag for this language.
    pub fn bcp47_tag(self) -> &'static str {
        match self {
            Self::English => "en",
        }
    }
}

/// Lemmatize a surface form into candidate lemmas.
///
/// `surface` may be any case; the pipeline lower-cases internally.
/// The returned `Form`s carry `lang = language.bcp47_tag()`. The
/// surface itself is always included as the first candidate (so
/// downstream direct lookup runs first).
pub fn lemmatize(surface: &str, language: Language) -> Vec<Form> {
    let canon = surface.to_ascii_lowercase();
    if canon.is_empty() {
        return Vec::new();
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<Form> = Vec::new();
    let push = |s: String, seen: &mut BTreeSet<String>, out: &mut Vec<Form>| {
        if s.is_empty() {
            return;
        }
        if !seen.insert(s.clone()) {
            return;
        }
        out.push(Form {
            written_rep: s,
            lang: language.bcp47_tag().to_string(),
        });
    };

    // (1) Identity.
    push(canon.clone(), &mut seen, &mut out);

    // (2) Irregular lookup.
    match language {
        Language::English => {
            for entry in irregular::lookup_irregular(&canon) {
                push(entry.lemma, &mut seen, &mut out);
            }
        }
    }

    // (3) Rule inversion.
    match language {
        Language::English => {
            for rule in english_rules() {
                for cand in rule.invert(&canon) {
                    push(cand, &mut seen, &mut out);
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lemma_strings(forms: &[Form]) -> Vec<String> {
        forms.iter().map(|f| f.written_rep.clone()).collect()
    }

    #[test]
    fn bcp47_tag_for_english() {
        assert_eq!(Language::English.bcp47_tag(), "en");
    }

    #[test]
    fn empty_input_yields_no_lemmas() {
        assert!(lemmatize("", Language::English).is_empty());
    }

    #[test]
    fn surface_is_first_candidate() {
        let forms = lemmatize("rights", Language::English);
        assert_eq!(
            forms.first().map(|f| f.written_rep.as_str()),
            Some("rights")
        );
    }

    #[test]
    fn lower_cases_input() {
        let forms = lemmatize("RIGHTS", Language::English);
        assert_eq!(
            forms.first().map(|f| f.written_rep.as_str()),
            Some("rights")
        );
    }

    #[test]
    fn every_form_carries_en_tag() {
        for form in lemmatize("rights", Language::English) {
            assert_eq!(form.lang, "en");
        }
    }

    // ── Rule-driven inversion ──────────────────────────────────────

    #[test]
    fn rights_to_right() {
        let lemmas = lemma_strings(&lemmatize("rights", Language::English));
        assert!(lemmas.contains(&"right".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn cities_to_city() {
        let lemmas = lemma_strings(&lemmatize("cities", Language::English));
        assert!(lemmas.contains(&"city".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn remedies_to_remedy() {
        let lemmas = lemma_strings(&lemmatize("remedies", Language::English));
        assert!(lemmas.contains(&"remedy".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn filed_to_file() {
        let lemmas = lemma_strings(&lemmatize("filed", Language::English));
        assert!(lemmas.contains(&"file".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn testified_to_testify() {
        let lemmas = lemma_strings(&lemmatize("testified", Language::English));
        assert!(lemmas.contains(&"testify".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn providing_to_provide() {
        let lemmas = lemma_strings(&lemmatize("providing", Language::English));
        assert!(lemmas.contains(&"provide".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn running_to_run() {
        let lemmas = lemma_strings(&lemmatize("running", Language::English));
        assert!(lemmas.contains(&"run".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn unhappy_to_happy() {
        let lemmas = lemma_strings(&lemmatize("unhappy", Language::English));
        assert!(lemmas.contains(&"happy".to_string()), "got {lemmas:?}");
    }

    // ── Irregular-driven inversion ─────────────────────────────────

    #[test]
    fn children_to_child() {
        let lemmas = lemma_strings(&lemmatize("children", Language::English));
        assert!(lemmas.contains(&"child".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn mice_to_mouse() {
        let lemmas = lemma_strings(&lemmatize("mice", Language::English));
        assert!(lemmas.contains(&"mouse".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn went_to_go() {
        let lemmas = lemma_strings(&lemmatize("went", Language::English));
        assert!(lemmas.contains(&"go".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn was_to_be() {
        let lemmas = lemma_strings(&lemmatize("was", Language::English));
        assert!(lemmas.contains(&"be".to_string()), "got {lemmas:?}");
    }

    #[test]
    fn better_to_good() {
        let lemmas = lemma_strings(&lemmatize("better", Language::English));
        assert!(lemmas.contains(&"good".to_string()), "got {lemmas:?}");
    }

    // ── Deduplication ──────────────────────────────────────────────

    #[test]
    fn deduplicates_repeated_candidates() {
        // "boxes" can be reached by both the direct -s strip (yielding
        // "boxe") and the -es-restoration allomorphy ("box"). Make
        // sure neither "boxe" nor "box" appears more than once.
        let lemmas = lemma_strings(&lemmatize("boxes", Language::English));
        let unique: BTreeSet<&String> = lemmas.iter().collect();
        assert_eq!(unique.len(), lemmas.len(), "duplicates in {lemmas:?}");
    }

    #[test]
    fn lemma_set_never_contains_empty_string() {
        for input in &["rights", "cities", "filed", "running", "went", "children"] {
            let lemmas = lemma_strings(&lemmatize(input, Language::English));
            assert!(
                !lemmas.iter().any(|s| s.is_empty()),
                "empty lemma from {input}: {lemmas:?}"
            );
        }
    }

    // ── Already-canonical inputs ───────────────────────────────────

    #[test]
    fn canonical_form_passes_through() {
        // "right" is already canonical — surface should be first; no
        // crash. Other candidates are accidental (`right` doesn't end
        // in any of our affixes) so we mostly get back just `right`.
        let lemmas = lemma_strings(&lemmatize("right", Language::English));
        assert_eq!(lemmas.first(), Some(&"right".to_string()));
    }

    // ── Property-style coverage ────────────────────────────────────

    // ── Property-based invariants ──────────────────────────────────
    //
    // Beesley & Karttunen (2003) Ch. 3: the inverse-application
    // semantics demand specific universal properties hold for every
    // surface form, not just hand-picked examples. proptest exercises
    // these laws over randomly-generated lowercase-ASCII strings.

    use proptest::prelude::*;

    fn arb_lowercase_word() -> impl Strategy<Value = String> {
        // 2..16 lowercase ASCII letters — covers realistic English-
        // word lengths without exploding the search space.
        proptest::collection::vec(prop::char::range('a', 'z'), 2..16)
            .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        #[test]
        fn property_lemmatize_is_deterministic(word in arb_lowercase_word()) {
            let a = lemmatize(&word, Language::English);
            let b = lemmatize(&word, Language::English);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn property_lemmatize_first_is_surface(word in arb_lowercase_word()) {
            let lemmas = lemmatize(&word, Language::English);
            prop_assert!(!lemmas.is_empty());
            prop_assert_eq!(&lemmas[0].written_rep, &word);
        }

        #[test]
        fn property_lemmatize_no_empty_candidates(word in arb_lowercase_word()) {
            for f in lemmatize(&word, Language::English) {
                prop_assert!(!f.written_rep.is_empty());
            }
        }

        #[test]
        fn property_lemmatize_all_forms_are_en(word in arb_lowercase_word()) {
            for f in lemmatize(&word, Language::English) {
                prop_assert_eq!(f.lang, "en");
            }
        }

        #[test]
        fn property_lemmatize_no_duplicates(word in arb_lowercase_word()) {
            let lemmas = lemmatize(&word, Language::English);
            let unique: BTreeSet<&String> = lemmas.iter().map(|f| &f.written_rep).collect();
            prop_assert_eq!(unique.len(), lemmas.len());
        }
    }

    #[test]
    fn coverage_on_inflected_corpus_terms() {
        // The 17%-of-statute-lemmas gap was driven by these forms.
        // Lemmatize must surface the canonical lemma for every one.
        let cases: &[(&str, &str)] = &[
            ("rights", "right"),
            ("remedies", "remedy"),
            ("cities", "city"),
            ("filed", "file"),
            ("testified", "testify"),
            ("providing", "provide"),
            ("running", "run"),
            ("sitting", "sit"),
            ("baked", "bake"),
            ("burdens", "burden"),
            ("children", "child"),
            ("women", "woman"),
            ("went", "go"),
            ("was", "be"),
            ("better", "good"),
            ("worse", "bad"),
            ("said", "say"),
            ("had", "have"),
            ("did", "do"),
        ];
        for (input, expected_lemma) in cases {
            let lemmas = lemma_strings(&lemmatize(input, Language::English));
            assert!(
                lemmas.contains(&expected_lemma.to_string()),
                "lemmatize({input}) → {lemmas:?}, missing `{expected_lemma}`"
            );
        }
    }
}
