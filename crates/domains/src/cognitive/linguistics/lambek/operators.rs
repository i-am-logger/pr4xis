//! Mathematical operators as a LOADED vocabulary — the praxis-way fix for
//! #169 (the tokenizer dropped `+`, `<`, … as punctuation).
//!
//! The operator set is NOT hand-enumerated in Rust. Each glyph, its OpenMath
//! symbol identity, and its STS signature (role, arity, result sort) load from
//! the bundled `data/operators/math-operators.xml` (OpenMath `arith1` +
//! `relation1` Content Dictionaries, glyphs per ISO 80000-2) through praxis'
//! own LMF reader — the same reader that loads function-words and WordNet.
//! Adding an operator (`^` / `power`, `≤` / `leq`, …) is one data line with
//! zero new Rust.
//!
//! # The category is LOADED, never a Rust-computed schema
//!
//! Each operator's CCG category is loaded as notation from
//! `math-operators.xml` and lowered to a [`LambekType`] by the
//! [`notation parser`](super::notation_parser) — the same loaded-data path as
//! the wh / POS categories, not a Rust `match`. An infix binary/n-ary operator
//! is the **coordination schema** `(X\X)/X` (Steedman 2000; Partee & Rooth
//! 1983) — it coordinates two like-typed operands into one; a relation is
//! `(NP\S)/NP`; a prefix unary is `NP/NP`. Reduction is forward then backward
//! function application — `A/B + B → A`, `A + A\B → B` — i.e. Lambek (1958)
//! residuation (Moortgat 1997).
//!
//! The operand atom is [`operand_atom`] (NP): a saturated numeric literal is a
//! referring expression — a noun *phrase*, like a proper noun or pronoun
//! (Hockenmaier & Steedman 2007, CCGbank) — so it composes both with an
//! operator's NP argument and with a copula's NP complement (`what is 10 + 10`
//! → `S[wq]`).
//!
//! Co-authored-by: awfmilton — the #169 bug report + research framed this.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use super::types::LambekType;
use crate::social::software::markup::xml::lmf::reader::read_wordnet;

/// The OpenMath Role of an operator symbol (OpenMath Standard 2.0 §2.1.4).
/// Every operator in `arith1`/`relation1` is an `application` symbol; the type
/// `Role` keeps the loaded fact honest and fails closed on any other role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorRole {
    /// `application` — a symbol applied to operands (the only role this
    /// vocabulary loads; `binder` / `attribution` / `error` are out of scope).
    Application,
}

/// The operand count of an operator — its OpenMath STS arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    /// One operand, prefixed: `unary_minus` (`-5`).
    Unary,
    /// Exactly two operands: `divide`, every `relation1` operator.
    Binary,
    /// Two-or-more, associative: `plus`, `times`.
    Nary,
}

/// The OpenMath result sort of an operator — the categorial codomain.
///
/// This is a CLOSED discriminator by axiom, not a vocabulary: a categorial
/// reduction lands in exactly one of two kinds of thing — a *term* (a number,
/// realized as the nominal atom NP) or a *proposition* (a truth value, realized
/// as the sentence atom S). OpenMath's own sort system bottoms out the same way
/// (`arith1` symbols are sort `number`; `relation1` symbols are sort `truth`).
/// Two variants is the whole space, so — unlike the rejected `interrogative:
/// bool` flag — it is not a partial encoding of an open set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultSort {
    /// A number — realized as the nominal atom NP (`10 + 10` is a number).
    Number,
    /// A truth value — realized as the sentence atom S (`10 < 20` is a claim).
    Truth,
}

/// One loaded operator: a glyph bound to an OpenMath symbol, its STS signature,
/// and its loaded CCG category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedOperator {
    /// The surface glyph (e.g. `+`, `<`, `×`).
    pub glyph: char,
    /// The OpenMath symbol identity, `cd#name` (e.g. `arith1#plus`).
    pub openmath_symbol: String,
    /// The OpenMath Role — `application` for all loaded operators.
    pub role: OperatorRole,
    /// The operand count (STS arity).
    pub arity: Arity,
    /// The result sort — the categorial codomain.
    pub result_sort: ResultSort,
    /// The CCG category, LOADED as notation from `math-operators.xml` and parsed
    /// by [`notation_parser`](super::notation_parser) — the category is DATA,
    /// not a Rust-computed schema. The infix coordination shape `(X\X)/X`
    /// (number) / relation shape `(NP\S)/NP` (truth) / prefix `NP/NP` (unary)
    /// per CCGbank (Steedman's coordination schema; Lambek 1958).
    pub category: LambekType,
}

