//! Schema-Vocabulary Closed-Class Lexicon — runtime loader.
//!
//! Wraps the LMF-encoded bundle at
//! `crates/domains/data/schema-vocabulary/schema_vocabulary.xml`
//! (registered in `praxis.toml` as `schema_vocabulary@2026`, pinned in
//! `praxis.lock`) in a `OnceLock`-cached `BTreeSet<String>` keyed by
//! lowercased written form. The set enumerates the closed-class
//! element/attribute/type/group/model-name vocabulary that appears in
//! published schema specifications (WHATWG HTML Living Standard, W3C
//! XML 1.0, LRC USLM XML User Guide) but lies outside English
//! WordNet's general-language coverage.
//!
//! Used by [`super::english_projection::is_schema_vocabulary`] to
//! recognize these tokens when a loaded XSD schema's
//! `<xs:element>` / `<xs:complexType>` / `<xs:attribute>` /
//! `<xs:attributeGroup>` / `<xs:group>` names are projected through
//! the English-projection functor. Per "Bottom-up loaded, never
//! encoded": the recognition vocabulary is loaded from the registered
//! `schema_vocabulary@2026` source — not hard-coded as a Rust string
//! match.
//!
//! # Literature
//!
//! Per the bundled XML's header — citations live alongside each
//! synset in the data file. The high-level sources are:
//!
//! - **WHATWG (current edition)** *HTML Living Standard*
//!   <https://html.spec.whatwg.org/>. CC-BY 4.0. Element / attribute
//!   names (br, img, del, meta, href, src, alt, colspan, rowspan).
//! - **Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (eds.) (2008)**
//!   *Extensible Markup Language (XML) 1.0 (Fifth Edition)*, W3C
//!   Recommendation 26 November 2008. §3.3.1 attribute types (idref);
//!   §1.1 origin (xml namespace prefix).
//! - **Office of the Law Revision Counsel, U.S. House (2024)** *USLM
//!   XML User Guide* v1.0.18. Published under 1 U.S.C. § 204; public
//!   domain per 17 U.S.C. § 105. Element / attribute / type-suffix /
//!   group-reference names (toc, num, pos, inline, def, enum, attrs,
//!   misc, usc).
//! - **Huddleston, Rodney & Geoffrey K. Pullum (2002)** *The
//!   Cambridge Grammar of the English Language* Cambridge University
//!   Press, Ch. 19 §1.2 — productive prefixation (sub- as one of
//!   the canonical productive prefixes). Sub-* hierarchical forms
//!   (subarticle, subparagraph, subclause, subitem, subsubitem).
//! - **Chiarcos, Christian & Maria Sukhareva (2015)** "OLiA —
//!   Ontologies of Linguistic Annotation" *Semantic Web Journal*
//!   6(4):379-386 — POS taxonomy used in the bundled LMF entries.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use std::sync::OnceLock;

/// The lazily-loaded set of every lemma written-form in the bundled
/// Schema-Vocabulary Closed-Class Lexicon, lowercased.
///
/// Caches on first call. Internal parser failure (XML malformed) is a
/// build-time invariant — the bundle ships with praxis — and panics
/// rather than silently returning an empty set, so any regression is
/// immediately visible.
///
/// Same `OnceLock<BTreeSet<String>>` pattern as
/// [`super::super::super::super::social::judicial::statute_structure::us_legal_lexicon::legal_lexicon`]
/// for the legal-lexicon and
/// [`super::super::super::super::social::judicial::statute_structure::term_extractor::english_stopwords`]
/// for the function-word lexicon.
pub(crate) fn schema_vocabulary() -> &'static alloc::collections::BTreeSet<String> {
    static LEXICON: OnceLock<alloc::collections::BTreeSet<String>> = OnceLock::new();
    LEXICON.get_or_init(|| {
        const XML: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/schema-vocabulary/schema_vocabulary.xml"
        ));
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(XML).expect(
            "bundled crates/domains/data/schema-vocabulary/schema_vocabulary.xml \
                 failed to parse — build-time invariant violated",
        );
        wn.entries
            .iter()
            .map(|e| e.lemma.written_form.to_lowercase())
            .collect()
    })
}

