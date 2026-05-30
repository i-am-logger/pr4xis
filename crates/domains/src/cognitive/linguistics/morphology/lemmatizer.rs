//! Lemmatizer — given a surface word form, produce the candidate
//! lemmas (canonical citation forms).
//!
//! Pipeline (Plisson, Lavrac & Mladenic 2004 — "A Rule Based
//! Approach to Word Lemmatization", *Proc. IS-2004*):
//!
//! 1. **Identity** — `surface` is itself a candidate lemma.
//! 2. **Irregular lookup** — consult the per-language irregular
//!    table. Suppletive and umlaut-driven inflection (`went` →
//!    `go`, `children` → `child`) is data, not derivable from rule.
//! 3. **Rule inversion** — for every per-language morphological
//!    rule, invoke [`super::MorphologicalRule::invert`] on
//!    `surface` to get the bare-strip candidate.
//! 4. **Allomorphy expansion** — for every per-language allomorphy
//!    rule, apply [`super::allomorphy::AllomorphyRule::expand`] to
//!    each bare-strip candidate to recover stems that triggered
//!    silent-e, doubled-consonant, y/i, etc. alternations.
//! 5. **Deduplication and lower-casing** — the final candidate set
//!    is normalised so case-only variants collapse and the surface
//!    itself appears at most once.
//!
//! The output is a list of typed [`Form`]s (ontolex:Form, BCP 47
//! language-tagged) — never bare `String` — so downstream consumers
//! (the statute → English adjunction) preserve typing through the
//! whole pipeline.
//!
//! # Language dispatch
//!
//! The [`Language`] enum carries one variant per supported
//! language. Adding a language = adding a variant + a
//! `morphology::<lang>` submodule with `rules()` /
//! `allomorphy_rules()` / `lookup_irregular()` exposed.
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

use super::allomorphy::AllomorphyRule;
use super::{Affix, MorphologicalRule, english};
use crate::cognitive::linguistics::lemon::lexicon::Form;

/// Supported lemmatisation languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// English. Uses `morphology::english::*`.
    English,
}

impl Language {
    /// BCP 47 tag for this language.
    pub fn bcp47_tag(self) -> &'static str {
        match self {
            Self::English => "en",
        }
    }

    fn rules(self) -> Vec<MorphologicalRule> {
        match self {
            Self::English => english::english_rules(),
        }
    }

    fn allomorphy_rules(self) -> Vec<AllomorphyRule> {
        match self {
            Self::English => english::english_allomorphy_rules(),
        }
    }

    fn lookup_irregular(self, surface: &str) -> Vec<String> {
        match self {
            Self::English => english::lookup_irregular(surface)
                .into_iter()
                .map(|e| e.lemma)
                .collect(),
        }
    }
}

