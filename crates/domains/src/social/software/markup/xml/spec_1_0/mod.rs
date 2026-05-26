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

pub use spec::{XML_1_0_FIFTH_EDITION, loaded_xml_1_0_fifth_edition};

mod spec;

/// Generated grammar predicates — emitted at build time by
/// `pr4xis::codegen::xml_grammar` from the loaded spec bytes. Provides:
///
/// - `is_char(c: u32) -> bool` — W3C XML 1.0 §2.2 [2] `Char`
/// - `is_name_start_char(c: u32) -> bool` — §2.3 [4] `NameStartChar`
/// - `is_name_char(c: u32) -> bool` — §2.3 [4a] `NameChar`
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
