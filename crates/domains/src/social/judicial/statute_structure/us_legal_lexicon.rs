//! U.S. Federal Legal-Text Closed-Class Lexicon — runtime loader.
//!
//! Wraps the LMF-encoded bundle at
//! `crates/domains/data/legal-text/us_legal_lexicon.xml` (registered
//! in `praxis.toml` as `us_legal_lexicon@2026`, pinned in
//! `praxis.lock`) in a `OnceLock`-cached `BTreeSet<String>` keyed by
//! lowercased written form. The set is the union of every
//! `<LexicalEntry><Lemma writtenForm="X" .../>...` token in the file:
//! citation abbreviations, month names, federal-agency acronyms,
//! U.S. state / place names, English productive compounds, and legal
//! terms-of-art.
//!
//! Used by
//! [`super::statute_understanding::is_statutory_term_of_art`] to
//! recognize these tokens as bounded statutory-text vocabulary rather
//! than reporting them as `Unresolved` lemmas.
//!
//! # Literature
//!
//! Per the bundled XML's header — citations live alongside each
//! category in the data file. The high-level sources are:
//!
//! - **U.S. Government Publishing Office (2016)** *GPO Style Manual*
//!   31st ed., Ch. 9 "Abbreviations and Letter Symbols" §9.4, §9.6,
//!   and §9.20 (Months and days). Public domain per 17 U.S.C. § 105.
//! - **Office of the Federal Register (2017)** *Federal Register
//!   Document Drafting Handbook* Ch. 4 §4.1 — federal-agency acronym
//!   enumeration. Public domain.
//! - **Office of the Federal Register (annual)** *U.S. Government
//!   Manual* — federal-agency canonical names. Public domain.
//! - **ISO 3166-2:US (ISO 2013)** — U.S. state + territory codes.
//! - **Huddleston, Rodney & Geoffrey K. Pullum (2002)** *The
//!   Cambridge Grammar of the English Language* Cambridge University
//!   Press, Ch. 7 (adpositions), Ch. 19 §1.2 (productive prefixation)
//!   and §4 (N+N compounding).
//! - **Bauer, Laurie (1983)** *English Word-Formation* Cambridge
//!   University Press, Ch. 6 (productive prefixation and
//!   compounding).
//! - **Garner, Bryan A. (ed.) (2019)** *Black's Law Dictionary* 11th
//!   ed., Thomson Reuters — U.S. legal terms-of-art.
//! - **Chiarcos, Christian & Maria Sukhareva (2015)** "OLiA —
//!   Ontologies of Linguistic Annotation" *Semantic Web Journal*
//!   6(4):379-386 — POS taxonomy.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use std::sync::OnceLock;

/// The lazily-loaded set of every lemma written-form in the bundled
/// U.S. Federal Legal-Text Closed-Class Lexicon, lowercased.
///
/// Caches on first call. Internal parser failure (XML malformed) is a
/// build-time invariant — the bundle ships with praxis — and panics
/// rather than silently returning an empty set, so any regression is
/// immediately visible.
///
/// Same `OnceLock<BTreeSet<String>>` pattern as
/// [`super::term_extractor::english_stopwords`] for the function-word
/// lexicon.
pub(crate) fn legal_lexicon() -> &'static alloc::collections::BTreeSet<String> {
    static LEXICON: OnceLock<alloc::collections::BTreeSet<String>> = OnceLock::new();
    LEXICON.get_or_init(|| {
        // The committed `.prx` — the content-addressed envelope carrying the
        // authored U.S. legal closed-class lexicon. The raw `.xml` is the
        // git-tracked source-of-truth but is EXCLUDED from the published crate;
        // only this `.prx` ships, materialized through the generalized
        // feature-light `[compact_archive_signatures]` gate (phase 2d) — so it
        // loads in the default `std`-only build with no `prx`/gzip feature. The
        // `read_wordnet` reader is unchanged.
        const US_LEGAL_LEXICON_PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/legal-text/us_legal_lexicon.prx"
        ));
        let xml = crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded(
            "us_legal_lexicon",
            "2026",
            US_LEGAL_LEXICON_PRX,
        );
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml).expect(
            "us_legal_lexicon committed .prx bytes failed to parse — build-time invariant violated",
        );
        wn.entries
            .iter()
            .map(|e| e.lemma.written_form.to_lowercase())
            .collect()
    })
}

