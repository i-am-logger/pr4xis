//! Bundled W3C XML 1.0 Fifth Edition spec bytes + cached accessor.
//!
//! Mirrors `social::software::markup::xml::mods::schema` (M4.ζ.3)
//! and `lmf::dtd` (M4.ζ.1) — the bundled spec bytes for the
//! `xml_1_0_fifth_edition@2008` source registered in `praxis.toml`;
//! the pinned sha256 in `praxis.lock` certifies these bytes.
//!
//! Downstream code (currently planned: M5.ε.2 codegen for §2.2 Char,
//! M5.ε.3 codegen for §2.3 Name productions, M5.ε.4 well-formedness
//! checks) reads the XML via [`loaded_xml_1_0_fifth_edition`] and
//! parses the embedded `<prod id="NT-X" num="N"><lhs>X</lhs><rhs>...</rhs></prod>`
//! blocks to derive predicates — replacing the hand-coded code-point
//! matches in `parser::grammar`.
//!
//! ## Citation
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (eds.) (2008) *Extensible Markup Language (XML)
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008,
//!   the XML-format source. The bundled file is a byte-for-byte
//!   copy; the `xml_1_0_fifth_edition@2008` hash in `praxis.lock`
//!   is sha256 of those bytes.

/// The bundled W3C XML 1.0 Fifth Edition XML-format bytes — the
/// normative source the parser's grammar predicates derive from
/// (per `feedback_bottom_up_loaded_not_encoded`).
pub const XML_1_0_FIFTH_EDITION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/markup-schemas/xml/xml_1_0_fifth_edition-2008.xml"
));

/// The loaded W3C XML 1.0 Fifth Edition spec bytes. Downstream
/// code queries this to anchor parser predicates in the published
/// EBNF rather than in hand-coded ranges.
#[must_use]
pub fn loaded_xml_1_0_fifth_edition() -> &'static str {
    XML_1_0_FIFTH_EDITION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_spec_starts_with_xml_declaration() {
        // The bundled spec opens with the W3C XML 1.0 §2.8 XML
        // declaration; assert the structural root is present rather
        // than parsing the whole document (parsing is exercised by
        // the codegen tests downstream).
        let bytes = loaded_xml_1_0_fifth_edition();
        assert!(
            bytes.starts_with("<?xml version='1.0' encoding='UTF-8'?>"),
            "bundled spec must begin with the XML 1.0 §2.8 declaration"
        );
        assert!(
            bytes.contains("<!DOCTYPE spec SYSTEM \"xmlspec.dtd\""),
            "bundled spec must use the xmlspec.dtd DOCTYPE"
        );
    }

    #[test]
    fn full_grammar_loads_with_all_live_productions() {
        // M5.ζ.2 acceptance test — every live <prod> block in the
        // bundled W3C XML 1.0 Fifth Edition spec parses to a typed
        // Term tree via pr4xis::xml_grammar::load_grammar. Per
        // feedback_corpus_wide_audit_on_load, this is the corpus-wide
        // audit that surfaces any unresolved spec production at test
        // time — never silently.
        //
        // The spec source contains 86 `<prod>` open-tags total but
        // one (line 2465's `<prod id='NT-ExternalDef'>`) is inside
        // an XML comment — a deleted production from an earlier
        // revision. After load_grammar strips W3C XML 1.0 §2.5
        // comments, the live grammar has exactly 85 productions.
        use pr4xis::xml_grammar::load_grammar;
        let grammar = load_grammar(loaded_xml_1_0_fifth_edition()).expect("load spec grammar");
        assert_eq!(
            grammar.len(),
            85,
            "W3C XML 1.0 Fifth Edition has 85 live EBNF productions \
             (the 86th `<prod id='NT-ExternalDef'>` is commented out at line 2465); \
             every live one must parse via load_grammar"
        );
        // Spot-check the productions M5.ε.2/.3 hand-codegen'd as
        // character-class predicates — they should now also resolve
        // via the full-grammar loader.
        for name in ["document", "Char", "S", "NameStartChar", "NameChar", "Name"] {
            assert!(
                grammar.lookup(name).is_some(),
                "production {name} must be loaded"
            );
        }
        // The deleted production must NOT be present.
        assert!(
            grammar.lookup("ExternalDef").is_none(),
            "ExternalDef is in an XML comment and must be skipped"
        );
    }

    #[test]
    fn spec_bytes_match_lock_hash() {
        use sha2::{Digest, Sha256};
        let bytes = loaded_xml_1_0_fifth_edition();
        let hash = Sha256::digest(bytes.as_bytes());
        let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex, "9f54011039f9a0e5f5629f4312c1e106ebe897707759a1d9ddee9dacf2fcc17a",
            "loaded XML 1.0 Fifth Edition bytes must match the praxis.lock pinned sha256"
        );
    }
}
