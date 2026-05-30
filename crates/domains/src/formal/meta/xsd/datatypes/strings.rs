//! Lexical / value / canonical mappings for the string-derived XSD
//! datatypes (W3C XML Schema 1.1 Part 2 §3.3.1, §3.4.1-§3.4.12).
//!
//! These datatypes share a value space of character sequences and
//! differ in two ways:
//!
//! - **whiteSpace** (§4.3.6) — the fixed facet that governs the
//!   lexical→value mapping: `string` *preserves*, `normalizedString`
//!   *replaces* (`#x9`/`#xA`/`#xD` → `#x20`), and `token` and all its
//!   derivatives *collapse* (replace, then trim and fold internal
//!   runs of spaces to one). The value *is* the canonical literal.
//! - **pattern** — `token`'s derivatives further restrict the
//!   collapsed value to an XML production: `NMTOKEN` to `Nmtoken`,
//!   `Name` to `Name`, `NCName` to a colon-free `Name`, and
//!   `language` to a BCP 47 tag. `ID` / `IDREF` / `ENTITY` share
//!   `NCName`'s lexical space (their distinctions are semantic).
//!
//! The list datatypes `NMTOKENS` / `IDREFS` / `ENTITIES` (§3.4.5 /
//! §3.4.10 / §3.4.12) are whitespace-separated, non-empty sequences of
//! their atomic item type.
//!
//! The `Name` / `NCName` / `NMTOKEN` character classes reuse the XML
//! 1.0 §2.3 `NameStartChar` / `NameChar` predicates from the XML
//! parser, since XSD defines these datatypes *by reference* to those
//! productions (§3.4.4-§3.4.7) — composition over re-derivation.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.1 string, §3.4.1-§3.4.12.
//! - **W3C XML 1.0 (Fifth Edition)**, Bray et al. 2008, §2.3
//!   (productions \[4\]/\[4a\]/\[5\] Name / NameChar / Nmtoken).
//! - **IETF BCP 47 / RFC 5646**, Tags for Identifying Languages.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::social::software::markup::xml::parser::grammar::{is_name_char, is_name_start_char};

// =============================================================================
// whiteSpace facet — W3C XSD 1.1 Part 2 §4.3.6.
// =============================================================================

/// The `whiteSpace` facet value (§4.3.6): the lexical→value
/// normalization a string datatype applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteSpace {
    /// No normalization — the value is the literal unchanged (`string`).
    Preserve,
    /// `#x9` (tab), `#xA` (LF), `#xD` (CR) each become `#x20`
    /// (`normalizedString`).
    Replace,
    /// Replace, then strip leading/trailing spaces and fold internal
    /// runs of spaces to a single space (`token` and its derivatives).
    Collapse,
}

/// Apply a `whiteSpace` facet to a literal (§4.3.6). Only the four XSD
/// whitespace characters are affected; other Unicode spaces are left
/// intact (they are not XSD whitespace).
pub fn apply_white_space(s: &str, ws: WhiteSpace) -> String {
    match ws {
        WhiteSpace::Preserve => s.to_string(),
        WhiteSpace::Replace => s
            .chars()
            .map(|c| {
                if matches!(c, '\t' | '\n' | '\r') {
                    ' '
                } else {
                    c
                }
            })
            .collect(),
        WhiteSpace::Collapse => {
            let replaced: String = s
                .chars()
                .map(|c| {
                    if matches!(c, '\t' | '\n' | '\r') {
                        ' '
                    } else {
                        c
                    }
                })
                .collect();
            replaced
                .split(' ')
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

// =============================================================================
// Pattern predicates — XML 1.0 §2.3 productions + BCP 47.
// =============================================================================

/// XML 1.0 §2.3 \[5\] `Nmtoken ::= (NameChar)+` — `xs:NMTOKEN`'s value
/// space (W3C XSD 1.1 Part 2 §3.4.4).
pub fn is_nmtoken(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_name_char)
}

/// XML 1.0 §2.3 \[4\] `Name ::= NameStartChar (NameChar)*` — `xs:Name`'s
/// value space (W3C XSD 1.1 Part 2 §3.4.6).
pub fn is_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => is_name_start_char(first) && chars.all(is_name_char),
        None => false,
    }
}

