//! Bundled W3C XML 1.0 Fifth Edition spec bytes + cached accessor.
//!
//! Mirrors `social::software::markup::xml::mods::schema` (M4.ζ.3)
//! and `lmf::dtd` (M4.ζ.1) — the bundled spec bytes for the
//! `xml_1_0_fifth_edition@2008` source registered in `praxis.toml`;
//! the pinned digest in `praxis.lock` certifies these bytes.
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
//!   is the content digest of those bytes.

/// The committed W3C XML 1.0 Fifth Edition spec `.prx` — the content-addressed
/// envelope carrying the W3C-published XML-format bytes. The raw `.xml` is
/// fetch-only (`pr4xis update`) and ships in NO crate; only this `.prx` is
/// committed + embedded, loaded through the generalized fail-closed
/// `[compact_archive_signatures]` gate (phase 2c). The SAME committed `.prx`
/// `build.rs` decodes at compile time to emit the grammar predicates.
const XML_1_0_FIFTH_EDITION_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/markup-schemas/xml/xml_1_0_fifth_edition-2008.prx"
));

/// The loaded W3C XML 1.0 Fifth Edition spec bytes. Downstream
/// code queries this to anchor parser predicates in the published
/// EBNF rather than in hand-coded ranges.
///
/// The bytes are materialized from the committed `.prx` through the fail-closed
/// `[compact_archive_signatures]` content gate
/// ([`raw_source_text_embedded`](crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded)),
/// cached for the process behind a `OnceLock`. The raw `.xml` is no longer
/// embedded — only the gated `.prx` is.
#[must_use]
pub fn loaded_xml_1_0_fifth_edition() -> &'static str {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    use std::sync::OnceLock;
    // The accessor returns a `Cow` (owned when the payload rides DEFLATE); the
    // `OnceLock` caches the one materialization for the process.
    static SPEC: OnceLock<alloc::borrow::Cow<'static, str>> = OnceLock::new();
    SPEC.get_or_init(|| {
        raw_source_text_embedded("xml_1_0_fifth_edition", "2008", XML_1_0_FIFTH_EDITION_PRX)
    })
    .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_grammar_loads_with_all_live_productions() {
        // M5.ζ.2 acceptance test — every live <prod> block in the
        // bundled W3C XML 1.0 Fifth Edition spec parses to a typed
        // Term tree via pr4xis::xml_grammar::load_grammar. Per
        // feedback_corpus_wide_audit_on_load, this is the corpus-wide
        // audit that surfaces any unresolved spec production at test
        // time — never silently.
        //
        // Structural assertion (not exact count): every <prod> tag
        // in the bundled spec source must yield a loaded production
        // EXCEPT for any whose `<prod>` open-tag falls inside a
        // §2.5 comment. The spec source carries 86 <prod> open-tags
        // total, with at least one inside an XML comment (a deleted
        // production from an earlier revision). Asserting the exact
        // count `85` against `grammar.len()` would be a brittle
        // bounded-discovery claim per
        // `feedback_no_bounded_discovery_counts`; instead we assert
        // the invariant that survives every spec revision:
        // `grammar.len() == count_uncommented(<prod>)`.
        use pr4xis::xml_grammar::load_grammar;
        let bytes = loaded_xml_1_0_fifth_edition();
        let grammar = load_grammar(bytes).expect("load spec grammar");
        let live_prod_count = count_uncommented_prod_opens(bytes);
        assert_eq!(
            grammar.len(),
            live_prod_count,
            "load_grammar yielded {} productions; the spec source has {} live <prod> tags \
             (after stripping §2.5 comments). The loader must classify every live <prod> \
             — drift indicates a parsing regression.",
            grammar.len(),
            live_prod_count,
        );
        // Sanity floor: a sane XML 1.0 grammar carries dozens of
        // productions, not 3 or 200. The exact published count is
        // an artifact of the source revision, not an ontology
        // invariant.
        assert!(
            grammar.len() >= 50,
            "grammar.len() = {} is implausibly small — load_grammar must be regressed",
            grammar.len()
        );
        // Spot-check the productions M5.ε.2/.3 hand-codegen'd as
        // character-class predicates — they should now also resolve
        // via the full-grammar loader. These are the load-bearing
        // production names downstream code references by string;
        // any one missing is a hard break.
        for name in ["document", "Char", "S", "NameStartChar", "NameChar", "Name"] {
            assert!(
                grammar.lookup(name).is_some(),
                "production {name} must be loaded"
            );
        }
        // The known-deleted production must NOT be present in the
        // loaded grammar — its `<prod>` open-tag sits inside an
        // XML comment and the parser must strip it.
        assert!(
            grammar.lookup("ExternalDef").is_none(),
            "ExternalDef is in an XML comment and must be skipped"
        );
    }

    /// Count `<prod` open-tag occurrences in `bytes` after eliding
    /// every `<!--…-->` comment span. The W3C XML 1.0 §2.5 spec
    /// describes comments as `(Char* - (Char* '--' Char*))` — we
    /// take the simpler "first `-->` after `<!--`" reading which
    /// suffices for the bundled spec source (which doesn't embed
    /// `<!--` inside other comments).
    ///
    /// Returns the number of `<prod` occurrences that are NOT inside
    /// any comment span — the live-production count load_grammar
    /// should report.
    fn count_uncommented_prod_opens(bytes: &str) -> usize {
        let mut total = 0usize;
        let mut cursor = 0usize;
        let raw_bytes = bytes.as_bytes();
        while cursor < raw_bytes.len() {
            // Find the next `<` at cursor.
            let Some(lt_rel) = bytes[cursor..].find('<') else {
                break;
            };
            let lt = cursor + lt_rel;
            let after_lt = &bytes[lt..];
            if after_lt.starts_with("<!--") {
                // Skip the comment span.
                let Some(close_rel) = after_lt.find("-->") else {
                    break;
                };
                cursor = lt + close_rel + 3;
            } else if after_lt.starts_with("<prod")
                && after_lt.as_bytes().get(5).is_some_and(|&b| {
                    b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' || b == b'>'
                })
            {
                // Match `<prod` as an open-tag *only* when followed
                // by whitespace or `>` — the xmlspec.dtd format
                // also defines `<prodgroup>` and `<prodrecap>`
                // which share the `<prod` prefix; they are not
                // production declarations.
                total += 1;
                cursor = lt + 5;
            } else {
                cursor = lt + 1;
            }
        }
        total
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn spec_bytes_match_lock_hash() {
        use pr4xis_runtime::address::ContentAddress;
        let bytes = loaded_xml_1_0_fifth_edition();
        let hex = ContentAddress::of(bytes.as_bytes()).to_hex();
        assert_eq!(
            hex, "af2259d1792179ec1a7a58f45f7fc69c588618e82d567b6bdd67636834b819bc",
            "loaded XML 1.0 Fifth Edition bytes must match the praxis.lock pinned digest"
        );
    }
}
