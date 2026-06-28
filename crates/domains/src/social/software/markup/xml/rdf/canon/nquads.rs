//! N-Quads parsing and **canonical** N-Quads serialization for RDFC-1.0.
//!
//! Two responsibilities, both ground-truthed against the W3C
//! *RDF Dataset Canonicalization* Recommendation (REC-rdf-canon-20240521):
//!
//! 1. A pull parser for the [N-Quads] grammar that reads the suite's
//!    `*-in.nq` fixtures into praxis [`Quad`]s, decoding the `ECHAR`
//!    (`\t \b \n \r \f \" \' \\`) and `UCHAR` (`\uXXXX` / `\UXXXXXXXX`)
//!    escapes that appear in both `IRIREF` and `STRING_LITERAL_QUOTE`
//!    productions (REC §A grammar pointers into [N-Quads]).
//! 2. A serializer for the **canonical** N-Quads form defined in
//!    REC §"A Canonical form of N-Quads" — a single space after subject,
//!    predicate, object and graph label; a single `LF` `EOL` after every
//!    statement including the last; `xsd:string`-typed literals drop the
//!    datatype IRI; the `ECHAR`/`UCHAR` re-escaping table is applied
//!    exactly as the appendix prescribes; `HEX` in any emitted `UCHAR`
//!    is *upper*-case while the `\u` / `\U` prefix is lower-case.
//!
//! The reused term model is praxis [`RdfTerm`] / [`Triple`]
//! (`super::super::term`) — RDFC adds only the [`Quad`] (a triple with an
//! optional graph-name component, RDF 1.1 dataset model) and never forks
//! the term enum.
//!
//! `no_std` + `alloc`; no I/O, no `std`-only API — wasm32-clean.
//!
//! [N-Quads]: https://www.w3.org/TR/n-quads/

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::super::term::{RdfTerm, Triple};
use super::CanonError;

/// The `xsd:string` datatype IRI. A literal carrying exactly this datatype
/// is, per RDF 1.1 §3.3, a *simple literal*; the canonical N-Quads form
/// (REC §"A Canonical form of N-Quads") MUST omit the datatype part for
/// such literals.
pub(crate) const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// One RDF quad: an RDF 1.1 [`Triple`] plus an optional *graph name*
/// component placing the triple in a named graph of a dataset
/// (RDF 1.1 Concepts §4 — RDF datasets are a default graph plus zero or
/// more named graphs). `graph == None` is the default graph.
///
/// A graph name is an IRI or a blank node (never a literal), exactly the
/// admissibility rule for subjects; RDFC-1.0 treats a blank node in graph
/// position as a fourth blank-node component of the quad (REC §4.4.3,
/// step 2 — "for each blank node that is a component of Q").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quad {
    /// The underlying triple (subject, predicate, object).
    pub triple: Triple,
    /// The graph name, or `None` for the default graph.
    pub graph: Option<RdfTerm>,
}

impl Quad {
    /// A quad in the default graph.
    pub fn new(subject: RdfTerm, predicate: String, object: RdfTerm) -> Self {
        Self {
            triple: Triple {
                subject,
                predicate,
                object,
            },
            graph: None,
        }
    }

    /// A quad in the named graph `graph`.
    pub fn in_graph(triple: Triple, graph: Option<RdfTerm>) -> Self {
        Self { triple, graph }
    }

    /// Lift a [`Triple`] into a default-graph quad (`graph = None`).
    ///
    /// The canonical embedding of the RDF 1.1 graph model into the
    /// dataset model (RDF 1.1 Concepts §4): every triple of a single
    /// graph — e.g. the one an RDF/XML document denotes — belongs to the
    /// dataset's *default graph*. Used by the OWL lens to present the
    /// source triple stream to RDFC-1.0 as a default-graph dataset.
    pub fn from_default_graph(triple: Triple) -> Self {
        Self {
            triple,
            graph: None,
        }
    }

    /// Convenience accessors mirroring N-Quads positions.
    pub fn subject(&self) -> &RdfTerm {
        &self.triple.subject
    }
    pub fn predicate(&self) -> &str {
        &self.triple.predicate
    }
    pub fn object(&self) -> &RdfTerm {
        &self.triple.object
    }
    pub fn graph(&self) -> Option<&RdfTerm> {
        self.graph.as_ref()
    }
}

