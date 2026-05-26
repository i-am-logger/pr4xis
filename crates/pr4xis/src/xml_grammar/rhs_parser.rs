//! Parser for the W3C XML 1.0 EBNF Notation (Appendix B) as it
//! appears in the **xmlspec.dtd** rendering of the spec's `<rhs>`
//! elements. Produces a typed [`Term`] tree.
//!
//! ## Input
//!
//! The body of a `<rhs>` element is mixed XML markup + text:
//!
//! ```xml
//! <rhs>'&lt;!ELEMENT' <nt def="NT-S">S</nt>
//!      <nt def="NT-Name">Name</nt> <nt def="NT-S">S</nt>?
//!      '>'</rhs>
//! ```
//!
//! - `<nt def="NT-X">X</nt>` — nonterminal reference; `X` is the
//!   LHS name of the referenced production.
//! - Text content between elements uses Appendix B syntax:
//!   - `'…'` / `"…"`  — literal token (with `&lt;` / `&gt;` /
//!     `&amp;` / `&quot;` / `&apos;` XML-entity-escaped per
//!     §4.6).
//!   - `#xN`           — single Unicode code point (hex).
//!   - `[#xN-#xM]`     — inclusive range (hex).
//!   - `[a-z]`         — inclusive range (ASCII).
//!   - `|`             — alternation.
//!   - `(` / `)`       — grouping.
//!   - `?` / `*` / `+` — postfix quantifiers.
//!   - `-`             — set subtraction (binds tighter than `|`).
//!   - whitespace      — sequence separator.
//!
//! ## Operator precedence
//!
//! From tightest to loosest, per Appendix B:
//! 1. postfix quantifiers `?` `*` `+`
//! 2. subtraction `A - B`
//! 3. juxtaposition (sequence)
//! 4. alternation `|`
//!
//! Grouping `( … )` is parsed as a sub-RHS and reduces to a single
//! [`Term`]. Adjacent character-class atoms (`#xN`, `[#xN-#xM]`,
//! `[a-z]`) at any level fold together into a single
//! [`Term::CharClass`] when separated only by `|` — matching the
//! character-class productions of §2.2 / §2.3 (Char, NameStartChar,
//! NameChar).
//!
//! ## Literature grounding
//!
//! See [crate::xml_grammar] module doc for the full literature
//! stack: W3C XML 1.0 Appendix B, Ford 2002/2004 (PEG semantics),
//! Wirth 1977 + ISO/IEC 14977:1996 (EBNF lineage), dryruby/ebnf
//! (reference Ruby implementation).

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use super::ast::{CodePointRange, Term};

/// Errors returned by [`parse_rhs`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseRhsError {
    /// Tokenisation failed at the given byte offset within the RHS
    /// source — typically a malformed range bracket, unterminated
    /// literal, or unknown character.
    Tokenize { position: usize, what: String },
    /// Empty RHS. Productions must have at least one term.
    EmptyRhs,
    /// A postfix quantifier (`?` / `*` / `+`) appeared with no
    /// preceding term.
    DanglingQuantifier { position: usize },
    /// A binary operator (`|` / `-`) appeared with a missing
    /// operand on one side.
    DanglingOperator { position: usize, op: char },
    /// Unbalanced grouping parentheses.
    UnbalancedParen { position: usize },
}

