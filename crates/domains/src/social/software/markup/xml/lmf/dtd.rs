//! The Global WordNet **WN-LMF 1.3 DTD** as a praxis-loaded source.
//!
//! WN-LMF is the XML 1.0 DTD that defines WordNet's XML format. The
//! bundled WordNet 2025 bytes declare DOCTYPE pointing at this DTD;
//! every `LexicalResource`, `Lexicon`, `LexicalEntry`, `Synset`,
//! `Sense` and `SyntacticBehaviour` element the WordNet reader
//! emits is declared in this DTD.
//!
//! This module is the typed-layer accessor that "loads like English":
//! a cached `&'static str` of the DTD bytes via [`loaded_wn_lmf_dtd`],
//! mirroring [`crate::formal::meta::xsd::uslm_vocabulary::USLM_1_0_18_XSD`]
//! and [`crate::applied::data_provisioning::registry::data_sources`].
//!
//! ## Citation
//!
//! - **Global Wordnet Association**, *WN-LMF* (Lexical Markup
//!   Framework) specification, <https://globalwordnet.github.io/schemas/>.
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (2008) *Extensible Markup Language (XML) 1.0
//!   (Fifth Edition)*, W3C Recommendation 26 November 2008 — §2.8
//!   document type declaration, §4 physical structures.
//! - **ISO/IEC 24613:2008** *Language resource management — Lexical
//!   Markup Framework (LMF)* — the abstract framework WN-LMF
//!   instantiates for the WordNet family.

/// The raw bytes of the bundled WN-LMF 1.3 DTD, embedded at build
/// time via `include_str!` so the runtime path is hermetic. Matches
/// the `wn_lmf_dtd@1.3` source registered in `praxis.toml`; the
/// pinned sha256 in `praxis.lock` certifies these bytes.
pub const WN_LMF_1_3_DTD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/markup-schemas/lmf/wn_lmf_dtd-1.3.dtd"
));

/// The loaded WN-LMF 1.3 DTD — the schema the bundled WordNet 2025
/// XML bytes declare DOCTYPE against. Downstream code queries this
/// to anchor LMF concept identity in the published DTD rather than
/// in the hand-coded runtime types (`feedback_bottom_up_loaded_not_encoded`).
#[must_use]
pub fn loaded_wn_lmf_dtd() -> &'static str {
    WN_LMF_1_3_DTD
}

/// True iff `name` is the local-name of an element type declared
/// in the bundled WN-LMF 1.3 DTD, looked up ontologically against
/// the [`DtdLens`](crate::formal::meta::dtd::DtdLens)-parsed
/// declaration set (W3C XML 1.0 §3.2 production [45] `elementdecl`).
/// Case-sensitive — XML element names are case-sensitive per W3C
/// XML 1.0 §3.
///
/// The lookup goes through the typed [`DtdSchema`](crate::formal::meta::dtd::DtdSchema):
/// the bundled DTD bytes are parsed once on first call (cached via
/// [`OnceLock`](std::sync::OnceLock)) into the praxis DTD ontology,
/// then names are matched against the parsed [`DtdConcept::ElementDecl`]
/// names — never a substring scan over the source bytes.
#[must_use]
pub fn is_wn_lmf_element(name: &str) -> bool {
    use crate::formal::meta::dtd::ontology::DtdConcept;
    use crate::formal::meta::well_behaved_lens::WellBehavedLens;
    static SCHEMA: std::sync::OnceLock<crate::formal::meta::dtd::DtdSchema> =
        std::sync::OnceLock::new();
    let schema = SCHEMA.get_or_init(|| {
        <crate::formal::meta::dtd::DtdLens as WellBehavedLens>::get(WN_LMF_1_3_DTD.as_bytes())
            .expect("bundled WN-LMF 1.3 DTD must parse cleanly")
    });
    schema
        .of_kind(DtdConcept::ElementDecl)
        .any(|d| d.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dtd_bytes_are_non_empty() {
        assert!(!WN_LMF_1_3_DTD.is_empty());
        assert!(WN_LMF_1_3_DTD.contains("<!ELEMENT"));
        assert!(WN_LMF_1_3_DTD.contains("LexicalResource"));
    }

    #[test]
    fn recognises_canonical_wn_lmf_elements() {
        // These five concepts ground the LMF runtime types in
        // crates/domains/src/social/software/markup/xml/lmf/ontology.rs.
        assert!(is_wn_lmf_element("LexicalResource"));
        assert!(is_wn_lmf_element("Lexicon"));
        assert!(is_wn_lmf_element("LexicalEntry"));
        assert!(is_wn_lmf_element("Synset"));
        assert!(is_wn_lmf_element("Sense"));
    }

    #[test]
    fn rejects_non_wn_lmf_names() {
        assert!(!is_wn_lmf_element("xs:element"));
        assert!(!is_wn_lmf_element("section"));
        assert!(!is_wn_lmf_element(""));
    }

    #[test]
    fn loaded_dtd_matches_praxis_lock_hash() {
        // The hash in praxis.lock is the SHA-256 of the bundled
        // file as-is. This test re-asserts the bytes haven't drifted
        // from what the lock declares — drift triggers
        // LockManifestAgreement failure separately.
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(WN_LMF_1_3_DTD.as_bytes());
        let digest = h.finalize();
        let got: alloc::string::String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            got, "dba306298d63e33f243903edce15d180c018db1a6e2c7e836d52480eb5659796",
            "WN-LMF DTD bytes drifted from praxis.lock pinned hash"
        );
    }

    proptest::proptest! {
        /// `is_wn_lmf_element` is a total function: every input
        /// (arbitrary string) maps to a deterministic bool, never
        /// panics. The classifier is a closed-world membership
        /// query against the parsed [`DtdSchema`]'s ElementDecl set.
        #[test]
        fn prop_is_wn_lmf_element_total(name in "[A-Za-z0-9_:-]{0,32}") {
            let _ = is_wn_lmf_element(&name);
        }

        /// Membership is consistent with the parsed schema: a name
        /// returns true iff it appears in the DtdLens-projected
        /// ElementDecl set. Closes the loop between the ontological
        /// lookup and the underlying parsed evidence.
        #[test]
        fn prop_is_wn_lmf_element_agrees_with_parsed_schema(
            name in "[A-Za-z0-9_:-]{0,32}",
        ) {
            use crate::formal::meta::dtd::ontology::DtdConcept;
            use crate::formal::meta::well_behaved_lens::WellBehavedLens;
            let schema =
                <crate::formal::meta::dtd::DtdLens as WellBehavedLens>::get(
                    WN_LMF_1_3_DTD.as_bytes(),
                )
                .expect("bundled WN-LMF DTD parses");
            let expected = schema
                .of_kind(DtdConcept::ElementDecl)
                .any(|d| d.name == name);
            proptest::prop_assert_eq!(is_wn_lmf_element(&name), expected);
        }
    }
}