/// XML Namespaces §3 `NCName ::= Name - (Char* ':' Char*)` — a `Name`
/// with no colon (W3C XSD 1.1 Part 2 §3.4.7).
pub fn is_ncname(s: &str) -> bool {
    is_name(s) && !s.contains(':')
}

/// BCP 47 / RFC 5646 language tag, restricted to the XSD `language`
/// pattern `[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*` (W3C XSD 1.1 Part 2
/// §3.4.3).
pub fn is_language(s: &str) -> bool {
    let mut parts = s.split('-');
    let primary = match parts.next() {
        Some(p) => p,
        None => return false,
    };
    if primary.is_empty() || primary.len() > 8 || !primary.bytes().all(|b| b.is_ascii_alphabetic())
    {
        return false;
    }
    parts.all(|p| !p.is_empty() && p.len() <= 8 && p.bytes().all(|b| b.is_ascii_alphanumeric()))
}

// =============================================================================
// Atomic string datatypes — parse (lexical→value) + canonical (identity).
// =============================================================================

/// `xs:string` (§3.3.1, whiteSpace=preserve): every literal is a valid
/// value, taken unchanged.
pub fn parse_string(lex: &str) -> String {
    apply_white_space(lex, WhiteSpace::Preserve)
}

/// `xs:normalizedString` (§3.4.1, whiteSpace=replace).
pub fn parse_normalized_string(lex: &str) -> String {
    apply_white_space(lex, WhiteSpace::Replace)
}

/// `xs:token` (§3.4.2, whiteSpace=collapse).
pub fn parse_token(lex: &str) -> String {
    apply_white_space(lex, WhiteSpace::Collapse)
}

/// `xs:language` (§3.4.3): a collapsed token matching the language
/// pattern.
pub fn parse_language(lex: &str) -> Option<String> {
    let v = parse_token(lex);
    is_language(&v).then_some(v)
}

/// `xs:NMTOKEN` (§3.4.4): a collapsed token matching `Nmtoken`.
pub fn parse_nmtoken(lex: &str) -> Option<String> {
    let v = parse_token(lex);
    is_nmtoken(&v).then_some(v)
}

/// `xs:Name` (§3.4.6): a collapsed token matching `Name`.
pub fn parse_name(lex: &str) -> Option<String> {
    let v = parse_token(lex);
    is_name(&v).then_some(v)
}

/// `xs:NCName` (§3.4.7), and its semantic restrictions `xs:ID`
/// (§3.4.8), `xs:IDREF` (§3.4.9), `xs:ENTITY` (§3.4.11), which share
/// the `NCName` lexical space.
pub fn parse_ncname(lex: &str) -> Option<String> {
    let v = parse_token(lex);
    is_ncname(&v).then_some(v)
}

/// The canonical literal of a string-family value is the value itself
/// (the lexical→value mapping already produced a canonical literal).
pub fn canonical_string_value(value: &str) -> String {
    value.to_string()
}

// =============================================================================
// List datatypes — §3.4.5 NMTOKENS / §3.4.10 IDREFS / §3.4.12 ENTITIES.
// =============================================================================

/// Parse a list datatype: collapse whitespace, split into items, and
/// validate each against `item_ok`. The list must be non-empty
/// (W3C XSD 1.1 Part 2 §3.4.5: `minLength` is 1). Returns the item
/// values.
pub fn parse_list<F: Fn(&str) -> bool>(lex: &str, item_ok: F) -> Option<Vec<String>> {
    let collapsed = parse_token(lex);
    if collapsed.is_empty() {
        return None;
    }
    let items: Vec<String> = collapsed.split(' ').map(str::to_string).collect();
    items.iter().all(|i| item_ok(i)).then_some(items)
}

/// `xs:NMTOKENS` (§3.4.5): a non-empty whitespace-separated list of
/// `NMTOKEN`.
pub fn parse_nmtokens(lex: &str) -> Option<Vec<String>> {
    parse_list(lex, is_nmtoken)
}

/// `xs:IDREFS` (§3.4.10) / `xs:ENTITIES` (§3.4.12): a non-empty
/// whitespace-separated list of `NCName` (their item types `IDREF` /
/// `ENTITY` share `NCName`'s lexical space).
pub fn parse_ncname_list(lex: &str) -> Option<Vec<String>> {
    parse_list(lex, is_ncname)
}

