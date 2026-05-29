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
    // Pre-pass: collapse XML `&nbsp;` to plain space. The
    // xmlspec.dtd rendering of the spec uses `&nbsp;` for
    // layout-only non-breaking space between alternation branches
    // (it never appears inside a literal). The other §4.6
    // predefined entities (`&lt;` / `&gt;` / `&amp;` / `&quot;` /
    // `&apos;`) ONLY appear inside `'…'` or `"…"` literal tokens
    // (e.g. `'&lt;!ELEMENT'`) — those are decoded in
    // `read_literal` via [`decode_entities`], NOT here, because
    // pre-decoding `&apos;` to `'` would terminate the enclosing
    // single-quoted literal prematurely.
    let preprocessed = collapse_nbsp(rhs_content);
    let tokens = tokenize(&preprocessed)?;
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

/// Collapse every `&nbsp;` to a single ASCII space. The spec uses
/// `&nbsp;` purely for typographic layout — semantically it's a
/// sequence separator, identical to a space in the EBNF.
fn collapse_nbsp(raw: &str) -> String {
    raw.replace("&nbsp;", " ")
}

// ---------------------------------------------------------------------------
// Tokenisation — §2.4 of this module: from raw <rhs> text to a flat
// token sequence. Strips XML markup, decodes entity references in
// literals, recognises Appendix B's micro-syntax.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Nt(String),                        // <nt def="NT-X">X</nt>
    Literal(String),                   // '…' or "…"
    Hex(u32),                          // #xN
    CharClass(Vec<CodePointRange>),    // [#xN-#xM] | [c-c] | [abc] (one or more atoms)
    NegatedChars(Vec<CodePointRange>), // [^abc] / [^#xN-#xM] — complement w.r.t. §2.2 Char
    OpenParen,                         // (
    CloseParen,                        // )
    Pipe,                              // |
    Minus,                             // -
    Question,                          // ?
    Star,                              // *
    Plus,                              // +
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
        // Character class: [#xN-#xM] | [c-c] | [abc...] | [^...]
        if bytes[i] == b'[' {
            let (consumed, ranges, negated) = read_char_class(s, i)?;
            let tok = if negated {
                Tok::NegatedChars(ranges)
            } else {
                Tok::CharClass(ranges)
            };
            out.push(Token { tok, pos: i });
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

/// Decode the W3C XML 1.0 §4.6 predefined entities plus the
/// `&nbsp;` HTML/XHTML reference used by the xmlspec.dtd rendering
/// of the spec for layout-only non-breaking spaces.
///
/// Decoded:
/// - `&lt;`   → `<`
/// - `&gt;`   → `>`
/// - `&amp;`  → `&`
/// - `&apos;` → `'`
/// - `&quot;` → `"`
/// - `&nbsp;` → ` ` (collapsed to plain space — the entity carries
///   no semantic content in spec EBNF)
///
/// Other entity references pass through unchanged (the `&`
/// character is preserved literally) — the spec doesn't use any
/// outside those listed.
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
        } else if rest.starts_with("&nbsp;") {
            (" ", "&nbsp;".len())
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

/// Read a W3C XML 1.0 Appendix B character-class bracket starting at
/// the `[`. Recognises the full Appendix B form:
///
/// - `[c-c]`               — ASCII range
/// - `[#xN-#xM]`           — hex range
/// - `[#xN]`               — single hex code point
/// - `[abc]`               — multi-char ASCII enumeration
/// - `[a-zA-Z0-9]`         — multiple atoms in one class
/// - `[^…]`                — complement (negated) w.r.t. §2.2 `Char`
///
/// Returns `(consumed_bytes, atoms, negated)`. The interpreter
/// resolves a negated class against the loaded §2.2 `Char` production
/// (i.e. as `Char - CharClass(atoms)`) — the spec uses `[^X]` to mean
/// "any §2.2 Char that's not in X".
fn read_char_class(s: &str, i: usize) -> Result<(usize, Vec<CodePointRange>, bool), ParseRhsError> {
    let rest = &s[i..];
    let close = rest.find(']').ok_or_else(|| ParseRhsError::Tokenize {
        position: i,
        what: "unterminated [char-class]".to_string(),
    })?;
    let raw_inner = &rest[1..close];
    let consumed = close + 1;

    // The character-class body is XML text in the spec source:
    // `[^&lt;&amp;"]` (in the AttValue / EntityValue productions)
    // means three excluded chars `<`, `&`, `"`. Decode the §4.6
    // predefined entity references before parsing atoms — without
    // this, the tokenizer would read the 5 chars of `&lt;` as five
    // separate excluded atoms (`&`, `l`, `t`, `;`, …), and any
    // attribute value containing the letter `l`, `t`, `a`, `m`, or
    // `p` would silently fail.
    let decoded = decode_entities(raw_inner);
    let mut inner: &str = &decoded;

    let negated = if let Some(stripped) = inner.strip_prefix('^') {
        inner = stripped;
        true
    } else {
        false
    };

    let mut atoms = Vec::new();
    while !inner.is_empty() {
        let (atom, rest_inner) = parse_class_atom(inner, i)?;
        atoms.push(atom);
        inner = rest_inner;
    }
    if atoms.is_empty() {
        return Err(ParseRhsError::Tokenize {
            position: i,
            what: "empty character class".to_string(),
        });
    }
    Ok((consumed, atoms, negated))
}

/// Parse one Appendix B character-class atom off the front of `inner`.
/// Atoms are one of: `#xN-#xM`, `#xN`, `c-c`, `c`.
fn parse_class_atom(inner: &str, byte_pos: usize) -> Result<(CodePointRange, &str), ParseRhsError> {
    // Hex single or hex range.
    if let Some(after_hash) = inner.strip_prefix("#x") {
        let (lo, after_lo) = split_off_hex(after_hash, byte_pos)?;
        if let Some(after_dash_hash) = after_lo.strip_prefix("-#x") {
            let (hi, tail) = split_off_hex(after_dash_hash, byte_pos)?;
            return Ok((CodePointRange { lo, hi }, tail));
        }
        return Ok((CodePointRange { lo, hi: lo }, after_lo));
    }
    // ASCII single or ASCII range.
    let first = inner
        .chars()
        .next()
        .ok_or_else(|| ParseRhsError::Tokenize {
            position: byte_pos,
            what: "expected a character-class atom".to_string(),
        })?;
    let after_first = &inner[first.len_utf8()..];
    // Peek for `-` followed by another char to form a range, but
    // only when the `-` is not the start of a trailing literal `-`.
    if let Some(rest_after_dash) = after_first.strip_prefix('-')
        && let Some(second) = rest_after_dash.chars().next()
    {
        let lo = first as u32;
        let hi = second as u32;
        let consumed = first.len_utf8() + 1 + second.len_utf8();
        return Ok((CodePointRange { lo, hi }, &inner[consumed..]));
    }
    // Single ASCII char.
    let cp = first as u32;
    Ok((CodePointRange { lo: cp, hi: cp }, after_first))
}

/// Pull leading hex digits off `s` returning `(value, remainder)`.
fn split_off_hex(s: &str, byte_pos: usize) -> Result<(u32, &str), ParseRhsError> {
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
            Tok::CharClass(ranges) => Ok(Term::CharClass(ranges)),
            // Negated class `[^X]` = §2.2 `Char` minus the atoms.
            // Encoded via Term::Subtraction with a NonTerminal ref to
            // the loaded `Char` production (Bray et al. 2008 §2.2 [2]).
            Tok::NegatedChars(ranges) => Ok(Term::Subtraction(
                Box::new(Term::NonTerminal("Char".to_string())),
                Box::new(Term::CharClass(ranges)),
            )),
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
    fn parses_default_decl_rhs_with_nested_optional_group() {
        // §3.3 [60] DefaultDecl as it appears in the spec — two
        // `<rhs>` blocks the loader concatenates with a space.
        // The third branch contains an outer paren that groups a
        // sequence `('#FIXED' S)? AttValue` — a nested Optional
        // wrapping an inner-paren Sequence of literal + NT.
        let rhs = r#"'#REQUIRED' | '#IMPLIED'  | (('#FIXED' <nt def="NT-S">S</nt>)? <nt def="NT-AttValue">AttValue</nt>)"#;
        let t = parse_rhs(rhs).unwrap();
        assert_eq!(
            t,
            Term::Alternation(vec![
                lit("#REQUIRED"),
                lit("#IMPLIED"),
                Term::Sequence(vec![
                    Term::Optional(Box::new(Term::Sequence(vec![lit("#FIXED"), nt("S")]))),
                    nt("AttValue"),
                ]),
            ])
        );
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