// ===========================================================================
// Parser
// ===========================================================================

/// Parse an N-Quads document into a `Vec<Quad>`.
///
/// Blank syntactic lines and `# …` comment lines are skipped (N-Quads
/// grammar). Each statement is `subject predicate object [graphLabel] .`.
/// Blank-node identifiers retain their *input* label (the `_:name` after
/// the `_:`), which RDFC §4.4.3 step 1 calls the *input blank node
/// identifier map*.
pub fn parse_nquads(input: &str) -> Result<Vec<Quad>, CanonError> {
    let mut quads = Vec::new();
    for raw_line in input.split('\n') {
        // Strip an optional trailing CR (tolerate CRLF inputs) and
        // surrounding ASCII whitespace.
        let line = raw_line.trim_matches(|c| c == ' ' || c == '\t' || c == '\r');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let quad = parse_statement(line)?;
        quads.push(quad);
    }
    Ok(quads)
}

/// Parse a single N-Quads statement line (without its trailing newline).
fn parse_statement(line: &str) -> Result<Quad, CanonError> {
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0usize;

    let subject = parse_subject(&chars, &mut pos)?;
    skip_ws(&chars, &mut pos);
    let predicate = parse_iri(&chars, &mut pos)?;
    skip_ws(&chars, &mut pos);
    let object = parse_object(&chars, &mut pos)?;
    skip_ws(&chars, &mut pos);

    // Optional graph label, then the terminating '.'.
    let graph = if pos < chars.len() && chars[pos] != '.' {
        let g = parse_graph_label(&chars, &mut pos)?;
        skip_ws(&chars, &mut pos);
        Some(g)
    } else {
        None
    };

    if pos >= chars.len() || chars[pos] != '.' {
        return Err(CanonError::Parse(format!(
            "expected '.' terminating statement: {line}"
        )));
    }
    pos += 1;
    skip_ws(&chars, &mut pos);
    if pos != chars.len() {
        return Err(CanonError::Parse(format!(
            "trailing content after statement: {line}"
        )));
    }

    Ok(Quad::in_graph(
        Triple {
            subject,
            predicate,
            object,
        },
        graph,
    ))
}

fn skip_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && (chars[*pos] == ' ' || chars[*pos] == '\t') {
        *pos += 1;
    }
}

/// `subject ::= IRIREF | BLANK_NODE_LABEL`.
fn parse_subject(chars: &[char], pos: &mut usize) -> Result<RdfTerm, CanonError> {
    match chars.get(*pos) {
        Some('<') => Ok(RdfTerm::Iri(parse_iri(chars, pos)?)),
        Some('_') => Ok(RdfTerm::Blank(parse_blank(chars, pos)?)),
        _ => Err(CanonError::Parse(
            "expected IRI or blank node in subject position".to_string(),
        )),
    }
}

/// `graphLabel ::= IRIREF | BLANK_NODE_LABEL`.
fn parse_graph_label(chars: &[char], pos: &mut usize) -> Result<RdfTerm, CanonError> {
    parse_subject(chars, pos)
}

/// `object ::= IRIREF | BLANK_NODE_LABEL | literal`.
fn parse_object(chars: &[char], pos: &mut usize) -> Result<RdfTerm, CanonError> {
    match chars.get(*pos) {
        Some('<') => Ok(RdfTerm::Iri(parse_iri(chars, pos)?)),
        Some('_') => Ok(RdfTerm::Blank(parse_blank(chars, pos)?)),
        Some('"') => parse_literal(chars, pos),
        _ => Err(CanonError::Parse(
            "expected IRI, blank node, or literal in object position".to_string(),
        )),
    }
}

/// `IRIREF ::= '<' (... | UCHAR)* '>'`. Returns the *decoded* IRI string
/// (UCHAR escapes resolved to their code points). The grammar excludes raw
/// control characters and the delimiter set `<>"{}|^`\` and space inside an
/// unescaped IRIREF; we accept what the suite emits and resolve escapes.
fn parse_iri(chars: &[char], pos: &mut usize) -> Result<String, CanonError> {
    if chars.get(*pos) != Some(&'<') {
        return Err(CanonError::Parse("expected '<' starting IRI".to_string()));
    }
    *pos += 1;
    let mut out = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        match c {
            '>' => {
                *pos += 1;
                return Ok(out);
            }
            '\\' => {
                *pos += 1;
                // Only UCHAR is valid inside an IRIREF (no ECHAR).
                out.push(parse_uchar(chars, pos)?);
            }
            _ => {
                out.push(c);
                *pos += 1;
            }
        }
    }
    Err(CanonError::Parse("unterminated IRI".to_string()))
}

