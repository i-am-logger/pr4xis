//! W3C XML 1.0 Fifth Edition spec — the source of grammar predicates.
//!
//! Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008) is the
//! normative grammar specification the praxis XML 1.0 parser
//! implements. The Fifth Edition's XML-format source (xmlspec.dtd)
//! ships with 86 `<prod>` blocks — one per EBNF production — that
//! the parser-predicates module derives from at build time. Per
//! `feedback_bottom_up_loaded_not_encoded`: every predicate the
//! parser reasons against comes from this loaded source, not from
//! hand-coded code-point ranges.
//!
//! ## Citation
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (eds.) (2008) *Extensible Markup Language (XML)
//!   1.0 (Fifth Edition)*, W3C Recommendation 26 November 2008,
//!   <https://www.w3.org/TR/2008/REC-xml-20081126/>; the XML-format
//!   source at
//!   <https://www.w3.org/TR/2008/REC-xml-20081126/REC-xml-20081126.xml>
//!   is what these bytes are byte-for-byte copies of.

pub use content_dispatch::{
    ContentDispatchTable, ContentItemKind, DispatchExtractionError, MiscDispatchTable,
    MiscItemKind, loaded_content_dispatch_table, loaded_misc_dispatch_table,
};
pub use encoding_labels::{
    EncodingLabelExtractionError, XmlEncodingFamilies, loaded_xml_encoding_families,
};
pub use spec::loaded_xml_1_0_fifth_edition;

mod content_dispatch;
mod encoding_labels;
mod spec;

/// The W3C XML 1.0 EBNF grammar — all 85 live productions parsed
/// from the bundled spec bytes into a typed
/// [`pr4xis::xml_grammar::Grammar`].
///
/// Cached via `OnceLock`: the first call parses the spec; every
/// subsequent call returns the same `&'static Grammar`. The parser
/// implementation in `parser::grammar` uses this to validate
/// DOCTYPE markup declarations (§3.2 `elementdecl`, §3.3
/// `AttlistDecl`, §4.7 `NotationDecl`) against the spec grammar
/// instead of skip-to-next-`>` heuristics, closing the well-
/// formedness gaps M5.ε.4 exposed.
///
/// Panics on a malformed grammar — a load failure here is a
/// regression in `xml_1_0_fifth_edition@2008` source bytes or in
/// the EBNF parser, both of which are covered by tests.
#[must_use]
pub fn loaded_xml_1_0_grammar() -> &'static pr4xis::xml_grammar::Grammar {
    static GRAMMAR: std::sync::OnceLock<pr4xis::xml_grammar::Grammar> = std::sync::OnceLock::new();
    GRAMMAR.get_or_init(|| {
        pr4xis::xml_grammar::load_grammar(loaded_xml_1_0_fifth_edition())
            .expect("W3C XML 1.0 Fifth Edition spec bytes must parse to a valid Grammar")
    })
}

#[cfg(test)]
mod loaded_grammar_tests {
    use super::loaded_xml_1_0_grammar;
    use pr4xis::xml_grammar::{Interpreter, MatchResult};

    fn matches_all(text: &str, prod: &str) -> bool {
        let mut interp = Interpreter::new(loaded_xml_1_0_grammar(), text);
        matches!(
            interp.match_production(prod, 0),
            MatchResult::Match { end_pos } if end_pos == text.len()
        )
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn att_value_negated_class_excludes_only_three_chars() {
        // Regression for the rhs_parser bug where the negated
        // character class `[^&lt;&amp;"]` in §3.1 [10] AttValue
        // (and the symmetric `[^&lt;&amp;']`) was tokenised as 10
        // excluded chars (`&`, `l`, `t`, `;`, `&`, `a`, `m`, `p`,
        // `;`, `"`) instead of three (`<`, `&`, `"`). With the
        // pre-fix tokenisation, any attribute value containing
        // `l`, `t`, `a`, `m`, or `p` parsed as NoMatch — for
        // example `"default"` or `'JavaBeans'`. With the fix in
        // `rhs_parser::read_char_class`, both match.
        assert!(matches_all("\"default\"", "AttValue"));
        assert!(matches_all("'JavaBeans'", "AttValue"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn default_decl_required_matches() {
        assert!(matches_all("#REQUIRED", "DefaultDecl"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn default_decl_implied_matches() {
        assert!(matches_all("#IMPLIED", "DefaultDecl"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn default_decl_fixed_value_matches() {
        // §3.3 [60] DefaultDecl ::= ... | (('#FIXED' S)? AttValue)
        // — the `#FIXED " "` prefix + AttValue.
        assert!(matches_all("#FIXED \"JavaBeans\"", "DefaultDecl"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn default_decl_bare_attvalue_double_quoted() {
        // §3.3 [60] DefaultDecl — bare AttValue (optional #FIXED omitted).
        assert!(matches_all("\"default\"", "DefaultDecl"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn default_decl_bare_attvalue_single_quoted() {
        assert!(matches_all("'default'", "DefaultDecl"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn att_value_with_double_quotes() {
        assert!(matches_all("\"foo\"", "AttValue"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn att_value_with_single_quotes() {
        assert!(matches_all("'foo'", "AttValue"));
    }
}

/// Generated grammar predicates — emitted at build time by
/// `pr4xis::codegen::xml_grammar` from the loaded spec bytes. Provides:
///
/// - `is_char(c: u32) -> bool` — W3C XML 1.0 §2.2 \[2\] `Char`
/// - `is_name_start_char(c: u32) -> bool` — §2.3 \[4\] `NameStartChar`
/// - `is_name_char(c: u32) -> bool` — §2.3 \[4a\] `NameChar`
///
/// Plus the underlying `CHAR_RANGES` / `NAME_START_CHAR_RANGES` /
/// `NAME_CHAR_RANGES` tables. Consumed by `parser::grammar` in
/// place of hand-coded `matches!` arms over hardcoded code points.
///
/// When the spec source is missing (e.g. a published-crate consumer
/// without the bundled bytes), the include resolves to a stub where
/// every predicate returns `false`. That makes the absence visible
/// (parser rejects every name) rather than silently using a fallback.
pub mod grammar {
    include!(concat!(env!("OUT_DIR"), "/xml_grammar_generated.rs"));
}