impl LoadedOperator {
    /// This operator's loaded CCG category.
    pub fn lambek_type(&self) -> LambekType {
        self.category.clone()
    }
}

/// The categorial atom of a saturated numeric operand: NP.
///
/// A number literal is a referring expression — a noun *phrase*, not a bare
/// common noun N (CCGbank types proper nouns / pronouns / bare referring
/// numerals as NP; Hockenmaier & Steedman 2007). This single helper is shared
/// by the number-literal recognizer AND the loaded operator categories, so
/// operand and operator-argument atoms agree by construction rather than by a
/// cross-site convention.
pub fn operand_atom() -> LambekType {
    LambekType::np()
}

/// A loaded mathematical-operator vocabulary — glyph → its operator readings
/// (a glyph can have several: `-` is both binary subtraction and unary
/// negation). Built by [`load`].
#[derive(Debug, Clone, Default)]
pub struct OperatorVocabulary(BTreeMap<char, Vec<LoadedOperator>>);

impl OperatorVocabulary {
    /// Is `c` a loaded operator glyph? (A data query, NOT `is_ascii_punctuation`
    /// — `?`/`.` are punctuation but not operators, so they still trim.)
    pub fn is_operator_glyph(&self, c: char) -> bool {
        self.0.contains_key(&c)
    }

    /// Every categorial type a single-glyph word carries (e.g. `-` → binary
    /// subtraction `(NP\NP)/NP` AND unary negation `NP/NP`). Empty if `word` is
    /// not a loaded operator glyph; the chart parser explores all readings.
    pub fn lambek_types_for(&self, word: &str) -> Vec<LambekType> {
        single_glyph(word)
            .and_then(|c| self.0.get(&c))
            .map(|ops| ops.iter().map(LoadedOperator::lambek_type).collect())
            .unwrap_or_default()
    }

    /// The primary (first-loaded) categorial type, if `word` is a loaded
    /// operator glyph.
    pub fn primary_type(&self, word: &str) -> Option<LambekType> {
        single_glyph(word)
            .and_then(|c| self.0.get(&c))
            .and_then(|ops| ops.first())
            .map(LoadedOperator::lambek_type)
    }

    /// The loaded readings for a glyph (test/inspection access).
    pub fn operators_for(&self, glyph: char) -> &[LoadedOperator] {
        self.0.get(&glyph).map_or(&[], Vec::as_slice)
    }

    /// Every loaded operator reading, across all glyphs.
    pub fn iter(&self) -> impl Iterator<Item = &LoadedOperator> {
        self.0.values().flatten()
    }

    /// Total number of loaded operator readings (entries, counting each glyph
    /// reading once).
    pub fn len(&self) -> usize {
        self.0.values().map(Vec::len).sum()
    }

    /// True iff no operator loaded.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The single-glyph char of `word`, or `None` if it is empty or multi-char
/// (loaded operators are single glyphs).
fn single_glyph(word: &str) -> Option<char> {
    let mut chars = word.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(c)
}

/// Load the bundled math-operator vocabulary from `data/operators/
/// math-operators.xml` through praxis' LMF reader.
///
/// Returns the parsed vocabulary BY VALUE (the function-words idiom): no global
/// `OnceLock`, so this stays `no_std`-clean and reads from the one XML source.
/// The bundle ships with praxis, so a parse / decode failure is a build-time
/// invariant and panics rather than silently degrading.
pub fn load() -> OperatorVocabulary {
    const XML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/operators/math-operators.xml"
    ));
    let wn = read_wordnet(XML).expect(
        "bundled crates/domains/data/operators/math-operators.xml failed to \
         parse — build-time invariant violated",
    );

    let mut map: BTreeMap<char, Vec<LoadedOperator>> = BTreeMap::new();
    for entry in &wn.entries {
        let glyph = single_glyph(&entry.lemma.written_form)
            .expect("operator writtenForm is a single glyph — build-time invariant");
        let sense = entry
            .senses
            .first()
            .expect("operator LexicalEntry carries a Sense — build-time invariant");
        let (role, arity, result_sort, category) = decode_signature(&sense.subcat).expect(
            "operator Sense subcat encodes 'ROLE ARITY RESULTSORT NOTATION' — build-time invariant",
        );
        map.entry(glyph).or_default().push(LoadedOperator {
            glyph,
            openmath_symbol: sense.synset.clone(),
            role,
            arity,
            result_sort,
            category,
        });
    }
    OperatorVocabulary(map)
}