/// The canonical literal of a list value: the items joined by a single
/// space (§4.3.6 collapse leaves exactly single-space separators).
pub fn canonical_list(items: &[String]) -> String {
    items.join(" ")
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: the `whiteSpace` facet (§4.3.6) is applied per datatype —
/// `string` preserves tabs/newlines, `normalizedString` replaces them
/// with spaces, and `token` additionally trims and folds runs.
pub struct WhiteSpaceFacetApplied;

impl Axiom for WhiteSpaceFacetApplied {
    fn verify(&self) -> Verdict {
        let ok = parse_string("a\tb\nc") == "a\tb\nc"
            && parse_normalized_string("a\tb\nc") == "a b c"
            && parse_token("  a \t b  ") == "a b"
            && parse_token("a\n\nb") == "a b"
            // Non-XSD whitespace (no-break space U+00A0) is preserved.
            && parse_token("a\u{00A0}b") == "a\u{00A0}b";
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "WhiteSpaceFacetApplied",
        "the whiteSpace facet is applied per datatype: string preserves, normalizedString replaces tab/LF/CR with space, token trims and folds runs; non-XSD whitespace is preserved",
        "W3C XSD 1.1 Part 2 §4.3.6, §3.3.1, §3.4.1, §3.4.2 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    WhiteSpaceFacetApplied,
    "W3C XSD 1.1 Part 2 §4.3.6, §3.3.1, §3.4.1, §3.4.2"
);

/// Axiom: the value of a string-family datatype is its own canonical
/// literal — re-parsing the canonical form is a fixpoint (§3.4.2: the
/// collapsed/replaced value is already in the lexical space).
pub struct StringValueIsCanonicalFixpoint;

impl Axiom for StringValueIsCanonicalFixpoint {
    fn verify(&self) -> Verdict {
        let samples = ["  Hello   World  ", "a\tb", "single"];
        for s in samples {
            let v = parse_token(s);
            // Canonical of a value is the value; re-parsing is stable.
            if canonical_string_value(&v) != v || parse_token(&v) != v {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // NCName list canonical round-trips.
        let Some(items) = parse_ncname_list("  a   b  c ") else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let canon = canonical_list(&items);
        if canon != "a b c" || parse_ncname_list(&canon).as_deref() != Some(items.as_slice()) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "StringValueIsCanonicalFixpoint",
        "a string-family value is its own canonical literal: canonicalization and re-parse are fixpoints, and list canonical forms use single-space separators",
        "W3C XSD 1.1 Part 2 §3.4.2, §3.4.5, §4.3.6 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    StringValueIsCanonicalFixpoint,
    "W3C XSD 1.1 Part 2 §3.4.2, §3.4.5, §4.3.6"
);

/// Axiom: the `Name` / `NCName` / `NMTOKEN` / `language` value spaces
/// conform to their productions — `NMTOKEN` admits a leading digit,
/// `Name` does not, `NCName` forbids a colon, and `language` matches
/// the BCP 47 pattern (W3C XSD 1.1 Part 2 §3.4.3-§3.4.7).
pub struct NameProductionConformance;

impl Axiom for NameProductionConformance {
    fn verify(&self) -> Verdict {
        let ok = parse_nmtoken("123abc").is_some()      // Nmtoken: digit start OK
            && parse_name("123abc").is_none()           // Name: no digit start
            && parse_name("a:b").is_some()              // Name allows colon
            && parse_ncname("a:b").is_none()            // NCName forbids colon
            && parse_ncname("_foo.bar-baz").is_some()   // NCName chars
            && parse_nmtoken("a b").is_none()           // space not a NameChar
            && parse_language("en").is_some()
            && parse_language("en-US").is_some()
            && parse_language("de-Latn-DE").is_some()
            && parse_language("toolongprimary").is_none() // > 8 letters
            && parse_language("en_US").is_none(); // underscore not allowed
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NameProductionConformance",
        "the Name/NCName/NMTOKEN/language value spaces conform to their productions: NMTOKEN admits a leading digit, Name does not, NCName forbids a colon, language matches the BCP 47 pattern",
        "W3C XSD 1.1 Part 2 §3.4.3-§3.4.7; W3C XML 1.0 §2.3 (Peterson et al. 2012; Bray et al. 2008)"
    );
}

pr4xis::register_axiom!(
    NameProductionConformance,
    "W3C XSD 1.1 Part 2 §3.4.3-§3.4.7; W3C XML 1.0 §2.3"
);

/// Axiom: list datatypes split on whitespace into a non-empty sequence
/// of valid items; an all-whitespace literal (no items) is rejected
/// (`minLength` 1, §3.4.5).
pub struct ListDatatypesSplitOnWhitespace;

impl Axiom for ListDatatypesSplitOnWhitespace {
    fn verify(&self) -> Verdict {
        let ok = parse_nmtokens("a b c").map(|v| v.len()) == Some(3)
            && parse_nmtokens("  one\ttwo\n three ").map(|v| v.len()) == Some(3)
            && parse_nmtokens("   ").is_none()          // no items
            && parse_nmtokens("").is_none()
            && parse_ncname_list("a:b c").is_none()     // invalid item (colon)
            && parse_ncname_list("ref1 ref2").map(|v| v.len()) == Some(2);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ListDatatypesSplitOnWhitespace",
        "list datatypes split on whitespace into a non-empty sequence of valid items; an all-whitespace literal is rejected",
        "W3C XSD 1.1 Part 2 §3.4.5, §3.4.10, §3.4.12 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    ListDatatypesSplitOnWhitespace,
    "W3C XSD 1.1 Part 2 §3.4.5, §3.4.10, §3.4.12"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn whitespace_modes() {
        assert_eq!(parse_string("a\tb"), "a\tb");
        assert_eq!(parse_normalized_string("a\tb\nc"), "a b c");
        assert_eq!(parse_token("  a   b  "), "a b");
    }

    #[test]
    fn name_productions() {
        assert!(parse_nmtoken("123").is_some());
        assert!(parse_name("123").is_none());
        assert!(parse_name("x:y").is_some());
        assert!(parse_ncname("x:y").is_none());
        assert!(parse_ncname("validName_1").is_some());
    }

    #[test]
    fn language_tags() {
        assert!(parse_language("en").is_some());
        assert!(parse_language("en-US").is_some());
        assert!(parse_language("zh-Hant-TW").is_some());
        assert!(parse_language("123").is_none());
        assert!(parse_language("verylongtag").is_none());
    }

    #[test]
    fn id_idref_entity_share_ncname() {
        // ID/IDREF/ENTITY all use parse_ncname.
        assert_eq!(parse_ncname("anchor1"), Some("anchor1".to_string()));
        assert!(parse_ncname("1bad").is_none());
    }

    #[test]
    fn lists() {
        assert_eq!(parse_nmtokens("a b c").unwrap().len(), 3);
        assert_eq!(canonical_list(&parse_nmtokens("  a  b ").unwrap()), "a b");
        assert!(parse_nmtokens("   ").is_none());
        assert!(parse_ncname_list("a b:c").is_none());
    }

    #[test]
    fn axiom_whitespace() {
        assert!(WhiteSpaceFacetApplied.verify().is_ok());
    }

    #[test]
    fn axiom_value_fixpoint() {
        assert!(StringValueIsCanonicalFixpoint.verify().is_ok());
    }

    #[test]
    fn axiom_name_conformance() {
        assert!(NameProductionConformance.verify().is_ok());
    }

    #[test]
    fn axiom_list_split() {
        assert!(ListDatatypesSplitOnWhitespace.verify().is_ok());
    }

    proptest! {
        /// Collapsing is idempotent: a collapsed token re-collapses to
        /// itself (Peterson et al. 2012 §4.3.6).
        #[test]
        fn prop_collapse_idempotent(s in ".*") {
            let once = parse_token(&s);
            let twice = parse_token(&once);
            prop_assert_eq!(once, twice);
        }

        /// Replace then collapse equals collapse (collapse subsumes
        /// replace).
        #[test]
        fn prop_collapse_subsumes_replace(s in ".*") {
            let via_replace = parse_token(&parse_normalized_string(&s));
            let direct = parse_token(&s);
            prop_assert_eq!(via_replace, direct);
        }

        /// Any all-NameChar non-empty string is a valid NMTOKEN and is
        /// its own value.
        #[test]
        fn prop_nmtoken_roundtrip(s in "[A-Za-z0-9_.-]{1,20}") {
            let v = parse_nmtoken(&s).expect("NameChar string is an NMTOKEN");
            prop_assert_eq!(v, s);
        }
    }
}
