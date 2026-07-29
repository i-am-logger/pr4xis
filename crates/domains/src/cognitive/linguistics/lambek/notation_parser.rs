//! CCG category notation → [`LambekType`] — the inverse of
//! [`LambekType::notation`](super::types::LambekType::notation).
//!
//! This is the ONE irreducible new primitive behind "lexical categories are
//! loaded data, not a Rust `match`": a loaded projection carries a word-class's
//! category as its standard CCGbank notation STRING (e.g. `S[wq]/(S[q]/PP)`),
//! and this parser turns that wire string into a structured [`LambekType`] at
//! the codec boundary — exactly as
//! [`operators::decode_signature`](super::operators) turns an OpenMath signature
//! string into a typed value. Without it a loaded category string can never
//! become a type, and the projection would collapse back into a hardcoded
//! `match`.
//!
//! It is the exact inverse of `notation()`, so the law `parse(t.notation()) ==
//! Some(t)` holds for every [`LambekType`] (`round_trips_every_svo_category`).
//!
//! # Grammar (the shape `notation()` emits)
//!
//! ```text
//! expr    := primary ( ('/' primary)* | ('\' primary)* )
//! primary := atom | '(' expr ')'
//! atom    := 'NP' | 'N' | 'PP' | 'S' ('[' feature ']')?
//! ```
//!
//! `/` is left-associative (`X/Y/Z` = `(X/Y)/Z`) and `\` is right-associative
//! (`X\Y\Z` = `X\(Y\Z)`); `notation()` parenthesizes every mixed nesting, so a
//! single level is always a homogeneous operator chain. Malformed input (a mixed
//! unparenthesized chain, an unknown atom/feature, unbalanced parens, trailing
//! junk) fails closed (`None`).

#[allow(unused_imports)]
use alloc::{boxed::Box, vec, vec::Vec};

use super::types::{AtomicType, LambekType, SentenceFeature};

/// Parse a CCG category in standard notation into a [`LambekType`], or `None`
/// (fails closed) if the string is not a well-formed category.
pub fn parse_category(input: &str) -> Option<LambekType> {
    let mut c = Cursor {
        b: input.as_bytes(),
        i: 0,
    };
    let t = c.expr()?;
    c.skip_ws();
    if c.i == c.b.len() { Some(t) } else { None }
}

struct Cursor<'a> {
    b: &'a [u8],
    i: usize,
}

