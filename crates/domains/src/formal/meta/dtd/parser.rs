//! `parse_dtd` — minimal DTD parser for the four declaration kinds
//! W3C XML 1.0 §2.8 + §3.2 + §3.3 + §4.2 + §4.7 (Bray et al. 2008).
//!
//! Scope: recognise `<!ELEMENT>`, `<!ATTLIST>`, `<!ENTITY>`, and
//! `<!NOTATION>` declarations in document order. Parameter-entity
//! expansion (§4.4) and conditional sections (§3.4) are deferred —
//! no entry in the praxis source registry currently uses them.
//!
//! The parser is a stream-style scanner: walk through the bytes,
//! locate `<!KIND ...>` openings, capture the declaration body up to
//! the next top-level `>`, and emit one [`DtdDeclaration`] per match.
//! Comments (`<!-- ... -->`) and processing instructions (`<? ... ?>`)
//! are skipped. The bundled WN-LMF 1.3 DTD parses cleanly under this
//! scope.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::DtdConcept;

/// One parsed declaration: the typed [`DtdConcept`] it instantiates
/// plus the literal name and body the parser captured. The body
/// preserves the source bytes between the kind keyword and the
/// closing `>`, useful for downstream decoders that want to walk
/// content models / attribute lists / entity values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtdDecl {
    /// The typed concept this declaration projects to —
    /// [`DtdConcept::ElementDecl`], [`DtdConcept::AttListDecl`],
    /// [`DtdConcept::EntityDecl`], or [`DtdConcept::NotationDecl`].
    pub kind: DtdConcept,
    /// The declared name (the first identifier inside the declaration
    /// body). For entities, includes the leading `%` if it's a
    /// parameter entity, stripped to just the name otherwise.
    pub name: String,
    /// The declaration body — everything between the kind keyword
    /// and the closing `>`, with the leading name stripped.
    pub body: String,
}

/// Alias kept stable for re-export — the parser's primary output
/// type. Same as [`DtdDecl`] today; the alias is the future-proof
/// surface in case the parser starts emitting richer structured
/// data per declaration.
pub type DtdDeclaration = DtdDecl;

/// Parse a DTD byte stream into the list of declarations it contains.
///
/// Returns the declarations in document order. Unrecognised text
/// (comments, PI, whitespace, parameter-entity references) is
/// silently skipped — the parser does not validate the declarations'
/// semantic legality, only the lexical shape.
#[must_use]
pub fn parse_dtd(bytes: &[u8]) -> Vec<DtdDecl> {
    let Ok(text) = core::str::from_utf8(bytes) else {
        return Vec::new();
    };
    let mut decls = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let rest = &text[cursor..];
        // Skip comments (W3C XML 1.0 §2.5 production [15]).
        if let Some(skip) = consume_prefix(rest, "<!--") {
            cursor += skip;
            if let Some(end) = text[cursor..].find("-->") {
                cursor += end + 3;
            } else {
                cursor = text.len();
            }
            continue;
        }
        // Skip processing instructions (§2.6 [16]).
        if let Some(skip) = consume_prefix(rest, "<?") {
            cursor += skip;
            if let Some(end) = text[cursor..].find("?>") {
                cursor += end + 2;
            } else {
                cursor = text.len();
            }
            continue;
        }
        // Recognise the four declaration kinds.
        if let Some((decl, advance)) = try_parse_decl(rest) {
            decls.push(decl);
            cursor += advance;
            continue;
        }
        // Advance one character (string-safe via char_indices).
        match text[cursor..].chars().next() {
            Some(c) => cursor += c.len_utf8(),
            None => break,
        }
    }
    decls
}

/// Try to parse one of the four declaration kinds at the start of
/// `rest`. Returns `(declaration, bytes-consumed)` on success.
fn try_parse_decl(rest: &str) -> Option<(DtdDecl, usize)> {
    for (prefix, kind) in DECL_PREFIXES {
        if let Some(after_prefix) = rest.strip_prefix(prefix) {
            let close = after_prefix.find('>')?;
            let body_text = &after_prefix[..close];
            let advance = prefix.len() + close + 1;
            let (name_cow, body) = split_name_body(body_text);
            let name: &str = name_cow.as_ref();
            // Refine EntityDecl into parameter / general per the
            // §4.2 `% name` discriminator.
            let refined_kind = match (kind, name.starts_with('%')) {
                (DtdConcept::EntityDecl, true) => DtdConcept::ParameterEntity,
                (DtdConcept::EntityDecl, false) => DtdConcept::GeneralEntity,
                _ => *kind,
            };
            // Strip leading `%` from parameter-entity names so
            // downstream lookups query by bare name.
            let bare_name = name.strip_prefix('%').unwrap_or(name).trim().to_string();
            return Some((
                DtdDecl {
                    kind: refined_kind,
                    name: bare_name,
                    body: body.to_string(),
                },
                advance,
            ));
        }
    }
    None
}

