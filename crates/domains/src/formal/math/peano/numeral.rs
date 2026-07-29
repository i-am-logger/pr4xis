//! The Numeral -> `calculator::Value` bridge (turing-benchmark A3, second
//! half): `english/function_word_store.rs`'s `NumeralRecord` carries only a
//! word's TEXT, with no path from that text to a computable value -- there
//! is a calculator evaluator (`formal::calculator::op::BinaryOp`) and there
//! are numeral tokens, but nothing before this file connected them.
//!
//! Scope, stated plainly: this is the closed-class BASIC numeral inventory
//! (Hurford 1975 ch. 2's simple lexical numerals: the units, the teens, and
//! the decade/hundred/thousand MULTIPLIER bases) as a small, hand-authored,
//! fully-cited table -- the same status as this crate's other closed
//! grammatical enumerations (e.g. `lambek::types::SentenceFeature`,
//! `lexicon::pos::InterjectionKind`'s kind taxonomy): a fixed linguistic
//! category, not empirical/corpus data, so there is no external source to
//! register in `praxis.toml` and load at runtime -- checked directly: the
//! loaded `function-words/english.xml` corpus has no numeral-value entries
//! to source from (only a POS legend mentioning "num"), and no CLDR/RBNF or
//! comparable numbering-data source is registered anywhere in this
//! workspace. Earlier drafts of this file called it a "loaded lookup
//! table", which overclaimed -- correcting that here rather than repeating
//! it. Hurford's base/multiplier COMPOSITION rules ("twenty-one" = 20+1,
//! "two hundred" = 2\u{00d7}100) are a real, larger, separately-scoped
//! grammar-integration task (the same class of work as Slice B's new
//! grammar frame) -- this file supplies the base vocabulary that
//! composition would consume, not the composition itself (tracked as a
//! follow-up, not a silent gap: see the crate's `.notes`/plan tracking for
//! this turing-benchmark thread).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::calculator::value::Value;

/// One entry in Hurford's (1975) basic numeral inventory: a written form
/// paired with the integer it lexicalizes.
struct NumeralEntry {
    word: &'static str,
    value: i64,
}

/// Hurford (1975) *The Linguistic Theory of Numerals*, Cambridge University
/// Press, ch. 2: the SIMPLE lexical numerals of English -- units 0-12 (each
/// its own root), the teens 13-19 (a "-teen" suffixed form, still simple
/// lexical items in Hurford's sense), and the decade/hundred/thousand
/// MULTIPLIER bases that a composed numeral phrase would scale
/// ("twenty" \u{00d7} 1, "twenty" + "one", etc. -- composition itself is out of
/// scope here).
const BASIC_NUMERALS: &[NumeralEntry] = &[
    NumeralEntry {
        word: "zero",
        value: 0,
    },
    NumeralEntry {
        word: "one",
        value: 1,
    },
    NumeralEntry {
        word: "two",
        value: 2,
    },
    NumeralEntry {
        word: "three",
        value: 3,
    },
    NumeralEntry {
        word: "four",
        value: 4,
    },
    NumeralEntry {
        word: "five",
        value: 5,
    },
    NumeralEntry {
        word: "six",
        value: 6,
    },
    NumeralEntry {
        word: "seven",
        value: 7,
    },
    NumeralEntry {
        word: "eight",
        value: 8,
    },
    NumeralEntry {
        word: "nine",
        value: 9,
    },
    NumeralEntry {
        word: "ten",
        value: 10,
    },
    NumeralEntry {
        word: "eleven",
        value: 11,
    },
    NumeralEntry {
        word: "twelve",
        value: 12,
    },
    NumeralEntry {
        word: "thirteen",
        value: 13,
    },
    NumeralEntry {
        word: "fourteen",
        value: 14,
    },
    NumeralEntry {
        word: "fifteen",
        value: 15,
    },
    NumeralEntry {
        word: "sixteen",
        value: 16,
    },
    NumeralEntry {
        word: "seventeen",
        value: 17,
    },
    NumeralEntry {
        word: "eighteen",
        value: 18,
    },
    NumeralEntry {
        word: "nineteen",
        value: 19,
    },
    NumeralEntry {
        word: "twenty",
        value: 20,
    },
    NumeralEntry {
        word: "thirty",
        value: 30,
    },
    NumeralEntry {
        word: "forty",
        value: 40,
    },
    NumeralEntry {
        word: "fifty",
        value: 50,
    },
    NumeralEntry {
        word: "sixty",
        value: 60,
    },
    NumeralEntry {
        word: "seventy",
        value: 70,
    },
    NumeralEntry {
        word: "eighty",
        value: 80,
    },
    NumeralEntry {
        word: "ninety",
        value: 90,
    },
    NumeralEntry {
        word: "hundred",
        value: 100,
    },
    NumeralEntry {
        word: "thousand",
        value: 1000,
    },
];