impl Cursor<'_> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn bump(&mut self) {
        self.i += 1;
    }

    fn skip_ws(&mut self) {
        while self.peek() == Some(b' ') {
            self.bump();
        }
    }

    /// `expr := primary ( ('/' primary)* | ('\' primary)* )` — a homogeneous
    /// operator chain (mixed nestings are always parenthesized by `notation()`).
    fn expr(&mut self) -> Option<LambekType> {
        let first = self.primary()?;
        self.skip_ws();
        match self.peek() {
            // Left-associative right-division: X/Y/Z = (X/Y)/Z.
            Some(b'/') => {
                let mut acc = first;
                while {
                    self.skip_ws();
                    self.peek() == Some(b'/')
                } {
                    self.bump();
                    let rhs = self.primary()?;
                    acc = LambekType::right_div(acc, rhs);
                }
                Some(acc)
            }
            // Right-associative left-division: X\Y\Z = X\(Y\Z).
            Some(b'\\') => {
                let mut items = vec![first];
                while {
                    self.skip_ws();
                    self.peek() == Some(b'\\')
                } {
                    self.bump();
                    items.push(self.primary()?);
                }
                // Right fold: [X, Y, Z] → X\(Y\Z). `items` is non-empty (it
                // starts with `first`), so the first pop always yields a value.
                let mut acc = items.pop()?;
                while let Some(head) = items.pop() {
                    acc = LambekType::left_div(head, acc);
                }
                Some(acc)
            }
            _ => Some(first),
        }
    }

    /// `primary := atom | '(' expr ')'`.
    fn primary(&mut self) -> Option<LambekType> {
        self.skip_ws();
        if self.peek() == Some(b'(') {
            self.bump();
            let t = self.expr()?;
            self.skip_ws();
            if self.peek() != Some(b')') {
                return None;
            }
            self.bump();
            Some(t)
        } else {
            self.atom()
        }
    }

    /// `atom := 'NP' | 'N' | 'PP' | 'S' ('[' feature ']')?`.
    fn atom(&mut self) -> Option<LambekType> {
        self.skip_ws();
        match self.peek()? {
            b'N' => {
                self.bump();
                if self.peek() == Some(b'P') {
                    self.bump();
                    Some(LambekType::np())
                } else {
                    Some(LambekType::n())
                }
            }
            b'P' => {
                self.bump();
                if self.peek() == Some(b'P') {
                    self.bump();
                    Some(LambekType::pp())
                } else {
                    None
                }
            }
            b'S' => {
                self.bump();
                if self.peek() == Some(b'[') {
                    self.bump();
                    let feature = self.feature()?;
                    Some(LambekType::atom(AtomicType::S(Some(feature))))
                } else {
                    Some(LambekType::atom(AtomicType::S(None)))
                }
            }
            _ => None,
        }
    }

    /// The `feature` inside `S[...]`, up to and consuming the `]`.
    fn feature(&mut self) -> Option<SentenceFeature> {
        let start = self.i;
        while self.peek().is_some() && self.peek() != Some(b']') {
            self.bump();
        }
        if self.peek() != Some(b']') {
            return None;
        }
        let frag = &self.b[start..self.i];
        self.bump(); // consume ']'
        // Inverse of notation()'s feature render (types.rs:168-178).
        Some(match frag {
            b"dcl" => SentenceFeature::Dcl,
            b"adj" => SentenceFeature::Adj,
            b"q" => SentenceFeature::Q,
            b"wq" => SentenceFeature::Wq,
            b"b" => SentenceFeature::Bare,
            b"ng" => SentenceFeature::Ng,
            b"pss" => SentenceFeature::Pss,
            b"pt" => SentenceFeature::Pt,
            b"to" => SentenceFeature::To,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::types::svo;

    /// THE LAW: the parser is the exact inverse of `notation()` — for every
    /// category the grammar uses, `parse(t.notation()) == Some(t)`. This is what
    /// makes "categories carried as notation strings" honest loaded data rather
    /// than a relocated match.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn round_trips_every_svo_category() {
        let categories = [
            svo::determiner(),
            svo::noun(),
            svo::proper_noun(),
            svo::intransitive_verb(),
            svo::transitive_verb(),
            svo::ditransitive_verb(),
            svo::adjective(),
            svo::preposition(),
            svo::adverb(),
            svo::predicate_adjective(),
            svo::copula(),
            svo::copula_adj(),
            svo::question_copula(),
            svo::wh_what(),
            svo::wh_determiner(),
            svo::wh_adverb(),
            svo::question_copula_pp(),
            svo::bare_transitive_verb(),
            svo::wh_what_object(),
            svo::does_support(),
            svo::infinitive_to(),
            svo::catenative_infinitival_predicate(),
        ];
        for cat in categories {
            let rendered = cat.notation();
            assert_eq!(
                parse_category(&rendered),
                Some(cat.clone()),
                "round-trip failed for {rendered}"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parses_the_interrogative_categories_verbatim() {
        assert_eq!(parse_category("S[wq]/(NP\\S)"), Some(svo::wh_what()));
        assert_eq!(
            parse_category("S[wq]/(NP\\S)/N"),
            Some(svo::wh_determiner())
        );
        assert_eq!(parse_category("S[wq]/(S[q]/PP)"), Some(svo::wh_adverb()));
        assert_eq!(
            parse_category("(S[q]/PP)/NP"),
            Some(svo::question_copula_pp())
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn atoms_and_features() {
        assert_eq!(parse_category("S"), Some(LambekType::s()));
        assert_eq!(parse_category("NP"), Some(LambekType::np()));
        assert_eq!(parse_category("N"), Some(LambekType::n()));
        assert_eq!(parse_category("PP"), Some(LambekType::pp()));
        assert_eq!(parse_category("S[dcl]"), Some(LambekType::s_dcl()));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn associativity_matches_notation() {
        // `/` left-associative, `\` right-associative — same shape notation emits.
        assert_eq!(
            parse_category("N/N/N").map(|t| t.notation()),
            Some("N/N/N".into())
        );
        assert_eq!(
            parse_category("N\\N\\N").map(|t| t.notation()),
            Some("N\\N\\N".into())
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn malformed_input_fails_closed() {
        for bad in ["", "X", "S[bogus]", "NP/", "(NP\\S", "NP S", "S[wq]/NP)"] {
            assert_eq!(parse_category(bad), None, "{bad:?} must fail closed");
        }
    }
}