/// Lemmatize a surface form into candidate lemmas.
///
/// `surface` may be any case; the pipeline lower-cases internally.
/// The returned `Form`s carry `lang = language.bcp47_tag()`. The
/// surface itself is always included as the first candidate (so
/// downstream direct lookup runs first).
///
/// # Examples
///
/// Inflected plural via the -s rule:
///
/// ```
/// use pr4xis_domains::cognitive::linguistics::morphology::lemmatizer::{
///     lemmatize, Language,
/// };
/// let lemmas: Vec<String> = lemmatize("rights", Language::English)
///     .into_iter()
///     .map(|f| f.written_rep)
///     .collect();
/// assert!(lemmas.contains(&"rights".to_string())); // identity
/// assert!(lemmas.contains(&"right".to_string()));  // -s strip
/// ```
///
/// Irregular form via the dual-route lookup (Pinker 1991):
///
/// ```
/// use pr4xis_domains::cognitive::linguistics::morphology::lemmatizer::{
///     lemmatize, Language,
/// };
/// let lemmas: Vec<String> = lemmatize("children", Language::English)
///     .into_iter()
///     .map(|f| f.written_rep)
///     .collect();
/// assert!(lemmas.contains(&"child".to_string()));
/// ```
///
/// Derivational chains via the fixed-point loop — the `inter-`
/// locative prefix and the `-ize` verb-forming suffix (added for the
/// USC Title-2 gap audit) recover the base lexeme:
///
/// ```
/// use pr4xis_domains::cognitive::linguistics::morphology::lemmatizer::{
///     lemmatize, Language,
/// };
/// let inter: Vec<String> = lemmatize("interparliamentary", Language::English)
///     .into_iter().map(|f| f.written_rep).collect();
/// assert!(inter.contains(&"parliamentary".to_string())); // strip inter-
///
/// let annual: Vec<String> = lemmatize("annualized", Language::English)
///     .into_iter().map(|f| f.written_rep).collect();
/// assert!(annual.contains(&"annual".to_string())); // -ed → -ize → annual
/// ```
///
/// Empty input is well-defined:
///
/// ```
/// use pr4xis_domains::cognitive::linguistics::morphology::lemmatizer::{
///     lemmatize, Language,
/// };
/// assert!(lemmatize("", Language::English).is_empty());
/// ```
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
    for lemma in language.lookup_irregular(&canon) {
        push(lemma, &mut seen, &mut out);
    }

    // (3) + (4) Rule inversion + allomorphy expansion, iterated to
    // fixed-point so chained derivations resolve. Words like
    // "nonenforceability" need TWO rule passes: strip `non-` to
    // "enforceability", then strip `-ability` to "enforce". Bauer
    // (1983) §6.5 documents prefix-suffix interaction in productive
    // English derivation (e.g. non-/un- + -ability/-ment chains).
    //
    // Iteration is bounded: each pass either grows the candidate
    // set or stops. With ≤2 prefixes and ≤4 suffixes typically
    // chained per legal-text derivation (Bauer ch.7), 4 passes is
    // a hard cap that captures all real chains without unbounded
    // growth.
    let allomorphy = language.allomorphy_rules();
    const MAX_PASSES: usize = 4;
    let mut pass = 0;
    let mut start_idx = 0;
    while pass < MAX_PASSES {
        let end_idx = out.len();
        if start_idx >= end_idx {
            break;
        }
        // Snapshot the surfaces to invert this pass — only the new
        // candidates from the previous pass (or the seed on pass 0).
        let surfaces: Vec<String> = out[start_idx..end_idx]
            .iter()
            .map(|f| f.written_rep.clone())
            .collect();
        for surface in &surfaces {
            for rule in language.rules() {
                let bare_candidates = rule.invert(surface);
                for bare in &bare_candidates {
                    push(bare.clone(), &mut seen, &mut out);
                }
                if let Affix::Suffix(suf) = &rule.affix {
                    for bare in &bare_candidates {
                        for allo in &allomorphy {
                            let extras = (allo.expand)(bare, surface, &suf.text);
                            for extra in extras {
                                push(extra, &mut seen, &mut out);
                            }
                        }
                    }
                    for allo in &allomorphy {
                        let extras = (allo.expand)("", surface, &suf.text);
                        for extra in extras {
                            push(extra, &mut seen, &mut out);
                        }
                    }
                }
            }
        }
        start_idx = end_idx;
        pass += 1;
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

    #[test]
    fn canonical_form_passes_through() {
        let lemmas = lemma_strings(&lemmatize("right", Language::English));
        assert_eq!(lemmas.first(), Some(&"right".to_string()));
    }

    // ── Property-based invariants ──────────────────────────────────

    use proptest::prelude::*;

    fn arb_lowercase_word() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop::char::range('a', 'z'), 2..16)
            .prop_map(|chars| chars.into_iter().collect())
    }

    // ── Concurrency: lemmatize must be thread-safe ────────────────
    //
    // The English rule-set is constructed per-call (cheap), but the
    // function still touches static-borrowed irregular-form data
    // and may race with itself. Spawn N threads, each calling
    // lemmatize concurrently; assert all observe identical output.

    #[test]
    fn concurrency_lemmatize_thread_safe() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        const N_THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(N_THREADS));
        let mut handles = Vec::with_capacity(N_THREADS);
        for _ in 0..N_THREADS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait();
                lemmatize("rights", Language::English)
                    .into_iter()
                    .map(|f| f.written_rep)
                    .collect::<Vec<_>>()
            }));
        }
        let results: Vec<Vec<String>> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let first = results[0].clone();
        for r in &results {
            assert_eq!(*r, first, "thread observed different lemmatize output");
        }
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