/// True iff `lemma` is recognized in the U.S. Federal Legal-Text
/// Closed-Class Lexicon. Comparison is lowercase.
///
/// This is the entry point [`super::statute_understanding::is_statutory_term_of_art`]
/// consults after its abbreviation- and section-marker structural
/// checks fail. The lexicon backs the bounded enumeration of citation
/// abbreviations, month names, federal-agency acronyms, U.S. state /
/// place names, English productive compounds, and legal
/// terms-of-art.
///
/// Per "Bottom-up loaded, never encoded" — the recognition vocabulary
/// is loaded from the registered `us_legal_lexicon@2026` source, not
/// hard-coded as a Rust string match.
pub fn is_in_legal_lexicon(lemma: &str) -> bool {
    let lowered = lemma.to_lowercase();
    legal_lexicon().contains(&lowered)
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Loader invariants ────────────────────────────────────────────

    #[test]
    fn lexicon_loads_with_nonzero_entries() {
        // The bundled file is a tracked invariant; loading produces a
        // non-empty set per the documented categories.
        let lex = legal_lexicon();
        assert!(
            !lex.is_empty(),
            "us_legal_lexicon must load to a non-empty set; got 0 entries"
        );
    }

    #[test]
    fn lexicon_lowercased() {
        // Every entry is stored lowercase so case-fold matching works.
        for entry in legal_lexicon() {
            assert_eq!(entry, &entry.to_lowercase());
        }
    }

    // ── Axiom: GPO Style Manual 2016 §9.20 month coverage ────────────

    #[test]
    fn axiom_gpo_style_manual_month_abbreviations_present() {
        // GPO Style Manual 2016 §9.20: months abbreviated as Jan., Feb.,
        // Mar., Apr., May, June, July, Aug., Sept. (or Sep.), Oct., Nov.,
        // Dec. (May/June/July are not abbreviated in GPO style; the rest
        // are.) Each abbreviated form must round-trip through the
        // lexicon.
        for m in [
            "jan", "feb", "mar", "apr", "aug", "sep", "sept", "oct", "nov", "dec",
        ] {
            assert!(
                is_in_legal_lexicon(m),
                "GPO Style Manual §9.20 month abbreviation missing: {m:?}"
            );
        }
    }

    #[test]
    fn axiom_full_month_names_present() {
        // The full month names appear in USC heading text where dates
        // are spelled out ("June 30, 1970"). Each must be recognized.
        for m in [
            "january",
            "february",
            "march",
            "april",
            "june",
            "july",
            "august",
            "september",
            "october",
            "november",
            "december",
        ] {
            assert!(is_in_legal_lexicon(m), "full month name missing: {m:?}");
        }
    }

    // ── Axiom: GPO Style Manual 2016 Ch. 9 citation abbreviations ────

    #[test]
    fn axiom_gpo_citation_abbreviations_present() {
        // "Stat." (Statutes-at-Large), "div." (division), "ch." (chapter),
        // "Pub. L." (Public Law) — the GPO Style Manual 2016 Ch. 9 §9.4
        // tabulated citation abbreviations as they appear in U.S. Code
        // Public-Law citations.
        for abbr in ["stat", "div", "ch", "pl", "pub", "etc", "et", "al"] {
            assert!(
                is_in_legal_lexicon(abbr),
                "GPO Style Manual Ch. 9 citation abbreviation missing: {abbr:?}"
            );
        }
    }

    // ── Axiom: ISO 3166-2:US state names ─────────────────────────────

    #[test]
    fn axiom_iso_3166_us_state_names_present() {
        // ISO 3166-2:US enumerates the 50 U.S. states + DC. Sample
        // states appearing in USC headings.
        for st in [
            "alaska", "hawaii", "kansas", "iowa", "ohio", "oregon", "wyoming", "nebraska",
        ] {
            assert!(
                is_in_legal_lexicon(st),
                "ISO 3166-2:US state name missing: {st:?}"
            );
        }
    }

    #[test]
    fn axiom_iso_3166_us_territories_present() {
        // Five inhabited U.S. territories per ISO 3166-2:US + Compact
        // of Free Association.
        for t in ["puerto", "guam", "mariana", "palau", "micronesia"] {
            assert!(
                is_in_legal_lexicon(t),
                "ISO 3166-2:US territory missing: {t:?}"
            );
        }
    }

    // ── Axiom: Federal Register Document Drafting Handbook agency acronyms ─

    #[test]
    fn axiom_federal_agency_acronyms_present() {
        // Federal Register Document Drafting Handbook 2017 §4.1 lists
        // executive-branch agency acronyms. Sample.
        for ag in ["faa", "gao", "nepa", "foia", "ntsb", "nasa", "icao"] {
            assert!(
                is_in_legal_lexicon(ag),
                "Federal-agency acronym missing: {ag:?}"
            );
        }
    }

    // ── Axiom: Huddleston & Pullum 2002 productive compounds ─────────

    #[test]
    fn axiom_productive_compounds_present() {
        // Huddleston & Pullum 2002 Ch. 19 §4 — N+N compounds and
        // §1.2 — productive prefixed forms; appearing in U.S. Code
        // heading text.
        for comp in [
            "rulemaking",
            "recordkeeping",
            "decisionmaking",
            "intercity",
            "intermodal",
            "multimodal",
            "interagency",
            "nonmailable",
            "multiyear",
        ] {
            assert!(
                is_in_legal_lexicon(comp),
                "productive English compound missing: {comp:?}"
            );
        }
    }

    // ── Axiom: Black's Law Dictionary legal terms-of-art ─────────────

    #[test]
    fn axiom_blacks_law_terms_of_art_present() {
        // Black's Law Dictionary 11th ed. (2019) — bounded U.S. legal
        // terms-of-art.
        for term in ["joinder", "misprision", "interpleader", "seamen"] {
            assert!(
                is_in_legal_lexicon(term),
                "Black's Law Dictionary term-of-art missing: {term:?}"
            );
        }
    }

    // ── Axiom: case-insensitivity ─────────────────────────────────────

    #[test]
    fn axiom_lookup_is_case_insensitive() {
        // Per the lookup contract, case-fold matching applies.
        assert!(is_in_legal_lexicon("OCT"));
        assert!(is_in_legal_lexicon("Oct"));
        assert!(is_in_legal_lexicon("oct"));
        assert!(is_in_legal_lexicon("AmTrak"));
        assert!(is_in_legal_lexicon("FAA"));
    }

    #[test]
    fn axiom_empty_input_rejected() {
        // Empty / whitespace inputs are not in the lexicon.
        assert!(!is_in_legal_lexicon(""));
    }

    // ── Negative axiom: general-English words ARE NOT in the lexicon ──

    #[test]
    fn axiom_general_english_not_in_lexicon() {
        // The lexicon is *complementary* to WordNet — it must not
        // contain general-English open-class words that WordNet
        // already covers. Spot-check by sampling well-known WordNet
        // lemmas.
        for general in ["employer", "retaliation", "statute", "court", "action"] {
            assert!(
                !is_in_legal_lexicon(general),
                "general-English word {general:?} leaked into legal lexicon — \
                 it must stay in WordNet"
            );
        }
    }

    // ── Functor laws: lookup is a faithful projection ────────────────
    //
    // The classifier `is_in_legal_lexicon: String → Bool` is a functor
    // from the discrete category of lowercase strings to {true, false}.
    // Functor laws to check:
    //   1. Identity preservation: lookup(x) == lookup(x) — referential
    //      transparency (no internal mutation across calls).
    //   2. Composition consistency: lookup(case_fold(x)) == lookup(x)
    //      for all x — the case-fold normalisation factors through.

    #[test]
    fn functor_law_identity_preservation() {
        // Same input must always produce same output across repeated
        // calls (OnceLock cache).
        for x in ["faa", "alaska", "rulemaking", "ZZZNotPresent"] {
            let a = is_in_legal_lexicon(x);
            let b = is_in_legal_lexicon(x);
            let c = is_in_legal_lexicon(x);
            assert_eq!(a, b);
            assert_eq!(b, c);
        }
    }

    #[test]
    fn functor_law_case_fold_factors_through() {
        // For every input, the result is invariant under case folding.
        for x in ["FAA", "Alaska", "RuleMaking", "BoEinG", "stat"] {
            let direct = is_in_legal_lexicon(x);
            let folded = is_in_legal_lexicon(&x.to_lowercase());
            assert_eq!(
                direct, folded,
                "case-fold factor failed on {x:?}: direct={direct} folded={folded}"
            );
        }
    }

    // ── Concurrency: OnceLock thread-safety ──────────────────────────

    #[test]
    fn concurrency_lazy_init_under_threads() {
        // Mirror the function-words concurrency test: every thread
        // observes the same atomic init.
        use std::sync::{Arc, Barrier};
        use std::thread;

        const N_THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(N_THREADS));
        let mut handles = Vec::with_capacity(N_THREADS);
        for _ in 0..N_THREADS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait();
                let set = legal_lexicon();
                // Every thread sees the same content.
                assert!(set.contains("faa"));
                assert!(set.contains("stat"));
                assert!(set.contains("rulemaking"));
                set.len()
            }));
        }
        let sizes: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let first = sizes[0];
        for s in &sizes {
            assert_eq!(*s, first, "thread observed different lexicon-set size");
        }
    }

    // ── Proptest: properties over random query strings ───────────────

    use proptest::prelude::*;

    fn arb_lemma() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            prop_oneof![
                prop::char::range('a', 'z'),
                prop::char::range('A', 'Z'),
                prop::char::range('0', '9'),
                Just('-'),
                Just('_'),
            ],
            0..24,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        /// Property: case-folding factors through (idempotence under
        /// case-fold lookup). For every input, looking up `x` and
        /// looking up `x.to_lowercase()` produce the same result.
        #[test]
        fn property_case_fold_factors_through(x in arb_lemma()) {
            let direct = is_in_legal_lexicon(&x);
            let folded = is_in_legal_lexicon(&x.to_lowercase());
            prop_assert_eq!(direct, folded);
        }

        /// Property: lookup is total — every input produces a Boolean
        /// without panic (no internal index-out-of-bounds etc.).
        #[test]
        fn property_total_function(x in arb_lemma()) {
            let _ = is_in_legal_lexicon(&x);
        }

        /// Property: every lemma the loader stores is recognized. For
        /// every word `w` in the loaded set, `is_in_legal_lexicon(w)`
        /// is true. Random sampling — proptest picks a random index
        /// into the loaded set.
        #[test]
        fn property_every_stored_lemma_recognized(idx in 0usize..256) {
            let lex = legal_lexicon();
            if lex.is_empty() {
                return Ok(());
            }
            let n = lex.len();
            // Use modulo so we always hit a valid entry.
            let i = idx % n;
            let entry = lex.iter().nth(i).unwrap();
            prop_assert!(
                is_in_legal_lexicon(entry),
                "stored entry {entry:?} not recognized by lookup"
            );
        }

        /// Property: empty / whitespace-only strings are not recognized
        /// (no entry in the bundled XML has an empty written form).
        #[test]
        fn property_empty_not_recognized(_x in any::<u8>()) {
            prop_assert!(!is_in_legal_lexicon(""));
            prop_assert!(!is_in_legal_lexicon(" "));
            prop_assert!(!is_in_legal_lexicon("\t"));
        }
    }
}