impl core::fmt::Display for ParseRhsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tokenize { position, what } => {
                write!(f, "tokenize error at {position}: {what}")
            }
            Self::EmptyRhs => f.write_str("empty RHS — productions must have at least one term"),
            Self::DanglingQuantifier { position } => {
                write!(f, "dangling postfix quantifier at {position}")
            }
            Self::DanglingOperator { position, op } => {
                write!(f, "dangling operator '{op}' at {position}")
            }
            Self::UnbalancedParen { position } => write!(f, "unbalanced parenthesis at {position}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseRhsError {}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse one production's RHS (the body of `<rhs>` in xmlspec.dtd).
///
/// `rhs_content` is the raw inner-text of the `<rhs>` element,
/// including the `<nt def="NT-X">X</nt>` markup. XML entity
/// references (`&lt;` / `&gt;` / `&amp;` / `&quot;` / `&apos;`) are
/// recognised within literals.
pub fn parse_rhs(rhs_content: &str) -> Result<Term, ParseRhsError> {
    let tokens = tokenize(rhs_content)?;
    if tokens.is_empty() {
        return Err(ParseRhsError::EmptyRhs);
    }
    let mut tp = TokenParser::new(&tokens);
    let term = tp.parse_alternation()?;
    if tp.peek().is_some() {
        return Err(ParseRhsError::UnbalancedParen {
            position: tp.pos_byte(),
        });
    }
    Ok(term)
}

// ---------------------------------------------------------------------------
// Tokenisation — §2.4 of this module: from raw <rhs> text to a flat
// token sequence. Strips XML markup, decodes entity references in
// literals, recognises Appendix B's micro-syntax.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Nt(String),      // <nt def="NT-X">X</nt>
    Literal(String), // '…' or "…"
    Hex(u32),        // #xN
    Range(u32, u32), // [#xN-#xM] or [c-c]
    OpenParen,       // (
    CloseParen,      // )
    Pipe,            // |
    Minus,           // -
    Question,        // ?
    Star,            // *
    Plus,            // +
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    /// Byte offset of the token's first byte in the source RHS.
    pos: usize,
}

/// Walk the RHS text from start to end emitting tokens.
fn tokenize(s: &str) -> Result<Vec<Token>, ParseRhsError> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip whitespace (sequence separator — no token emitted).
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // <nt def="NT-X">X</nt>
        if s[i..].starts_with("<nt") {
            let (consumed, name) = read_nt_element(s, i)?;
            out.push(Token {
                tok: Tok::Nt(name),
                pos: i,
            });
            i += consumed;
            continue;
        }
        // Quoted literal: '…' or "…"
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i] as char;
            let (consumed, lit) = read_literal(s, i, quote)?;
            out.push(Token {
                tok: Tok::Literal(lit),
                pos: i,
            });
            i += consumed;
            continue;
        }
        // Hex code point: #xN  (single, distinct from range)
        if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'x') {
            let (consumed, code) = read_hex(s, i + 2)?;
            out.push(Token {
                tok: Tok::Hex(code),
                pos: i,
            });
            i += 2 + consumed;
            continue;
        }
        // Range: [#xN-#xM] or [c-c]
        if bytes[i] == b'[' {
            let (consumed, lo, hi) = read_range(s, i)?;
            out.push(Token {
                tok: Tok::Range(lo, hi),
                pos: i,
            });
            i += consumed;
            continue;
        }
        // Single-char operators.
        let single = match bytes[i] {
            b'(' => Some(Tok::OpenParen),
            b')' => Some(Tok::CloseParen),
            b'|' => Some(Tok::Pipe),
            b'-' => Some(Tok::Minus),
            b'?' => Some(Tok::Question),
            b'*' => Some(Tok::Star),
            b'+' => Some(Tok::Plus),
            _ => None,
        };
        if let Some(t) = single {
            out.push(Token { tok: t, pos: i });
            i += 1;
            continue;
        }
        return Err(ParseRhsError::Tokenize {
            position: i,
            what: format!("unexpected byte {:?}", bytes[i] as char),
        });
    }
    Ok(out)
}