/// True iff `lemma` is recognized in the Schema-Vocabulary
/// Closed-Class Lexicon. Comparison is lowercase.
///
/// This is the entry point [`super::english_projection::is_schema_vocabulary`]
/// consults to recognize XSD-declared element / attribute / type /
/// group / model-name vocabulary when the per-name tokens fail to
/// resolve through WordNet AND are not classifiable as statutory-
/// terms-of-art.
///
/// Per "Bottom-up loaded, never encoded" — the recognition vocabulary
/// is loaded from the registered `schema_vocabulary@2026` source, not
/// hard-coded as a Rust string match.
pub fn is_in_schema_vocabulary(lemma: &str) -> bool {
    let lowered = lemma.to_lowercase();
    schema_vocabulary().contains(&lowered)
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
        let lex = schema_vocabulary();
        assert!(
            !lex.is_empty(),
            "schema_vocabulary must load to a non-empty set; got 0 entries"
        );
    }

    #[test]
    fn lexicon_lowercased() {
        // Every entry is stored lowercase so case-fold matching works.
        for entry in schema_vocabulary() {
            assert_eq!(entry, &entry.to_lowercase());
        }
    }

    // ── Axiom: WHATWG HTML Living Standard element coverage ──────────

    #[test]
    fn axiom_whatwg_html_element_names_present() {
        // WHATWG HTML Living Standard <https://html.spec.whatwg.org/>:
        // br (§4.5.27 line break), img (§4.8.3 image embed), del
        // (§4.7.2 deleted content), meta (§4.2.5 metadata).
        for el in ["br", "img", "del", "meta"] {
            assert!(
                is_in_schema_vocabulary(el),
                "WHATWG HTML element name missing: {el:?}"
            );
        }
    }

    #[test]
    fn axiom_whatwg_html_attribute_names_present() {
        // WHATWG HTML Living Standard attribute names: href (§6.4),
        // src (§6.3), alt (§4.8.4.4), colspan / rowspan (§4.9.11).
        for at in ["href", "src", "alt", "colspan", "rowspan"] {
            assert!(
                is_in_schema_vocabulary(at),
                "WHATWG HTML attribute name missing: {at:?}"
            );
        }
    }

    // ── Axiom: W3C XML 1.0 attribute-type names ──────────────────────

    #[test]
    fn axiom_w3c_xml10_attribute_types_present() {
        // W3C XML 1.0 (Fifth Edition) §3.3.1 attribute types: idref.
        // The xml namespace prefix itself per §1.1 origin.
        for term in ["idref", "xml"] {
            assert!(
                is_in_schema_vocabulary(term),
                "W3C XML 1.0 term missing: {term:?}"
            );
        }
    }

    // ── Axiom: LRC USLM User Guide v1.0.18 element/attribute names ───

    #[test]
    fn axiom_lrc_uslm_element_names_present() {
        // LRC USLM XML User Guide v1.0.18 element names:
        // toc (§V.6 table of contents), num (§V.2 numeric identifier),
        // inline (§V.3 inline content marker), def (§V.5 definition).
        for el in ["toc", "num", "inline", "def"] {
            assert!(
                is_in_schema_vocabulary(el),
                "LRC USLM element name missing: {el:?}"
            );
        }
    }

    #[test]
    fn axiom_lrc_uslm_attribute_names_present() {
        // LRC USLM XML User Guide v1.0.18 attribute names + group
        // refs + type suffixes + domain identifier:
        // pos (§V.4 page-position), enum (§V.7 enumeration),
        // attrs (§V.1 attribute-group ref), misc (§V.8 miscellaneous),
        // usc (§I.2 U.S. Code identifier).
        for term in ["pos", "enum", "attrs", "misc", "usc"] {
            assert!(
                is_in_schema_vocabulary(term),
                "LRC USLM term missing: {term:?}"
            );
        }
    }

    // ── Axiom: Huddleston & Pullum 2002 productive sub-* forms ───────

    #[test]
    fn axiom_productive_sub_prefixed_forms_present() {
        // Huddleston & Pullum 2002 Cambridge Grammar of English
        // Language Ch. 19 §1.2 productive prefixation: sub- attaches
        // productively to base lemmas (article, paragraph, clause,
        // item) each of which is in WordNet. The sub-prefixed forms
        // appear in the LRC USLM hierarchy per USLM User Guide §V.10.
        for sub in [
            "subarticle",
            "subparagraph",
            "subclause",
            "subitem",
            "subsubitem",
        ] {
            assert!(
                is_in_schema_vocabulary(sub),
                "productive sub-* form missing: {sub:?}"
            );
        }
    }

    // ── Axiom: sub-* decomposition leaves a base lemma ───────────────

    /// Huddleston & Pullum 2002 §19.5: every sub-prefixed form
    /// decomposes cleanly into "sub" + base lemma. The bundled entries
    /// `subarticle / subparagraph / subclause / subitem / subsubitem`
    /// each carry a base lemma (`article / paragraph / clause / item`)
    /// — `subsubitem` reduces to `subitem`, which further reduces to
    /// `item`. The axiom test below asserts the canonical
    /// decomposition.
    #[test]
    fn axiom_sub_decomposition_canonical_pairs() {
        let pairs: &[(&str, &str)] = &[
            ("subarticle", "article"),
            ("subparagraph", "paragraph"),
            ("subclause", "clause"),
            ("subitem", "item"),
            ("subsubitem", "subitem"),
        ];
        for (whole, base) in pairs {
            assert!(
                whole
                    .strip_prefix("sub")
                    .map(|tail| tail == *base)
                    .unwrap_or(false),
                "{whole:?} does not strip-prefix to {base:?}: \
                 productive sub-* decomposition broken"
            );
            // The base lemma must be a real English content word
            // we'd expect WordNet to know. Cross-checked by the
            // english_projection layer; here we just assert the
            // string-level decomposition is well-formed.
            assert!(
                !base.is_empty(),
                "base lemma for {whole:?} must be non-empty"
            );
        }
    }

    // ── Axiom: case-insensitivity ─────────────────────────────────────

    #[test]
    fn axiom_lookup_is_case_insensitive() {
        // Per the lookup contract, case-fold matching applies.
        assert!(is_in_schema_vocabulary("IMG"));
        assert!(is_in_schema_vocabulary("Img"));
        assert!(is_in_schema_vocabulary("img"));
        assert!(is_in_schema_vocabulary("RowSpan"));
        assert!(is_in_schema_vocabulary("HREF"));
    }

    #[test]
    fn axiom_empty_input_rejected() {
        // Empty / whitespace inputs are not in the lexicon.
        assert!(!is_in_schema_vocabulary(""));
    }

    // ── Negative axiom: general-English words ARE NOT in the lexicon ─

    #[test]
    fn axiom_general_english_not_in_lexicon() {
        // The lexicon is *complementary* to WordNet — it must not
        // contain general-English open-class words that WordNet
        // already covers.
        for general in ["section", "type", "element", "attribute", "schema"] {
            assert!(
                !is_in_schema_vocabulary(general),
                "general-English word {general:?} leaked into schema vocabulary — \
                 it must stay in WordNet"
            );
        }
    }

    // ── Functor laws: lookup is a faithful projection ────────────────
    //
    // The classifier `is_in_schema_vocabulary: String → Bool` is a
    // functor from the discrete category of lowercase strings to
    // {true, false}. Functor laws to check:
    //   1. Identity preservation: lookup(x) == lookup(x) — referential
    //      transparency (no internal mutation across calls).
    //   2. Composition consistency: lookup(case_fold(x)) == lookup(x)
    //      for all x — the case-fold normalisation factors through.
    //
    // Mirrors the parallel functor-law tests in
    // `us_legal_lexicon::tests::functor_law_*` (uniform test depth per
    // `feedback_uniform_test_depth_across_ontologies`).

    #[test]
    fn functor_law_identity_preservation() {
        for x in ["img", "toc", "subarticle", "ZZZNotPresent"] {
            let a = is_in_schema_vocabulary(x);
            let b = is_in_schema_vocabulary(x);
            let c = is_in_schema_vocabulary(x);
            assert_eq!(a, b);
            assert_eq!(b, c);
        }
    }

    #[test]
    fn functor_law_case_fold_factors_through() {
        for x in ["IMG", "Toc", "SubArticle", "ColSpan", "href"] {
            let direct = is_in_schema_vocabulary(x);
            let folded = is_in_schema_vocabulary(&x.to_lowercase());
            assert_eq!(
                direct, folded,
                "case-fold factor failed on {x:?}: direct={direct} folded={folded}"
            );
        }
    }

    // ── Concurrency: OnceLock thread-safety ──────────────────────────

    #[test]
    fn concurrency_lazy_init_under_threads() {
        // Every thread observes the same atomic init.
        use std::sync::{Arc, Barrier};
        use std::thread;

        const N_THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(N_THREADS));
        let mut handles = Vec::with_capacity(N_THREADS);
        for _ in 0..N_THREADS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait();
                let set = schema_vocabulary();
                // Every thread sees the same content.
                assert!(set.contains("img"));
                assert!(set.contains("toc"));
                assert!(set.contains("subarticle"));
                set.len()
            }));
        }
        let sizes: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let first = sizes[0];
        for s in &sizes {
            assert_eq!(*s, first, "thread observed different vocabulary-set size");
        }
    }

    // ── Load idempotence: reading the bundle file twice equals once ──

    #[test]
    fn load_idempotence_two_reads_equal() {
        // Reading the bundle file twice gives identical typed data —
        // the LMF reader is a deterministic functor (its source is the
        // immutable embedded XML string).
        let a = schema_vocabulary();
        let b = schema_vocabulary();
        // OnceLock guarantees identity, but assert content-equality
        // too in case the OnceLock is bypassed in future.
        assert_eq!(a.len(), b.len());
        for x in a {
            assert!(b.contains(x));
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
            let direct = is_in_schema_vocabulary(&x);
            let folded = is_in_schema_vocabulary(&x.to_lowercase());
            prop_assert_eq!(direct, folded);
        }

        /// Property: lookup is total — every input produces a Boolean
        /// without panic (no internal index-out-of-bounds etc.).
        #[test]
        fn property_total_function(x in arb_lemma()) {
            let _ = is_in_schema_vocabulary(&x);
        }

        /// Property: every lemma the loader stores is recognized. For
        /// every word `w` in the loaded set, `is_in_schema_vocabulary(w)`
        /// is true.
        #[test]
        fn property_every_stored_lemma_recognized(idx in 0usize..256) {
            let lex = schema_vocabulary();
            if lex.is_empty() {
                return Ok(());
            }
            let n = lex.len();
            let i = idx % n;
            let entry = lex.iter().nth(i).unwrap();
            prop_assert!(
                is_in_schema_vocabulary(entry),
                "stored entry {entry:?} not recognized by lookup"
            );
        }

        /// Property: empty / whitespace-only strings are not recognized.
        #[test]
        fn property_empty_not_recognized(_x in any::<u8>()) {
            prop_assert!(!is_in_schema_vocabulary(""));
            prop_assert!(!is_in_schema_vocabulary(" "));
            prop_assert!(!is_in_schema_vocabulary("\t"));
        }

        /// Property: load idempotence — repeated reads return identical
        /// content. The OnceLock guarantees object identity; this
        /// property checks logical equality so a future refactor that
        /// bypasses OnceLock would still preserve the contract.
        #[test]
        fn property_load_idempotent(_seed in any::<u32>()) {
            let a = schema_vocabulary();
            let b = schema_vocabulary();
            prop_assert_eq!(a.len(), b.len());
            for x in a {
                prop_assert!(b.contains(x));
            }
        }
    }
}