/// `BLANK_NODE_LABEL ::= '_:' (...)`. Returns the label *without* the `_:`
/// prefix (the praxis [`RdfTerm::Blank`] carries the bare label; the `_:`
/// is a serialization artifact per REC §2 terminology).
fn parse_blank(chars: &[char], pos: &mut usize) -> Result<String, CanonError> {
    if chars.get(*pos) != Some(&'_') || chars.get(*pos + 1) != Some(&':') {
        return Err(CanonError::Parse(
            "expected '_:' starting blank node".to_string(),
        ));
    }
    *pos += 2;
    let mut out = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        // Label terminates at whitespace or the statement '.'.
        if c == ' ' || c == '\t' {
            break;
        }
        // A '.' inside a label is legal (PN_CHARS), but a trailing '.'
        // that terminates the statement is not. Disambiguate: a '.' is
        // part of the label only if a non-terminating label char follows.
        if c == '.' {
            let next = chars.get(*pos + 1).copied();
            match next {
                Some(n) if n != ' ' && n != '\t' && n != '.' => {
                    out.push(c);
                    *pos += 1;
                    continue;
                }
                _ => break,
            }
        }
        out.push(c);
        *pos += 1;
    }
    if out.is_empty() {
        return Err(CanonError::Parse("empty blank node label".to_string()));
    }
    Ok(out)
}

/// `literal ::= STRING_LITERAL_QUOTE ('^^' IRIREF | LANGTAG)?`.
fn parse_literal(chars: &[char], pos: &mut usize) -> Result<RdfTerm, CanonError> {
    let lexical = parse_string_literal_quote(chars, pos)?;
    match chars.get(*pos) {
        Some('^') => {
            // '^^' IRIREF
            if chars.get(*pos + 1) != Some(&'^') {
                return Err(CanonError::Parse("expected '^^' for datatype".to_string()));
            }
            *pos += 2;
            let datatype = parse_iri(chars, pos)?;
            Ok(RdfTerm::Literal {
                lexical,
                lang: None,
                datatype: Some(datatype),
            })
        }
        Some('@') => {
            // LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
            *pos += 1;
            let mut tag = String::new();
            while *pos < chars.len() {
                let c = chars[*pos];
                if c.is_ascii_alphanumeric() || c == '-' {
                    tag.push(c);
                    *pos += 1;
                } else {
                    break;
                }
            }
            if tag.is_empty() {
                return Err(CanonError::Parse("empty language tag".to_string()));
            }
            Ok(RdfTerm::Literal {
                lexical,
                lang: Some(tag),
                datatype: None,
            })
        }
        // No datatype, no language tag: a simple literal (xsd:string).
        _ => Ok(RdfTerm::Literal {
            lexical,
            lang: None,
            datatype: None,
        }),
    }
}

/// `STRING_LITERAL_QUOTE ::= '"' ([^"\\\n\r] | ECHAR | UCHAR)* '"'`.
/// Returns the decoded lexical form.
fn parse_string_literal_quote(chars: &[char], pos: &mut usize) -> Result<String, CanonError> {
    if chars.get(*pos) != Some(&'"') {
        return Err(CanonError::Parse(
            "expected '\"' starting literal".to_string(),
        ));
    }
    *pos += 1;
    let mut out = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        match c {
            '"' => {
                *pos += 1;
                return Ok(out);
            }
            '\\' => {
                *pos += 1;
                out.push(parse_echar_or_uchar(chars, pos)?);
            }
            _ => {
                out.push(c);
                *pos += 1;
            }
        }
    }
    Err(CanonError::Parse("unterminated literal".to_string()))
}

