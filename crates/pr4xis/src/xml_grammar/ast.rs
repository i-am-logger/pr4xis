//! Typed AST for W3C XML 1.0 EBNF Notation (Appendix B).
//!
//! The variants are exactly the syntactic forms Appendix B uses:
//! literal token, nonterminal reference, character class, the four
//! composition operators (sequence, alternation, three quantifiers),
//! and set subtraction.
//!
//! ## Why owned (`Vec`-based) instead of static (`&'static [_]`)
//!
//! The grammar is loaded once at startup via [`crate::xml_grammar::parse_rhs`]
//! against the registered W3C XML 1.0 spec bytes, then cached by the
//! consumer via `OnceLock`. Construction happens once; subsequent
//! lookups borrow `&Production`. The owned representation lets the
//! same parser-output type serve both build-time (codegen tests) and
//! run-time (interpreter input) callers without const-fn gymnastics.

use alloc::{boxed::Box, string::String, vec::Vec};

/// One inclusive Unicode code-point range — the leaf of character
/// class productions like §2.2 `Char` and §2.3 `NameStartChar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodePointRange {
    /// Lower bound (inclusive).
    pub lo: u32,
    /// Upper bound (inclusive).
    pub hi: u32,
}

impl CodePointRange {
    /// True iff `c` lies in the inclusive range.
    #[must_use]
    pub fn contains(&self, c: u32) -> bool {
        c >= self.lo && c <= self.hi
    }
}

/// One node of the W3C XML 1.0 EBNF AST per Appendix B.
///
/// Operator precedence in the source notation (from highest to lowest):
/// `?` `*` `+` (postfix quantifiers), juxtaposition (sequence),
/// `-` (subtraction), `|` (alternation). Grouping `( … )` makes this
/// explicit. The parser in `super::rhs_parser` (private) reads source text
/// into this tree with that precedence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// A literal string token. Appendix B: `'…'` or `"…"`. Example:
    /// `'<!ELEMENT'` in §3.2 \[45\] `elementdecl`.
    Literal(String),
    /// A nonterminal reference — the LHS name of another production.
    /// Appendix B: italicised name in the printed spec; `<nt def="NT-X">X</nt>`
    /// markup in the xmlspec.dtd source form. Example: `Name` inside
    /// `elementdecl`'s RHS.
    NonTerminal(String),
    /// A character class — alternation of single hex code points and
    /// inclusive ranges. Appendix B: `#xN | #xM | [#xN-#xM] | [a-z]`.
    /// Stored as a disjunction; matching scans the cursor's next
    /// character against every range.
    CharClass(Vec<CodePointRange>),
    /// Juxtaposition: each term must match in document order,
    /// advancing the cursor between. Empty `Vec` matches empty
    /// input.
    Sequence(Vec<Term>),
    /// Alternation `A | B | …` — Ford 2004 §2.2 ordered choice:
    /// try branches left-to-right; first match wins; restore the
    /// cursor on each failed branch.
    Alternation(Vec<Term>),
    /// `A?` — zero or one occurrence. Always succeeds; advances iff
    /// `A` matched (Ford 2004 §2.3 "optional").
    Optional(Box<Term>),
    /// `A*` — zero or more (Ford 2004 §2.3 "Kleene closure"). Always
    /// succeeds; consumes greedy as long as `A` keeps matching.
    ZeroOrMore(Box<Term>),
    /// `A+` — one or more (Ford 2004 §2.3 "positive closure").
    /// Equivalent to `A A*`.
    OneOrMore(Box<Term>),
    /// `A - B` — set subtraction. The interpreter matches `A` then
    /// verifies the matched text does NOT also match `B`. Used by
    /// §2.5 Comment (`Char - '-'`) and §2.7 CDSect.
    Subtraction(Box<Term>, Box<Term>),
}

/// One W3C XML 1.0 production: `lhs ::= rhs` plus the production's
/// bracketed number from the spec (e.g. `"45"` for `elementdecl`,
/// `"4a"` for `NameChar`).
///
/// The interpreter looks productions up by [`Production::name`]
/// when expanding a [`Term::NonTerminal`] reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Production {
    /// Left-hand-side name (e.g. `"elementdecl"`).
    pub name: String,
    /// Spec-assigned number (e.g. `"45"`, `"4a"`). Some productions
    /// use letter-suffixed numbers — the field is `String`, not
    /// `u32`, to preserve the published identifier.
    pub number: String,
    /// Right-hand-side AST.
    pub rhs: Term,
}

/// A loaded W3C XML 1.0 grammar — every production parsed from the
/// spec, indexed by LHS name for the interpreter's `NonTerminal`
/// lookups.
///
/// Loaded once via `OnceLock`-cached call to
/// [`crate::xml_grammar::load_grammar`] (M5.ζ.2 follow-up; not yet
/// in this module).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Grammar {
    productions: Vec<Production>,
}

impl Grammar {
    /// Empty grammar.
    #[must_use]
    pub fn new() -> Self {
        Self {
            productions: Vec::new(),
        }
    }

    /// Add a production. Productions are appended; the interpreter
    /// looks them up by name, so the first occurrence of a given
    /// `name` wins (consistent with §4.5 "first declaration wins"
    /// semantics for entities, though productions in the spec are
    /// non-duplicate by construction).
    pub fn add(&mut self, production: Production) {
        self.productions.push(production);
    }

    /// Find a production by LHS name. Linear scan — the W3C XML 1.0
    /// grammar has 86 productions; binary-search / hashmap is not
    /// worth the dependency.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&Production> {
        self.productions.iter().find(|p| p.name == name)
    }

    /// Total number of productions loaded.
    #[must_use]
    pub fn len(&self) -> usize {
        self.productions.len()
    }

    /// True iff no productions have been added.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.productions.is_empty()
    }

    /// Iterate over all productions in load order.
    pub fn productions(&self) -> impl Iterator<Item = &Production> {
        self.productions.iter()
    }
}
