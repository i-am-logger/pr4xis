//! Parsing Expression Grammar interpreter for the W3C XML 1.0
//! EBNF, consuming the typed [`Grammar`] loaded by
//! [`crate::xml_grammar::load_grammar`].
//!
//! ## Semantic model
//!
//! Ford's Parsing Expression Grammar (PEG) formalism — Ford (2004)
//! *POPL '04* §2 — interprets each [`Term`] as a recognition
//! function `Term → Input → Position → MatchResult`:
//!
//! | Term variant            | Semantics                                                  |
//! |-------------------------|------------------------------------------------------------|
//! | [`Literal(s)`]          | succeeds iff `input[pos..]` starts with `s`               |
//! | [`NonTerminal(n)`]      | recursively matches the named production                  |
//! | [`CharClass(rs)`]       | succeeds on one Unicode code point inside one of `rs`     |
//! | [`Sequence(items)`]     | each item matches in order; cursor advances between       |
//! | [`Alternation(brs)`]    | tries every branch; longest match wins (leftmost on tie)  |
//! | [`Optional(t)`]         | always succeeds; advances iff `t` matched                 |
//! | [`ZeroOrMore(t)`]       | greedy Kleene closure; always succeeds                    |
//! | [`OneOrMore(t)`]        | greedy positive closure; `t` must match at least once     |
//! | [`Subtraction(a, b)`]   | `a` matches AND that same text doesn't also match `b`     |
//!
//! ## Alternation semantics — longest-match, not PEG ordered choice
//!
//! Ford's PEG (2004 §2.2) makes alternation ordered: the first
//! branch that matches wins. That semantics is wrong for the W3C
//! XML 1.0 grammar, which is **context-free** and assumes the
//! conventional CFG resolution: when several branches match,
//! prefer the longest. Concretely, §3.3.1 [56]
//! `TokenizedType ::= 'ID' | 'IDREF' | 'IDREFS' | 'ENTITY' |
//! 'ENTITIES' | 'NMTOKEN' | 'NMTOKENS'` lists shorter prefixes
//! first; on input `IDREFS`, PEG ordered choice would commit to
//! the 2-char `ID` match, the outer rule would then expect S, and
//! the whole declaration would be (wrongly) rejected — the spec's
//! intent is clearly to match `IDREFS` greedily. Birman & Ullman's
//! *Top-Down Parsing with Selectable Outputs* (TDPL/GTDPL, 1973)
//! formalises an alternative recursive-descent semantics that
//! commits to the longest matching alternative; the praxis
//! interpreter implements that.
//!
//! Concretely, [`match_term`]'s `Alternation` arm tries every
//! branch from the same start position and keeps the largest
//! `end_pos`. On a tie, the leftmost (smaller-index) branch wins
//! — same convention as POSIX regex BRE/ERE and as `lex`/`flex`
//! (Lesk 1975).
//!
//! See Mascarenhas, Medeiros & Ierusalimschy (2014) *Science of
//! Computer Programming* 96.2 on the gap between CFG and PEG and
//! the role of choice-semantics in bridging them.
//!
//! ## Linear-time guarantee — Packrat memoisation
//!
//! Ford (2002a) "Packrat Parsing: a Practical Linear-Time Algorithm
//! with Backtracking" (MIT Master's thesis); Ford (2002b) "Packrat
//! Parsing: Simple, Powerful, Lazy, Linear Time" (*ICFP '02*). The
//! cache holds `(production_name, cursor_position) → MatchResult`.
//! Every time the interpreter recurses into a `NonTerminal`, it
//! first consults the cache; cache hits return immediately. With N
//! productions and an input of length L, the cache has at most
//! `N × L` entries — Ford 2002a §4.1's linear-time bound.
//!
//! ## What this module is NOT
//!
//! - Not a parser-AST builder. The interpreter returns a single
//!   end-position when a production matches; it does NOT emit a
//!   parse tree. M5.ζ.4 will layer AST construction on top by
//!   passing capture-handlers per production.
//! - Not lossless on whitespace. The interpreter is faithful to the
//!   PEG semantics; whitespace handling is whatever the grammar
//!   says (the W3C XML 1.0 grammar uses explicit `S` nonterminals
//!   so this is faithful).
//!
//! ## Literature
//!
//! See [`crate::xml_grammar`] module doc for the full literature
//! stack. The PEG layer specifically cites Ford 2002a (linear-time
//! proof), Ford 2002b (functional pearl), Ford 2004 (POPL
//! formalism), and Hutton & Meijer 1996 (parser combinator
//! abstraction — the `Parser a = String → Maybe (a, String)` type
//! mirrors our `Term → &str → Pos → MatchResult`).

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

use alloc::string::{String, ToString};

use super::ast::{Grammar, Term};