/// Read `<nt def="NT-X">X</nt>` starting at position `i`. Returns
/// `(consumed_bytes, name)`.
fn read_nt_element(s: &str, i: usize) -> Result<(usize, String), ParseRhsError> {
    let rest = &s[i..];
    let close_open = rest.find('>').ok_or_else(|| ParseRhsError::Tokenize {
        position: i,
        what: "unterminated <nt> open tag".to_string(),
    })?;
    let after_open = close_open + 1;
    let inner_start = after_open;
    let close_tag = rest[inner_start..]
        .find("</nt>")
        .ok_or_else(|| ParseRhsError::Tokenize {
            position: i,
            what: "missing </nt>".to_string(),
        })?;
    let name = rest[inner_start..inner_start + close_tag]
        .trim()
        .to_string();
    let consumed = inner_start + close_tag + "</nt>".len();
    Ok((consumed, name))
}

/// Read a `'…'` or `"…"` literal starting at position `i` (the byte
/// at `i` is the opening quote). Decodes the five predefined XML
/// entity references per §4.6.
fn read_literal(s: &str, i: usize, quote: char) -> Result<(usize, String), ParseRhsError> {
    let rest = &s[i + 1..];
    let end = rest.find(quote).ok_or_else(|| ParseRhsError::Tokenize {
        position: i,
        what: format!("unterminated {quote} literal"),
    })?;
    let raw = &rest[..end];
    Ok((1 + end + 1, decode_entities(raw)))
}

/// Decode the five W3C XML 1.0 §4.6 predefined entities:
/// `&lt;` `&gt;` `&amp;` `&apos;` `&quot;`. Anything else passes
/// through unchanged — the spec's RHS literals only ever use these.
fn decode_entities(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let replacement = if rest.starts_with("&lt;") {
            ("<", "&lt;".len())
        } else if rest.starts_with("&gt;") {
            (">", "&gt;".len())
        } else if rest.starts_with("&amp;") {
            ("&", "&amp;".len())
        } else if rest.starts_with("&quot;") {
            ("\"", "&quot;".len())
        } else if rest.starts_with("&apos;") {
            ("'", "&apos;".len())
        } else {
            // Unknown entity — emit `&` and continue.
            ("&", 1)
        };
        out.push_str(replacement.0);
        rest = &rest[replacement.1..];
    }
    out.push_str(rest);
    out
}

/// Read hex digits starting at position `i`, terminated by any
/// non-hex byte. Returns `(consumed, code_point)`.
fn read_hex(s: &str, i: usize) -> Result<(usize, u32), ParseRhsError> {
    let bytes = s.as_bytes();
    let mut j = i;
    while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
        j += 1;
    }
    if j == i {
        return Err(ParseRhsError::Tokenize {
            position: i,
            what: "expected hex digits after #x".to_string(),
        });
    }
    let code = u32::from_str_radix(&s[i..j], 16).map_err(|_| ParseRhsError::Tokenize {
        position: i,
        what: format!("invalid hex code point {:?}", &s[i..j]),
    })?;
    Ok((j - i, code))
}

/// Read a `[lo-hi]` range starting at the `[`. Supports both
/// `[#xN-#xM]` (hex) and `[a-z]` (ASCII). Returns `(consumed, lo, hi)`.
fn read_range(s: &str, i: usize) -> Result<(usize, u32, u32), ParseRhsError> {
    let rest = &s[i..];
    let close = rest.find(']').ok_or_else(|| ParseRhsError::Tokenize {
        position: i,
        what: "unterminated [range]".to_string(),
    })?;
    let inner = &rest[1..close];
    // Two cases: `#xN-#xM` (hex on both sides) or `c-c` (ASCII).
    if let Some(stripped) = inner.strip_prefix("#x") {
        // `#xN-#xM`
        let (lo, rest_after_lo) = split_off_hex(stripped, i)?;
        let rest_after_lo =
            rest_after_lo
                .strip_prefix("-#x")
                .ok_or_else(|| ParseRhsError::Tokenize {
                    position: i,
                    what: "expected `-#x` separator in hex range".to_string(),
                })?;
        let (hi, tail) = split_off_hex(rest_after_lo, i)?;
        if !tail.is_empty() {
            return Err(ParseRhsError::Tokenize {
                position: i,
                what: format!("trailing bytes in hex range: {tail:?}"),
            });
        }
        Ok((close + 1, lo, hi))
    } else {
        // ASCII range: exactly 3 chars `c-c`.
        let chars: Vec<char> = inner.chars().collect();
        if chars.len() != 3 || chars[1] != '-' {
            return Err(ParseRhsError::Tokenize {
                position: i,
                what: format!("not an ASCII range: [{inner}]"),
            });
        }
        let lo = chars[0] as u32;
        let hi = chars[2] as u32;
        Ok((close + 1, lo, hi))
    }
}