/// The four §2.8 [29] markup-declaration opening prefixes plus the
/// concept each one declares.
const DECL_PREFIXES: &[(&str, DtdConcept)] = &[
    // §3.2 [45] elementdecl
    ("<!ELEMENT ", DtdConcept::ElementDecl),
    // §3.3 [52] AttlistDecl
    ("<!ATTLIST ", DtdConcept::AttListDecl),
    // §4.2 [70] EntityDecl (sub-classified into GE / PE post-match)
    ("<!ENTITY ", DtdConcept::EntityDecl),
    // §4.7 [82] NotationDecl
    ("<!NOTATION ", DtdConcept::NotationDecl),
];

/// Split a declaration body into the first whitespace-delimited
/// identifier and the remaining body text. The identifier captured
/// is what the declaration declares (an element name, an entity
/// name, etc.); the body is the production-specific tail (content
/// model, attribute list, entity value, etc.).
///
/// For `<!ENTITY % name ...>` the leading `% ` is the parameter-
/// entity marker (W3C XML 1.0 §4.2 production [72]): a lone `%`
/// token followed by whitespace and then the actual name. We
/// detect that prefix and stitch it back onto the name as `%name`
/// so callers can discriminate parameter vs. general entities by
/// a simple `starts_with('%')` check.
fn split_name_body(body: &str) -> (alloc::borrow::Cow<'_, str>, &str) {
    let body = body.trim_start();
    let (first, rest) = take_first_token(body);
    if first == "%" {
        // Parameter-entity marker — concatenate `%` with the next
        // token so the returned name reads `%name`.
        let rest_trimmed = rest.trim_start();
        let (real_name, real_rest) = take_first_token(rest_trimmed);
        let mut joined = alloc::string::String::with_capacity(1 + real_name.len());
        joined.push('%');
        joined.push_str(real_name);
        (alloc::borrow::Cow::Owned(joined), real_rest.trim_start())
    } else {
        (alloc::borrow::Cow::Borrowed(first), rest.trim_start())
    }
}

/// Split off the first whitespace-delimited token from `s` and
/// return `(token, remainder)`. If `s` has no whitespace, the
/// whole string is the token and the remainder is empty.
fn take_first_token(s: &str) -> (&str, &str) {
    let mut end = s.len();
    for (i, c) in s.char_indices() {
        if c.is_whitespace() {
            end = i;
            break;
        }
    }
    (&s[..end], &s[end..])
}

/// If `s` starts with `prefix`, return its byte-length. Used to keep
/// the top-level cursor management explicit.
fn consume_prefix(s: &str, prefix: &str) -> Option<usize> {
    if s.starts_with(prefix) {
        Some(prefix.len())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_element_decl() {
        let dtd = b"<!ELEMENT root (#PCDATA)>";
        let decls = parse_dtd(dtd);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, DtdConcept::ElementDecl);
        assert_eq!(decls[0].name, "root");
        assert_eq!(decls[0].body, "(#PCDATA)");
    }

    #[test]
    fn parses_attlist_decl() {
        let dtd = b"<!ATTLIST elem id ID #REQUIRED>";
        let decls = parse_dtd(dtd);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, DtdConcept::AttListDecl);
        assert_eq!(decls[0].name, "elem");
    }

    #[test]
    fn parses_general_entity() {
        let dtd = br#"<!ENTITY copy "(c)">"#;
        let decls = parse_dtd(dtd);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, DtdConcept::GeneralEntity);
        assert_eq!(decls[0].name, "copy");
    }

    #[test]
    fn parses_parameter_entity() {
        let dtd = br#"<!ENTITY % shared "common">"#;
        let decls = parse_dtd(dtd);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, DtdConcept::ParameterEntity);
        assert_eq!(decls[0].name, "shared");
    }

    #[test]
    fn parses_notation() {
        let dtd = br#"<!NOTATION jpeg PUBLIC "image/jpeg" "viewer">"#;
        let decls = parse_dtd(dtd);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, DtdConcept::NotationDecl);
        assert_eq!(decls[0].name, "jpeg");
    }

    #[test]
    fn skips_comments() {
        let dtd = b"<!-- header --><!ELEMENT root EMPTY>";
        let decls = parse_dtd(dtd);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].kind, DtdConcept::ElementDecl);
        assert_eq!(decls[0].name, "root");
    }

    #[test]
    fn parses_wn_lmf_dtd() {
        // Sanity check against the bundled WN-LMF 1.3 DTD: the parser
        // recognises the published LexicalResource / Lexicon /
        // LexicalEntry / Synset / Sense ElementDecls plus their
        // AttListDecl siblings.
        let dtd = crate::social::software::markup::xml::lmf::WN_LMF_1_3_DTD;
        let decls = parse_dtd(dtd.as_bytes());
        let names: Vec<_> = decls
            .iter()
            .filter(|d| d.kind == DtdConcept::ElementDecl)
            .map(|d| d.name.as_str())
            .collect();
        for canonical in [
            "LexicalResource",
            "Lexicon",
            "LexicalEntry",
            "Synset",
            "Sense",
        ] {
            assert!(
                names.contains(&canonical),
                "WN-LMF DTD missing ElementDecl for `{canonical}`; saw: {names:?}"
            );
        }
    }
}