/// Outcome of a single attempted match.
///
/// `Match { end_pos }` carries the byte offset (in the input) past
/// the last matched character. `NoMatch` is silent — no error
/// metadata, mirroring Ford 2004 §2.1 where failure is just absence
/// of success. (Diagnostic-quality error reporting is M5.ζ.4 work.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchResult {
    /// The term matched the input starting at the call's `pos`
    /// argument; `end_pos` is the cursor position past the last
    /// matched byte.
    Match {
        /// Byte offset in the input past the last matched character.
        end_pos: usize,
    },
    /// The term did not match at the given position. Per Ford 2004
    /// §2.2 ordered-choice semantics, this is a clean failure: the
    /// caller can try the next alternative or backtrack.
    NoMatch,
}

/// W3C XML 1.0 grammar interpreter.
///
/// Construct via [`Interpreter::new`] over a `(grammar, input)`
/// pair — the cache is valid only for that input, per Ford 2002a
/// §4.1's semantics (a packrat table memoises decisions about the
/// fixed input string). To recognise a different input, construct
/// a fresh `Interpreter`.
#[derive(Debug)]
pub struct Interpreter<'g, 'i> {
    grammar: &'g Grammar,
    input: &'i str,
    /// Packrat memoisation table per Ford 2002a §4.1. Keyed by
    /// `(production_name, cursor_position)`; the cache is built up
    /// lazily as productions are first visited at each position.
    /// Validity is scoped to the bound `input` — recognising a
    /// different input requires a new `Interpreter`.
    cache: HashMap<(String, usize), MatchResult>,
}

impl<'g, 'i> Interpreter<'g, 'i> {
    /// Build a fresh interpreter over `(grammar, input)`.
    #[must_use]
    pub fn new(grammar: &'g Grammar, input: &'i str) -> Self {
        Self {
            grammar,
            input,
            cache: HashMap::new(),
        }
    }

    /// The bound input.
    #[must_use]
    pub fn input(&self) -> &'i str {
        self.input
    }

    /// Match the named production starting at `pos`. Cached on
    /// `(name, pos)`.
    pub fn match_production(&mut self, name: &str, pos: usize) -> MatchResult {
        // Packrat lookup — Ford 2002a §4.1.
        let key = (name.to_string(), pos);
        if let Some(cached) = self.cache.get(&key) {
            return *cached;
        }
        // Tentatively cache NoMatch to break direct/indirect left-
        // recursion cycles; per Ford 2002a §5, left recursion in PEG
        // is detected by the cache: a recursive call into the same
        // production at the same position sees NoMatch and returns
        // immediately. The W3C XML 1.0 grammar contains no left
        // recursion, so this is a safety belt, not a feature.
        self.cache.insert(key.clone(), MatchResult::NoMatch);

        let rhs = match self.grammar.lookup(name) {
            Some(p) => p.rhs.clone(),
            None => return MatchResult::NoMatch,
        };
        let result = self.match_term(&rhs, pos);
        self.cache.insert(key, result);
        result
    }

    /// Match an arbitrary [`Term`] at `pos`. This is the dispatch
    /// for every Appendix B operator.
    pub fn match_term(&mut self, term: &Term, pos: usize) -> MatchResult {
        match term {
            Term::Literal(s) => {
                if pos <= self.input.len() && self.input[pos..].starts_with(s.as_str()) {
                    MatchResult::Match {
                        end_pos: pos + s.len(),
                    }
                } else {
                    MatchResult::NoMatch
                }
            }
            Term::NonTerminal(name) => self.match_production(name, pos),
            Term::CharClass(ranges) => match_char_class(self.input, pos, ranges),
            Term::Sequence(items) => {
                let mut current = pos;
                for item in items {
                    match self.match_term(item, current) {
                        MatchResult::Match { end_pos } => current = end_pos,
                        MatchResult::NoMatch => return MatchResult::NoMatch,
                    }
                }
                MatchResult::Match { end_pos: current }
            }
            Term::Alternation(branches) => {
                // Longest-match alternation (see module-doc:
                // "Alternation semantics") — try every branch from
                // `pos` and keep the largest `end_pos`. Birman &
                // Ullman 1973 (GTDPL). On a tie the leftmost
                // (earlier-declared) branch wins, matching the
                // POSIX regex / lex(1) convention.
                let mut best: Option<usize> = None;
                for branch in branches {
                    if let MatchResult::Match { end_pos } = self.match_term(branch, pos)
                        && best.is_none_or(|b| end_pos > b)
                    {
                        best = Some(end_pos);
                    }
                }
                match best {
                    Some(end_pos) => MatchResult::Match { end_pos },
                    None => MatchResult::NoMatch,
                }
            }
            Term::Optional(inner) => match self.match_term(inner, pos) {
                MatchResult::Match { end_pos } => MatchResult::Match { end_pos },
                MatchResult::NoMatch => MatchResult::Match { end_pos: pos },
            },
            Term::ZeroOrMore(inner) => {
                let mut current = pos;
                loop {
                    match self.match_term(inner, current) {
                        // Ford 2004 §2.3: progress check prevents
                        // infinite loops on terms that match empty.
                        MatchResult::Match { end_pos } if end_pos > current => current = end_pos,
                        _ => break,
                    }
                }
                MatchResult::Match { end_pos: current }
            }
            Term::OneOrMore(inner) => match self.match_term(inner, pos) {
                MatchResult::Match { end_pos: first_end } => {
                    let mut current = first_end;
                    loop {
                        match self.match_term(inner, current) {
                            MatchResult::Match { end_pos } if end_pos > current => {
                                current = end_pos
                            }
                            _ => break,
                        }
                    }
                    MatchResult::Match { end_pos: current }
                }
                MatchResult::NoMatch => MatchResult::NoMatch,
            },
            Term::Subtraction(a, b) => {
                // W3C XML 1.0 Appendix B `A - B`: A's matched text
                // is accepted iff that same text would not also
                // match B from the same starting position. The
                // canonical use is §2.5 Comment's `Char - '-'`.
                let a_result = self.match_term(a, pos);
                if let MatchResult::Match { end_pos: a_end } = a_result {
                    if let MatchResult::Match { end_pos: b_end } = self.match_term(b, pos)
                        && b_end == a_end
                    {
                        return MatchResult::NoMatch;
                    }
                    MatchResult::Match { end_pos: a_end }
                } else {
                    MatchResult::NoMatch
                }
            }
        }
    }
}