/// Pull leading hex digits off `s` returning `(value, remainder)`.
fn split_off_hex<'a>(s: &'a str, byte_pos: usize) -> Result<(u32, &'a str), ParseRhsError> {
    let end = s
        .bytes()
        .position(|b| !b.is_ascii_hexdigit())
        .unwrap_or(s.len());
    if end == 0 {
        return Err(ParseRhsError::Tokenize {
            position: byte_pos,
            what: "expected hex digits".to_string(),
        });
    }
    let value = u32::from_str_radix(&s[..end], 16).map_err(|_| ParseRhsError::Tokenize {
        position: byte_pos,
        what: format!("invalid hex in range: {:?}", &s[..end]),
    })?;
    Ok((value, &s[end..]))
}

// ---------------------------------------------------------------------------
// Token-stream parser — recursive descent with explicit precedence.
// ---------------------------------------------------------------------------

struct TokenParser<'a> {
    tokens: &'a [Token],
    idx: usize,
}

impl<'a> TokenParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, idx: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.idx)
    }

    fn bump(&mut self) -> Option<&'a Token> {
        let t = self.tokens.get(self.idx);
        if t.is_some() {
            self.idx += 1;
        }
        t
    }

    fn pos_byte(&self) -> usize {
        self.tokens
            .get(self.idx)
            .or_else(|| self.tokens.last())
            .map(|t| t.pos)
            .unwrap_or(0)
    }

    /// Lowest precedence: alternation. `A | B | C` → `Alternation([A, B, C])`.
    /// Adjacent character-class branches fold into a single
    /// `Term::CharClass`.
    fn parse_alternation(&mut self) -> Result<Term, ParseRhsError> {
        let first = self.parse_sequence()?;
        if !matches!(self.peek().map(|t| &t.tok), Some(Tok::Pipe)) {
            return Ok(first);
        }
        let mut branches = vec![first];
        while matches!(self.peek().map(|t| &t.tok), Some(Tok::Pipe)) {
            let pipe_pos = self.bump().unwrap().pos;
            if self
                .peek()
                .map(|t| matches!(&t.tok, Tok::CloseParen | Tok::Pipe))
                .unwrap_or(true)
            {
                return Err(ParseRhsError::DanglingOperator {
                    position: pipe_pos,
                    op: '|',
                });
            }
            branches.push(self.parse_sequence()?);
        }
        Ok(fold_char_class(branches))
    }

    /// Mid precedence: sequence (juxtaposition). Reads atoms until
    /// we hit `|`, `)`, or end of input.
    fn parse_sequence(&mut self) -> Result<Term, ParseRhsError> {
        let mut items = Vec::new();
        while let Some(t) = self.peek() {
            if matches!(t.tok, Tok::Pipe | Tok::CloseParen) {
                break;
            }
            items.push(self.parse_subtraction()?);
        }
        if items.is_empty() {
            return Err(ParseRhsError::DanglingOperator {
                position: self.pos_byte(),
                op: '|',
            });
        }
        Ok(if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Term::Sequence(items)
        })
    }

    /// Subtraction: `A - B` is left-associative; the `B` operand is
    /// one quantified atom (the spec only uses this for `Char - 'something'`
    /// style productions, so a single atom on the right suffices).
    fn parse_subtraction(&mut self) -> Result<Term, ParseRhsError> {
        let lhs = self.parse_quantified()?;
        if let Some(Token {
            tok: Tok::Minus,
            pos,
        }) = self.peek()
        {
            let minus_pos = *pos;
            self.bump();
            // The right operand must exist.
            if self
                .peek()
                .map(|t| matches!(t.tok, Tok::Pipe | Tok::CloseParen))
                .unwrap_or(true)
            {
                return Err(ParseRhsError::DanglingOperator {
                    position: minus_pos,
                    op: '-',
                });
            }
            let rhs = self.parse_quantified()?;
            Ok(Term::Subtraction(Box::new(lhs), Box::new(rhs)))
        } else {
            Ok(lhs)
        }
    }

    /// Highest precedence: atom + optional postfix quantifier.
    fn parse_quantified(&mut self) -> Result<Term, ParseRhsError> {
        let atom = self.parse_atom()?;
        if let Some(t) = self.peek() {
            match t.tok {
                Tok::Question => {
                    self.bump();
                    return Ok(Term::Optional(Box::new(atom)));
                }
                Tok::Star => {
                    self.bump();
                    return Ok(Term::ZeroOrMore(Box::new(atom)));
                }
                Tok::Plus => {
                    self.bump();
                    return Ok(Term::OneOrMore(Box::new(atom)));
                }
                _ => {}
            }
        }
        Ok(atom)
    }

    /// One atomic term — literal, nt-ref, char-class atom, or
    /// parenthesised group.
    fn parse_atom(&mut self) -> Result<Term, ParseRhsError> {
        let t = self.bump().ok_or(ParseRhsError::EmptyRhs)?.clone();
        match t.tok {
            Tok::Nt(name) => Ok(Term::NonTerminal(name)),
            Tok::Literal(s) => Ok(Term::Literal(s)),
            Tok::Hex(code) => Ok(Term::CharClass(vec![CodePointRange { lo: code, hi: code }])),
            Tok::Range(lo, hi) => Ok(Term::CharClass(vec![CodePointRange { lo, hi }])),
            Tok::OpenParen => {
                let inner = self.parse_alternation()?;
                let close = self.bump();
                match close.map(|t| &t.tok) {
                    Some(Tok::CloseParen) => Ok(inner),
                    _ => Err(ParseRhsError::UnbalancedParen { position: t.pos }),
                }
            }
            Tok::CloseParen => Err(ParseRhsError::UnbalancedParen { position: t.pos }),
            Tok::Question | Tok::Star | Tok::Plus => {
                Err(ParseRhsError::DanglingQuantifier { position: t.pos })
            }
            Tok::Pipe => Err(ParseRhsError::DanglingOperator {
                position: t.pos,
                op: '|',
            }),
            Tok::Minus => Err(ParseRhsError::DanglingOperator {
                position: t.pos,
                op: '-',
            }),
        }
    }
}