/// Decode a backslash escape inside a string literal: an `ECHAR`
/// (`t b n r f " ' \`) or a `UCHAR` (`u`/`U`). `*pos` points just past the
/// backslash.
fn parse_echar_or_uchar(chars: &[char], pos: &mut usize) -> Result<char, CanonError> {
    let c = *chars
        .get(*pos)
        .ok_or_else(|| CanonError::Parse("dangling backslash escape".to_string()))?;
    let decoded = match c {
        't' => '\u{0009}',
        'b' => '\u{0008}',
        'n' => '\u{000A}',
        'r' => '\u{000D}',
        'f' => '\u{000C}',
        '"' => '"',
        '\'' => '\'',
        '\\' => '\\',
        'u' | 'U' => return parse_uchar(chars, pos),
        other => {
            return Err(CanonError::Parse(format!(
                "invalid string escape: \\{other}"
            )));
        }
    };
    *pos += 1;
    Ok(decoded)
}

/// Decode a `UCHAR`: `'u' HEX{4}` or `'U' HEX{8}`. `*pos` points at the
/// `u`/`U` marker. Advances past the escape.
fn parse_uchar(chars: &[char], pos: &mut usize) -> Result<char, CanonError> {
    let marker = *chars
        .get(*pos)
        .ok_or_else(|| CanonError::Parse("dangling unicode escape".to_string()))?;
    let width = match marker {
        'u' => 4,
        'U' => 8,
        other => {
            return Err(CanonError::Parse(format!(
                "invalid unicode escape marker: \\{other}"
            )));
        }
    };
    *pos += 1;
    let mut value: u32 = 0;
    for _ in 0..width {
        let h = *chars
            .get(*pos)
            .ok_or_else(|| CanonError::Parse("truncated unicode escape".to_string()))?;
        let digit = h
            .to_digit(16)
            .ok_or_else(|| CanonError::Parse(format!("non-hex digit in unicode escape: {h}")))?;
        value = value * 16 + digit;
        *pos += 1;
    }
    char::from_u32(value).ok_or_else(|| {
        CanonError::Parse(format!("unicode escape is not a scalar value: {value:X}"))
    })
}

// ===========================================================================
// Canonical N-Quads serializer
// ===========================================================================

/// Serialize one term in canonical N-Quads form, in subject/object/graph
/// position (predicate is always an IRI and goes through [`serialize_iri`]).
pub(crate) fn serialize_term(term: &RdfTerm) -> String {
    match term {
        RdfTerm::Iri(iri) => serialize_iri(iri),
        RdfTerm::Blank(label) => format!("_:{label}"),
        RdfTerm::Literal {
            lexical,
            lang,
            datatype,
        } => serialize_literal(lexical, lang.as_deref(), datatype.as_deref()),
    }
}

/// Serialize an `IRIREF` in canonical form.
///
/// The canonical N-Quads appendix says each code point is represented by
/// exactly one of `UCHAR`, `ECHAR`, or the unencoded character, "where the
/// relevant production allows for a choice". The N-Quads `IRIREF`
/// production forbids exactly the delimiter set `< > " { } | ^ \` ` ` and
/// space and the C0/C1 control range; those (and only those) MUST be
/// emitted as `UCHAR`. Every other code point is emitted natively — which
/// is why the suite's `<urn:ex: >` round-trips to the native NBSP and
/// `\U0001F303` to the native `🌃`.
pub(crate) fn serialize_iri(iri: &str) -> String {
    let mut out = String::with_capacity(iri.len() + 2);
    out.push('<');
    for ch in iri.chars() {
        let cp = ch as u32;
        let needs_uchar = matches!(
            ch,
            '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\' | ' '
        ) || cp <= 0x20
            || (0x7F..=0x9F).contains(&cp);
        if needs_uchar {
            push_uchar(&mut out, cp);
        } else {
            out.push(ch);
        }
    }
    out.push('>');
    out
}

/// Serialize a literal in canonical form (REC §"A Canonical form of
/// N-Quads"): simple literals (`xsd:string`, or no datatype/lang) emit only
/// the quoted lexical form; lang-tagged literals append `@tag`; otherwise
/// the datatype IRI is appended via `^^`.
fn serialize_literal(lexical: &str, lang: Option<&str>, datatype: Option<&str>) -> String {
    let mut out = String::new();
    out.push('"');
    push_escaped_literal_body(&mut out, lexical);
    out.push('"');
    match (lang, datatype) {
        (Some(tag), _) => {
            out.push('@');
            out.push_str(tag);
        }
        (None, Some(dt)) if dt != XSD_STRING => {
            out.push_str("^^");
            out.push_str(&serialize_iri(dt));
        }
        // No lang, and datatype absent or exactly xsd:string → simple
        // literal: the datatype IRI is omitted.
        _ => {}
    }
    out
}