/// Decode the OpenMath STS signature carried in a Sense `subcat`
/// (`"ROLE ARITY RESULTSORT NOTATION"`) once, at the codec boundary, into typed
/// values. The 4th token is the CCG category in standard notation, lowered to a
/// [`LambekType`] by the notation parser — so the category is LOADED data, not
/// a Rust-computed schema. `None` on any unknown/unparseable token (fails
/// closed).
fn decode_signature(subcat: &[String]) -> Option<(OperatorRole, Arity, ResultSort, LambekType)> {
    let role = match subcat.first()?.as_str() {
        "application" => OperatorRole::Application,
        _ => return None,
    };
    let arity = match subcat.get(1)?.as_str() {
        "unary" => Arity::Unary,
        "binary" => Arity::Binary,
        "nary" => Arity::Nary,
        _ => return None,
    };
    let result_sort = match subcat.get(2)?.as_str() {
        "number" => ResultSort::Number,
        "truth" => ResultSort::Truth,
        _ => return None,
    };
    let category = super::notation_parser::parse_category(subcat.get(3)?)?;
    Some((role, arity, result_sort, category))
}

/// True iff `word` is a decimal number literal — a non-empty run of ASCII
/// digits with at most one decimal point (ISO 80000-2 §3 decimal numbers;
/// Unicode general category Nd). Such a literal types as [`operand_atom`].
pub fn is_number_literal(word: &str) -> bool {
    !word.is_empty()
        && word.chars().all(|c| c.is_ascii_digit() || c == '.')
        && word.chars().any(|c| c.is_ascii_digit())
        && word.chars().filter(|&c| c == '.').count() <= 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::types::reduce;

    // ── Loader invariants ─────────────────────────────────────────────

    #[test]
    fn vocabulary_loads_the_operator_set() {
        let v = load();
        assert!(
            !v.is_empty(),
            "math-operators.xml must load a non-empty set"
        );
        // Every arithmetic + relational glyph the bundle declares is present.
        for g in [
            '+', '-', '*', '×', '/', '÷', '−', '=', '<', '>', '≤', '≥', '≠',
        ] {
            assert!(v.is_operator_glyph(g), "missing operator glyph {g:?}");
        }
        // Non-operators are NOT operators (the tokenizer must still trim them).
        for g in ['?', '.', 'a', '1'] {
            assert!(!v.is_operator_glyph(g), "{g:?} is not an operator");
        }
    }

    #[test]
    fn plus_loads_as_arith1_nary_number() {
        let v = load();
        let ops = v.operators_for('+');
        assert_eq!(ops.len(), 1, "+ has one reading");
        let plus = &ops[0];
        assert_eq!(plus.openmath_symbol, "arith1#plus");
        assert_eq!(plus.role, OperatorRole::Application);
        assert_eq!(plus.arity, Arity::Nary);
        assert_eq!(plus.result_sort, ResultSort::Number);
    }

    #[test]
    fn minus_loads_both_binary_and_unary_readings() {
        // `-` is BOTH binary subtraction and unary negation — two loaded
        // readings, so the chart can pick the one that reduces.
        let v = load();
        let ops = v.operators_for('-');
        assert_eq!(ops.len(), 2, "- has binary + unary readings");
        let arities: Vec<Arity> = ops.iter().map(|o| o.arity).collect();
        assert!(arities.contains(&Arity::Binary));
        assert!(arities.contains(&Arity::Unary));
    }

    // ── The type is derived AND reduces (in repo constructors) ─────────

    #[test]
    fn arithmetic_infix_is_the_coordination_schema_and_reduces_to_np() {
        // The LOADED category for '+' (n-ary number) is (NP\NP)/NP — Steedman's
        // (X\X)/X — parsed from math-operators.xml, not Rust-computed.
        let plus = load()
            .operators_for('+')
            .first()
            .expect("+ loads")
            .lambek_type();
        assert_eq!(plus.notation(), "(NP\\NP)/NP");
        // 10 + 10 : NP (NP\NP)/NP NP → forward then backward application → NP.
        let after_forward = reduce(&plus, &operand_atom()).expect("forward: (NP\\NP)/NP + NP");
        assert_eq!(after_forward.notation(), "NP\\NP");
        let result = reduce(&operand_atom(), &after_forward).expect("backward: NP + NP\\NP");
        assert_eq!(
            result,
            operand_atom(),
            "10 + 10 is a noun phrase (a number)"
        );
    }

    #[test]
    fn relation_infix_reduces_to_a_sentence() {
        // The LOADED category for '<' (binary truth) is (NP\S)/NP — a proposition.
        let lt = load()
            .operators_for('<')
            .first()
            .expect("< loads")
            .lambek_type();
        assert_eq!(lt.notation(), "(NP\\S)/NP");
        let after_forward = reduce(&lt, &operand_atom()).expect("forward: (NP\\S)/NP + NP");
        assert_eq!(after_forward.notation(), "NP\\S");
        let result = reduce(&operand_atom(), &after_forward).expect("backward: NP + NP\\S");
        assert!(result.is_sentence(), "10 < 20 is a claim (a sentence)");
    }

    #[test]
    fn every_loaded_operator_reduces_to_its_result_sort() {
        // Coverage over the WHOLE bundle (not just +, <): each operator's
        // DERIVED type must actually reduce with NP operands to the sort its
        // loaded result_sort claims — Number → NP, Truth → S. This is what
        // proves ×, ÷, −, ≤, ≥, ≠ work, not merely that they load.
        let v = load();
        assert!(v.len() >= 13, "the full arithmetic + relational set loads");
        for op in v.iter() {
            let expected = match op.result_sort {
                ResultSort::Number => operand_atom(),
                ResultSort::Truth => LambekType::s(),
            };
            let ty = op.lambek_type();
            let reduced = match op.arity {
                // op operand → forward application.
                Arity::Unary => reduce(&ty, &operand_atom()).unwrap_or_else(|| {
                    panic!("{} ({}) unary did not reduce", op.glyph, op.openmath_symbol)
                }),
                // operand op operand → forward then backward application.
                Arity::Binary | Arity::Nary => {
                    let forward = reduce(&ty, &operand_atom()).unwrap_or_else(|| {
                        panic!(
                            "{} ({}) forward did not reduce",
                            op.glyph, op.openmath_symbol
                        )
                    });
                    reduce(&operand_atom(), &forward).unwrap_or_else(|| {
                        panic!(
                            "{} ({}) backward did not reduce",
                            op.glyph, op.openmath_symbol
                        )
                    })
                }
            };
            assert_eq!(
                reduced, expected,
                "{} ({}) reduced to the wrong sort",
                op.glyph, op.openmath_symbol
            );
        }
    }

    #[test]
    fn unary_prefix_takes_one_operand() {
        // The LOADED unary reading of '-' is NP/NP — prefix unary minus.
        let neg = load()
            .operators_for('-')
            .iter()
            .find(|o| o.arity == Arity::Unary)
            .expect("- has a unary reading")
            .lambek_type();
        assert_eq!(neg.notation(), "NP/NP");
        let result = reduce(&neg, &operand_atom()).expect("forward: NP/NP + NP");
        assert_eq!(result, operand_atom(), "-5 is a number");
    }

    // ── Number-literal recognizer ──────────────────────────────────────

    #[test]
    fn number_literals_are_recognized() {
        for s in ["10", "0", "3", "20", "3.14", "100"] {
            assert!(is_number_literal(s), "{s:?} is a number literal");
        }
        for s in ["", ".", "1.2.3", "10a", "x", "+", "-"] {
            assert!(!is_number_literal(s), "{s:?} is not a number literal");
        }
    }
}