/// Scan the next Unicode code point at `pos` against the disjunction
/// of inclusive ranges. Advances by the UTF-8 byte length on match.
fn match_char_class(input: &str, pos: usize, ranges: &[super::ast::CodePointRange]) -> MatchResult {
    if pos > input.len() {
        return MatchResult::NoMatch;
    }
    if let Some(c) = input[pos..].chars().next() {
        let cp = c as u32;
        for r in ranges {
            if r.contains(cp) {
                return MatchResult::Match {
                    end_pos: pos + c.len_utf8(),
                };
            }
        }
    }
    MatchResult::NoMatch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_grammar::load_grammar;

    fn matches_completely(result: MatchResult, expected_end: usize) -> bool {
        matches!(result, MatchResult::Match { end_pos } if end_pos == expected_end)
    }

    #[test]
    fn matches_literal() {
        let grammar = Grammar::new();
        let t = Term::Literal("<!ELEMENT".to_string());
        let mut ok = Interpreter::new(&grammar, "<!ELEMENT doc");
        assert!(matches_completely(ok.match_term(&t, 0), "<!ELEMENT".len()));
        let mut not_ok = Interpreter::new(&grammar, "<doc/>");
        assert!(matches!(not_ok.match_term(&t, 0), MatchResult::NoMatch));
    }

    #[test]
    fn matches_char_class_from_loaded_char_production() {
        let spec = "<prod id=\"NT-Char\" num=\"2\">\
            <lhs>Char</lhs>\
            <rhs>#x9 | #xA | #xD | [#x20-#xD7FF]</rhs>\
        </prod>";
        let grammar = load_grammar(spec).unwrap();
        // 'A' (U+0041) is in [#x20-#xD7FF]
        let mut a = Interpreter::new(&grammar, "A");
        assert!(matches_completely(a.match_production("Char", 0), 1));
        // Tab (\t = 0x09) matches the single #x9 atom.
        let mut tab = Interpreter::new(&grammar, "\t");
        assert!(matches_completely(tab.match_production("Char", 0), 1));
        // NUL (0x00) is not in any range.
        let mut nul = Interpreter::new(&grammar, "\0");
        assert!(matches!(
            nul.match_production("Char", 0),
            MatchResult::NoMatch
        ));
    }

    #[test]
    fn matches_sequence_via_real_name_production() {
        // Mini grammar: Name = NameStartChar (NameChar)*
        // NameStartChar = [A-Z] | "_" | [a-z]
        // NameChar = NameStartChar | "-" | [0-9]
        let spec = "
            <prod id=\"NT-NameStartChar\" num=\"4\">
                <lhs>NameStartChar</lhs>
                <rhs>[A-Z] | \"_\" | [a-z]</rhs>
            </prod>
            <prod id=\"NT-NameChar\" num=\"4a\">
                <lhs>NameChar</lhs>
                <rhs><nt def=\"NT-NameStartChar\">NameStartChar</nt> | \"-\" | [0-9]</rhs>
            </prod>
            <prod id=\"NT-Name\" num=\"5\">
                <lhs>Name</lhs>
                <rhs><nt def=\"NT-NameStartChar\">NameStartChar</nt> (<nt def=\"NT-NameChar\">NameChar</nt>)*</rhs>
            </prod>
        ";
        let grammar = load_grammar(spec).unwrap();
        let mut foo = Interpreter::new(&grammar, "foo-bar123");
        assert!(matches_completely(
            foo.match_production("Name", 0),
            "foo-bar123".len()
        ));
        let mut underscore = Interpreter::new(&grammar, "_");
        assert!(matches_completely(
            underscore.match_production("Name", 0),
            1
        ));
        // A leading digit isn't a NameStartChar.
        let mut digit = Interpreter::new(&grammar, "1abc");
        assert!(matches!(
            digit.match_production("Name", 0),
            MatchResult::NoMatch
        ));
    }

    #[test]
    fn matches_alternation_longest_branch_wins() {
        // The W3C XML 1.0 grammar is CFG-style, so alternation here
        // is longest-match (Birman & Ullman 1973 GTDPL "longest"
        // selection), not Ford-PEG ordered choice. This is the
        // module-doc-cited deviation; without it, productions like
        // §3.3.1 [56] `TokenizedType ::= 'ID' | 'IDREF' | 'IDREFS' |
        // ...` would commit to a 2-char prefix and the surrounding
        // declaration would fail.
        let spec = "
            <prod id=\"NT-X\" num=\"99\">
                <lhs>X</lhs>
                <rhs>\"abc\" | \"abcdef\"</rhs>
            </prod>
        ";
        let grammar = load_grammar(spec).unwrap();
        let mut interp = Interpreter::new(&grammar, "abcdef");
        // The longer alternative wins even though it appears second.
        assert!(matches_completely(interp.match_production("X", 0), 6));
    }

    #[test]
    fn matches_alternation_leftmost_wins_on_tie() {
        // Tie-breaking convention (POSIX regex / lex(1)): equal-length
        // matches go to the leftmost (earlier-declared) branch.
        let spec = "
            <prod id=\"NT-X\" num=\"99\">
                <lhs>X</lhs>
                <rhs>\"ab\" | \"ab\"</rhs>
            </prod>
        ";
        let grammar = load_grammar(spec).unwrap();
        let mut interp = Interpreter::new(&grammar, "ab");
        assert!(matches_completely(interp.match_production("X", 0), 2));
    }

    #[test]
    fn matches_optional_and_kleene() {
        let spec = "
            <prod id=\"NT-A\" num=\"1\">
                <lhs>A</lhs>
                <rhs>\"a\"? \"b\"*</rhs>
            </prod>
        ";
        let grammar = load_grammar(spec).unwrap();
        // Per Ford 2002a §4.1, the cache is bound to one input;
        // construct a fresh Interpreter per input.
        let mut empty = Interpreter::new(&grammar, "");
        assert!(matches_completely(empty.match_production("A", 0), 0));
        let mut single_a = Interpreter::new(&grammar, "a");
        assert!(matches_completely(single_a.match_production("A", 0), 1));
        let mut abbb = Interpreter::new(&grammar, "abbb");
        assert!(matches_completely(abbb.match_production("A", 0), 4));
        let mut bbb = Interpreter::new(&grammar, "bbb");
        assert!(matches_completely(bbb.match_production("A", 0), 3));
    }

    #[test]
    fn matches_subtraction_for_comment_body_char() {
        // §2.5 Comment uses `Char - '-'` — a Char that is not a hyphen.
        let spec = "
            <prod id=\"NT-Char\" num=\"2\">
                <lhs>Char</lhs>
                <rhs>[#x20-#xD7FF]</rhs>
            </prod>
            <prod id=\"NT-CharNotHyphen\" num=\"99\">
                <lhs>CharNotHyphen</lhs>
                <rhs><nt def=\"NT-Char\">Char</nt> - \"-\"</rhs>
            </prod>
        ";
        let grammar = load_grammar(spec).unwrap();
        let mut a = Interpreter::new(&grammar, "a");
        assert!(matches_completely(
            a.match_production("CharNotHyphen", 0),
            1
        ));
        // '-' is a Char but is subtracted out.
        let mut hyphen = Interpreter::new(&grammar, "-");
        assert!(matches!(
            hyphen.match_production("CharNotHyphen", 0),
            MatchResult::NoMatch
        ));
    }

    #[test]
    fn match_production_caches_result_across_calls() {
        let spec = "
            <prod id=\"NT-X\" num=\"1\">
                <lhs>X</lhs>
                <rhs>\"hello\"</rhs>
            </prod>
        ";
        let grammar = load_grammar(spec).unwrap();
        let mut interp = Interpreter::new(&grammar, "hello world");
        let r1 = interp.match_production("X", 0);
        let r2 = interp.match_production("X", 0);
        assert_eq!(r1, r2);
        // Cache entry is populated.
        assert!(interp.cache.contains_key(&("X".to_string(), 0)));
    }
}