/// Apply the canonical `STRING_LITERAL_QUOTE` escaping table from the
/// REC appendix:
///
/// - `\b \t \n \f \r \" \\` → `ECHAR` (NB: `'` is *not* ECHAR-escaped in
///   canonical output — the suite's test060 shows `\'` decoding to a bare
///   `'`);
/// - `U+0000..U+0007`, `U+000B` (VT), `U+000E..U+001F`, `U+007F` (DEL),
///   and any non-`Char` code point → lowercase `\u` + 4 upper-hex `UCHAR`;
/// - everything else → native UTF-8.
fn push_escaped_literal_body(out: &mut String, lexical: &str) {
    for ch in lexical.chars() {
        let cp = ch as u32;
        match ch {
            '\u{0008}' => out.push_str("\\b"),
            '\u{0009}' => out.push_str("\\t"),
            '\u{000A}' => out.push_str("\\n"),
            '\u{000C}' => out.push_str("\\f"),
            '\u{000D}' => out.push_str("\\r"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => {
                if cp <= 0x07
                    || cp == 0x0B
                    || (0x0E..=0x1F).contains(&cp)
                    || cp == 0x7F
                    || !is_xml_char(cp)
                {
                    push_uchar(out, cp);
                } else {
                    out.push(ch);
                }
            }
        }
    }
}

/// Push a `UCHAR` for `cp`: lowercase `\u` + 4 upper-hex for the BMP,
/// lowercase `\U` + 8 upper-hex above it. The REC appendix pins HEX to
/// uppercase `[0-9A-F]` and the `\u`/`\U` marker to lowercase.
fn push_uchar(out: &mut String, cp: u32) {
    if cp <= 0xFFFF {
        out.push_str("\\u");
        out.push_str(&format!("{cp:04X}"));
    } else {
        out.push_str("\\U");
        out.push_str(&format!("{cp:08X}"));
    }
}

/// The XML 1.1 `Char` production (referenced by the canonical N-Quads
/// appendix): U+0009, U+000A, U+000D, U+0020..U+D7FF, U+E000..U+FFFD,
/// U+10000..U+10FFFF. Code points outside it MUST be emitted as `UCHAR`.
fn is_xml_char(cp: u32) -> bool {
    cp == 0x9
        || cp == 0xA
        || cp == 0xD
        || (0x20..=0xD7FF).contains(&cp)
        || (0xE000..=0xFFFD).contains(&cp)
        || (0x10000..=0x10FFFF).contains(&cp)
}

/// Serialize one quad as a single canonical N-Quads line, *including* its
/// trailing `LF` (REC appendix: the final `EOL` MUST be provided, and each
/// `EOL` is a single `LF`). Exactly one `U+0020` separates each component.
pub(crate) fn serialize_quad(quad: &Quad) -> String {
    let mut out = String::new();
    out.push_str(&serialize_term(quad.subject()));
    out.push(' ');
    out.push_str(&serialize_iri(quad.predicate()));
    out.push(' ');
    out.push_str(&serialize_term(quad.object()));
    out.push(' ');
    if let Some(g) = quad.graph() {
        out.push_str(&serialize_term(g));
        out.push(' ');
    }
    out.push('.');
    out.push('\n');
    out
}

#[cfg(test)]
mod totality_tests {
    use super::parse_nquads;
    use proptest::prelude::*;

    proptest! {
        /// Honest at the N-Quads input boundary: ∀ string the line-based parser
        /// returns `Ok`/`Err`, never panics (no index-out-of-bounds on the
        /// char-cursor for adversarial input).
        #[test]
        fn prop_parse_nquads_is_total(s in any::<String>()) {
            let _ = parse_nquads(&s);
        }
    }

    pr4xis::register_praxis_value!(prop_parse_nquads_is_total, Honest);
}