/// The Numeral -> `Value` functor proper: a written numeral form's
/// `calculator::Value`, read from the single canonical `BASIC_NUMERALS`
/// table -- never an inline `if word == "three"` at a call site. `None` for
/// any surface outside Hurford's basic inventory (a composed numeral
/// phrase, or a non-numeral word).
pub fn value_of_numeral_word(word: &str) -> Option<Value> {
    BASIC_NUMERALS
        .iter()
        .find(|entry| entry.word == word)
        .map(|entry| Value::int(entry.value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::calculator::op::BinaryOp;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn basic_numerals_resolve_to_their_value() {
        assert_eq!(value_of_numeral_word("zero"), Some(Value::int(0)));
        assert_eq!(value_of_numeral_word("seven"), Some(Value::int(7)));
        assert_eq!(value_of_numeral_word("twelve"), Some(Value::int(12)));
        assert_eq!(value_of_numeral_word("thirteen"), Some(Value::int(13)));
        assert_eq!(value_of_numeral_word("ninety"), Some(Value::int(90)));
        assert_eq!(value_of_numeral_word("hundred"), Some(Value::int(100)));
        assert_eq!(value_of_numeral_word("thousand"), Some(Value::int(1000)));
    }

    /// Every entry in the loaded table, not a hand-picked sample -- the
    /// spot-check above exercises the happy path; this iterates the actual
    /// `BASIC_NUMERALS` inventory (all 30 entries: units 0-12, teens 13-19,
    /// and the decade/hundred/thousand multiplier bases) and asserts each
    /// one resolves to precisely the value it declares, so no entry can
    /// silently regress without a spot-check happening to cover it.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_basic_numeral_resolves_to_its_declared_value() {
        for entry in BASIC_NUMERALS {
            assert_eq!(
                value_of_numeral_word(entry.word),
                Some(Value::int(entry.value)),
                "numeral {:?} did not resolve to its declared value {}",
                entry.word,
                entry.value,
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_composed_numeral_phrase_is_honestly_unresolved() {
        // "twenty-one" is a COMPOSED phrase (Hurford's base/multiplier
        // combination), not a simple lexical numeral -- out of scope, and
        // this function says so by returning None rather than guessing.
        assert_eq!(value_of_numeral_word("twenty-one"), None);
        assert_eq!(value_of_numeral_word("dog"), None);
    }

    /// The A3 generated test (turing-benchmark spec): every `BinaryOp`
    /// (including `Modulo`) applied to a sampled pair of resolved numeral
    /// values agrees with the DIRECTLY computed i64/f64 ground truth --
    /// "parse -> eval == predicted value" over the whole operator set, not
    /// just addition. The WAIS-IV Arithmetic subtest's single-digit operand
    /// convention bounds the sample (0-9), matching the same cited range
    /// `peano::ontology`'s computational axioms use.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_binary_op_agrees_with_ground_truth_over_resolved_numerals() {
        const WORDS: [&str; 10] = [
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];
        for (a_word, a) in WORDS.iter().zip(0i64..) {
            for (b_word, b) in WORDS.iter().zip(0i64..) {
                let av = value_of_numeral_word(a_word).expect("basic numeral resolves");
                let bv = value_of_numeral_word(b_word).expect("basic numeral resolves");

                for op in [
                    BinaryOp::Add,
                    BinaryOp::Subtract,
                    BinaryOp::Multiply,
                    BinaryOp::Divide,
                    BinaryOp::Power,
                    BinaryOp::Modulo,
                ] {
                    let result = op.apply(&av, &bv);
                    match op {
                        BinaryOp::Add => assert_eq!(result, Ok(Value::int(a + b))),
                        BinaryOp::Subtract => assert_eq!(result, Ok(Value::int(a - b))),
                        BinaryOp::Multiply => assert_eq!(result, Ok(Value::int(a * b))),
                        BinaryOp::Divide => {
                            if b == 0 {
                                assert!(result.is_err());
                            } else {
                                assert_eq!(result, Value::rational(a, b));
                            }
                        }
                        BinaryOp::Power => {
                            let Ok(Value::Float(got)) = result else {
                                panic!("Power must produce a Float, got {result:?}");
                            };
                            assert!((got - (a as f64).powf(b as f64)).abs() < 1e-9);
                        }
                        BinaryOp::Modulo => {
                            if b == 0 {
                                assert!(result.is_err());
                            } else {
                                let Ok(Value::Float(got)) = result else {
                                    panic!("Modulo must produce a Float, got {result:?}");
                                };
                                assert!((got - (a as f64 % b as f64)).abs() < 1e-9);
                            }
                        }
                    }
                }
            }
        }
    }
}