/// Post-process alternation branches: any branch that is a singleton
/// [`Term::CharClass`] folds with adjacent siblings into one combined
/// `CharClass`. This matches the spec's character-class productions
/// (§2.2 Char, §2.3 NameStartChar / NameChar) which are pure
/// disjunctions of code-point alternatives.
fn fold_char_class(branches: Vec<Term>) -> Term {
    let mut ranges = Vec::new();
    let mut other_branches: Vec<Term> = Vec::new();
    for b in branches {
        match b {
            Term::CharClass(rs) => ranges.extend(rs),
            other => other_branches.push(other),
        }
    }
    if other_branches.is_empty() {
        return Term::CharClass(ranges);
    }
    if ranges.is_empty() {
        return if other_branches.len() == 1 {
            other_branches.pop().unwrap()
        } else {
            Term::Alternation(other_branches)
        };
    }
    // Mixed — keep the char-class atoms as a single Alternation branch
    // alongside the rest.
    let mut all = Vec::with_capacity(other_branches.len() + 1);
    all.push(Term::CharClass(ranges));
    all.extend(other_branches);
    Term::Alternation(all)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn nt(name: &str) -> Term {
        Term::NonTerminal(name.to_string())
    }

    fn lit(s: &str) -> Term {
        Term::Literal(s.to_string())
    }

    fn range(lo: u32, hi: u32) -> CodePointRange {
        CodePointRange { lo, hi }
    }

    #[test]
    fn parses_char_production_rhs() {
        // §2.2 [2] Char
        let rhs = "#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]";
        let t = parse_rhs(rhs).unwrap();
        assert_eq!(
            t,
            Term::CharClass(vec![
                range(9, 9),
                range(0xA, 0xA),
                range(0xD, 0xD),
                range(0x20, 0xD7FF),
                range(0xE000, 0xFFFD),
                range(0x10000, 0x10FFFF),
            ])
        );
    }

    #[test]
    fn parses_name_start_char_with_ascii_literals() {
        // §2.3 [4] NameStartChar — mixes "':'", "[A-Z]", "'_'", etc.
        let rhs = "\":\" | [A-Z] | \"_\" | [a-z]";
        let t = parse_rhs(rhs).unwrap();
        // ":" and "_" fold to char-class via the post-processing — let's
        // accept either: literal-with-1-char OR Alternation with literal.
        // Actually the parser emits Literal for "':'" because the source
        // uses quoted form. fold_char_class keeps them separate.
        match t {
            Term::Alternation(branches) => {
                // First branch should be the [A-Z] and [a-z] folded into
                // one CharClass.
                assert!(branches.iter().any(|b| matches!(b, Term::CharClass(_))));
                assert!(
                    branches
                        .iter()
                        .any(|b| matches!(b, Term::Literal(s) if s == ":"))
                );
                assert!(
                    branches
                        .iter()
                        .any(|b| matches!(b, Term::Literal(s) if s == "_"))
                );
            }
            _ => panic!("expected Alternation, got {t:?}"),
        }
    }

    #[test]
    fn parses_elementdecl_rhs_with_nt_refs_and_literal() {
        // §3.2 [45] elementdecl — verbatim from xmlspec.dtd format
        // (the `&lt;` entity is the spec's encoding of `<`).
        let rhs = "'&lt;!ELEMENT' <nt def=\"NT-S\">S</nt> <nt def=\"NT-Name\">Name</nt> \
                   <nt def=\"NT-S\">S</nt> <nt def=\"NT-contentspec\">contentspec</nt> \
                   <nt def=\"NT-S\">S</nt>? '>'";
        let t = parse_rhs(rhs).unwrap();
        match t {
            Term::Sequence(items) => {
                assert_eq!(items[0], lit("<!ELEMENT"));
                assert_eq!(items[1], nt("S"));
                assert_eq!(items[2], nt("Name"));
                assert_eq!(items[3], nt("S"));
                assert_eq!(items[4], nt("contentspec"));
                // S? — Optional(NonTerminal("S"))
                assert_eq!(items[5], Term::Optional(Box::new(nt("S"))));
                assert_eq!(items[6], lit(">"));
            }
            _ => panic!("expected Sequence, got {t:?}"),
        }
    }

    #[test]
    fn parses_name_char_with_nt_reference_then_alternation() {
        // §2.3 [4a] NameChar — references NameStartChar via <nt> then ORs
        // additional code points. `fold_char_class` groups char-class
        // atoms (#xN, [#xN-#xM], [0-9]) into one combined CharClass
        // branch; non-char-class branches (NT refs, ASCII-literal
        // singletons) sit alongside in the Alternation.
        let rhs = "<nt def=\"NT-NameStartChar\">NameStartChar</nt> | \"-\" | \".\" | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]";
        let t = parse_rhs(rhs).unwrap();
        match t {
            Term::Alternation(branches) => {
                // NameStartChar reference is present (position may
                // shift after char-class folding).
                assert!(
                    branches
                        .iter()
                        .any(|b| matches!(b, Term::NonTerminal(s) if s == "NameStartChar"))
                );
                // Exactly one folded CharClass with all four code-point
                // atoms: [0-9], #xB7, [#x0300-#x036F], [#x203F-#x2040].
                let cc_count = branches
                    .iter()
                    .filter(|b| matches!(b, Term::CharClass(_)))
                    .count();
                assert_eq!(cc_count, 1, "char-class atoms must fold to one branch");
                let cc = branches
                    .iter()
                    .find_map(|b| match b {
                        Term::CharClass(r) => Some(r),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(cc.len(), 4);
                assert!(
                    branches
                        .iter()
                        .any(|b| matches!(b, Term::Literal(s) if s == "-"))
                );
                assert!(
                    branches
                        .iter()
                        .any(|b| matches!(b, Term::Literal(s) if s == "."))
                );
            }
            _ => panic!("expected Alternation, got {t:?}"),
        }
    }

    #[test]
    fn parses_subtraction_for_comment_char() {
        // §2.5 [15] Comment body uses (Char - '-')
        // Use a small fixture with just the subtraction.
        let rhs = "<nt def=\"NT-Char\">Char</nt> - \"-\"";
        let t = parse_rhs(rhs).unwrap();
        assert_eq!(
            t,
            Term::Subtraction(Box::new(nt("Char")), Box::new(lit("-")))
        );
    }

    #[test]
    fn parses_kleene_star_and_plus() {
        // §3 [43] content uses ... (... CharData?)* etc.
        let rhs = "<nt def=\"NT-A\">A</nt>* <nt def=\"NT-B\">B</nt>+";
        let t = parse_rhs(rhs).unwrap();
        assert_eq!(
            t,
            Term::Sequence(vec![
                Term::ZeroOrMore(Box::new(nt("A"))),
                Term::OneOrMore(Box::new(nt("B"))),
            ])
        );
    }

    #[test]
    fn parses_grouping_for_sequence_inside_alternation() {
        // (A B) | C — grouping forces precedence.
        let rhs = "(<nt def=\"NT-A\">A</nt> <nt def=\"NT-B\">B</nt>) | <nt def=\"NT-C\">C</nt>";
        let t = parse_rhs(rhs).unwrap();
        assert_eq!(
            t,
            Term::Alternation(vec![Term::Sequence(vec![nt("A"), nt("B")]), nt("C"),])
        );
    }

    #[test]
    fn decodes_xml_predefined_entities_in_literals() {
        let rhs = "'&lt;' | '&gt;' | '&amp;' | '&quot;' | '&apos;'";
        let t = parse_rhs(rhs).unwrap();
        match t {
            Term::Alternation(branches) => {
                let lits: Vec<&str> = branches
                    .iter()
                    .filter_map(|b| {
                        if let Term::Literal(s) = b {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .collect();
                assert!(lits.contains(&"<"));
                assert!(lits.contains(&">"));
                assert!(lits.contains(&"&"));
                assert!(lits.contains(&"\""));
                assert!(lits.contains(&"'"));
            }
            _ => panic!("expected Alternation"),
        }
    }

    #[test]
    fn rejects_empty_rhs() {
        assert!(matches!(parse_rhs(""), Err(ParseRhsError::EmptyRhs)));
        assert!(matches!(parse_rhs("   "), Err(ParseRhsError::EmptyRhs)));
    }

    #[test]
    fn rejects_dangling_quantifier() {
        assert!(matches!(
            parse_rhs("?"),
            Err(ParseRhsError::DanglingQuantifier { .. })
        ));
    }

    #[test]
    fn rejects_unbalanced_paren() {
        let r = parse_rhs("( #x20");
        assert!(matches!(r, Err(ParseRhsError::UnbalancedParen { .. })));
    }
}
