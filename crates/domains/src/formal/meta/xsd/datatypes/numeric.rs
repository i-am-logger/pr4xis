//! Lexical / value / canonical mappings for the boolean, decimal, and
//! integer-family XSD datatypes (W3C XML Schema 1.1 Part 2).
//!
//! Part 2 defines, for each datatype, three coupled artefacts:
//!
//! - a **·lexical space·** — the set of admissible literals (§3.3.x.1
//!   / §3.4.x.1 grammars);
//! - a **·value space·** — the abstract values (here modelled by Rust
//!   types: [`bool`] for boolean, [`DecimalValue`] for decimal and the
//!   integer family);
//! - a **·canonical mapping·** value → literal — total and injective,
//!   the inverse-section of the (many-to-one) ·lexical mapping·.
//!
//! ## Arbitrary precision without a bignum
//!
//! `xs:decimal` and `xs:integer` have *unbounded* precision (§3.3.3,
//! §3.4.13). Rather than depend on a bignum crate, [`DecimalValue`]
//! stores the value as normalized digit strings (sign + integer
//! digits + fraction digits). Two literals map to the same
//! `DecimalValue` iff they denote the same decimal value, and the
//! canonical mapping is a pure string construction over the normal
//! form — exact at any magnitude. Bounded integer ranges (`xs:byte`
//! … `xs:unsignedLong`) are checked by digit-string magnitude
//! comparison, so no fixed-width overflow is possible.
//!
//! ## Canonical decimal in XSD 1.1
//!
//! XSD 1.1 §3.3.3.1 ·decimalCanonicalMap· renders an integer-valued
//! decimal *without* a decimal point — `·noDecimalPtCanonicalMap·`,
//! the same auxiliary `·integerCanonicalMap·` uses (§3.4.13.1). So the
//! canonical form of the decimal value `1.0` is `"1"`, not `"1.0"`
//! (the XSD 1.0 prose note's "decimal point required" was superseded
//! by the 1.1 algorithmic mapping). A non-integer decimal keeps its
//! point with trailing zeros stripped: `1.50` → `"1.5"`.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.2 boolean, §3.3.3 decimal,
//!   §3.4.13–§3.4.25 the integer family.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::cmp::Ordering;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::ontology::XsdDatatypeConcept;

// =============================================================================
// boolean — W3C XSD 1.1 Part 2 §3.3.2.
// =============================================================================

/// ·boolean lexical mapping· (§3.3.2.2): the lexical space is exactly
/// `{"true", "false", "1", "0"}`; `1`/`true` map to `true`,
/// `0`/`false` to `false`. Returns `None` for any other literal.
pub fn parse_boolean(lex: &str) -> Option<bool> {
    match lex {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// ·boolean canonical mapping· (§3.3.2.2): `true` → `"true"`,
/// `false` → `"false"`. The numeric literals `1`/`0` are valid lexical
/// reps but never canonical.
pub fn canonical_boolean(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

// =============================================================================
// decimal value space — W3C XSD 1.1 Part 2 §3.3.3.
// =============================================================================

/// A point in the `xs:decimal` value space (§3.3.3.1), stored as a
/// normal form so that value equality is structural:
///
/// - `int_digits` — the integer part, with no leading zeros except the
///   single digit `"0"` when the integer part is zero;
/// - `frac_digits` — the fraction part, with no trailing zeros (may be
///   empty, denoting an integer value);
/// - `negative` — the sign; always `false` for the value zero (no
///   negative zero in the decimal value space).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecimalValue {
    negative: bool,
    int_digits: String,
    frac_digits: String,
}

impl DecimalValue {
    /// Construct a normalized [`DecimalValue`] from raw parts: strip
    /// leading zeros from the integer part (keeping one), strip
    /// trailing zeros from the fraction, and collapse a zero value's
    /// sign (no negative zero).
    fn normalize(negative: bool, int: &str, frac: &str) -> Self {
        let trimmed_int = int.trim_start_matches('0');
        let int_digits = if trimmed_int.is_empty() {
            "0".to_string()
        } else {
            trimmed_int.to_string()
        };
        let frac_digits = frac.trim_end_matches('0').to_string();
        let is_zero = int_digits == "0" && frac_digits.is_empty();
        DecimalValue {
            negative: negative && !is_zero,
            int_digits,
            frac_digits,
        }
    }

    /// True if this value has no fractional part (is an integer).
    pub fn is_integer(&self) -> bool {
        self.frac_digits.is_empty()
    }

    /// True if this value is zero.
    pub fn is_zero(&self) -> bool {
        self.int_digits == "0" && self.frac_digits.is_empty()
    }

    /// The sign: `true` if strictly negative.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// ·decimalCanonicalMap· (§3.3.3.1). Integer-valued decimals are
    /// rendered without a decimal point (`·noDecimalPtCanonicalMap·`);
    /// non-integers carry their normalized fraction.
    pub fn canonical_decimal(&self) -> String {
        let sign = if self.negative { "-" } else { "" };
        if self.frac_digits.is_empty() {
            format!("{sign}{}", self.int_digits)
        } else {
            format!("{sign}{}.{}", self.int_digits, self.frac_digits)
        }
    }

    /// ·integerCanonicalMap· (§3.4.13.1): no decimal point, no leading
    /// zeros, no `+`. Returns `None` if this value has a fractional
    /// part (it is not an integer value).
    pub fn canonical_integer(&self) -> Option<String> {
        if !self.is_integer() {
            return None;
        }
        let sign = if self.negative { "-" } else { "" };
        Some(format!("{sign}{}", self.int_digits))
    }
}

/// Parse a `·decimalLexicalRep·` (§3.3.3.1):
/// `('+'|'-')? ( digit+ ('.' digit*)? | '.' digit+ )`. Returns the
/// normalized [`DecimalValue`], or `None` if `lex` is outside the
/// lexical space.
pub fn parse_decimal(lex: &str) -> Option<DecimalValue> {
    let (negative, rest) = split_sign(lex);
    if rest.is_empty() {
        return None;
    }
    let (int_part, frac_part) = match rest.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (rest, None),
    };
    // All parts must be ASCII digits.
    if !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    match frac_part {
        // No decimal point: integer part must be non-empty (digit+).
        None => {
            if int_part.is_empty() {
                return None;
            }
        }
        // With a decimal point: `digit+ '.' digit*` or `'.' digit+`.
        // At least one digit on some side, and the fraction must be
        // all digits.
        Some(frac) => {
            if !frac.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            if int_part.is_empty() && frac.is_empty() {
                return None; // bare "." is not a decimal
            }
        }
    }
    Some(DecimalValue::normalize(
        negative,
        int_part,
        frac_part.unwrap_or(""),
    ))
}

/// Parse an `·integerLexicalRep·` (§3.4.13.1): `('+'|'-')? digit+` —
/// no decimal point. Returns an integer-valued [`DecimalValue`], or
/// `None` if `lex` is outside the integer lexical space.
pub fn parse_integer(lex: &str) -> Option<DecimalValue> {
    let (negative, rest) = split_sign(lex);
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(DecimalValue::normalize(negative, rest, ""))
}

/// Split an optional leading `+`/`-` sign. Returns `(negative, rest)`.
fn split_sign(lex: &str) -> (bool, &str) {
    if let Some(rest) = lex.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = lex.strip_prefix('+') {
        (false, rest)
    } else {
        (false, lex)
    }
}

// =============================================================================
// Integer-family range checks — W3C XSD 1.1 Part 2 §3.4.14–§3.4.25.
// =============================================================================

/// Compare two non-negative integer magnitudes given as digit strings
/// with no leading zeros (the [`DecimalValue::normalize`] invariant):
/// the longer string is larger; equal lengths compare
/// lexicographically (ASCII digit order = numeric order).
fn cmp_magnitude(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// True if the integer-valued `v` lies in the value space of the
/// derived integer datatype `dt` (W3C XSD 1.1 Part 2 §3.4.13–§3.4.25).
/// `dt` must be an integer-family datatype; non-integer datatypes and
/// fractional `v` return `false`.
pub fn integer_in_range(dt: XsdDatatypeConcept, v: &DecimalValue) -> bool {
    use XsdDatatypeConcept as D;
    if !v.is_integer() {
        return false;
    }
    let neg = v.negative;
    let mag = v.int_digits.as_str();
    let is_zero = v.is_zero();
    match dt {
        // §3.4.13 integer — unbounded.
        D::Integer => true,
        // Sign-bounded, magnitude-unbounded (§3.4.14, .15, .20, .25).
        D::NonNegativeInteger => !neg, // ≥ 0 (zero already has neg = false)
        D::PositiveInteger => !neg && !is_zero, // ≥ 1
        D::NonPositiveInteger => neg || is_zero, // ≤ 0
        D::NegativeInteger => neg && !is_zero, // ≤ -1
        // Signed two's-complement bounds (§3.4.16–§3.4.19).
        D::Long => signed_bounded(neg, mag, "9223372036854775808", "9223372036854775807"),
        D::Int => signed_bounded(neg, mag, "2147483648", "2147483647"),
        D::Short => signed_bounded(neg, mag, "32768", "32767"),
        D::Byte => signed_bounded(neg, mag, "128", "127"),
        // Unsigned bounds (§3.4.21–§3.4.24).
        D::UnsignedLong => unsigned_bounded(neg, mag, is_zero, "18446744073709551615"),
        D::UnsignedInt => unsigned_bounded(neg, mag, is_zero, "4294967295"),
        D::UnsignedShort => unsigned_bounded(neg, mag, is_zero, "65535"),
        D::UnsignedByte => unsigned_bounded(neg, mag, is_zero, "255"),
        // Not an integer-family datatype.
        _ => false,
    }
}

/// A signed integer type admits `[-neg_max, pos_max]`. `neg_max` /
/// `pos_max` are the magnitude digit strings of the bounds (e.g. byte:
/// neg_max = "128", pos_max = "127").
fn signed_bounded(neg: bool, mag: &str, neg_max: &str, pos_max: &str) -> bool {
    if neg {
        cmp_magnitude(mag, neg_max) != Ordering::Greater
    } else {
        cmp_magnitude(mag, pos_max) != Ordering::Greater
    }
}

/// An unsigned integer type admits `[0, max]`: no negative non-zero
/// values, magnitude at most `max`.
fn unsigned_bounded(neg: bool, mag: &str, is_zero: bool, max: &str) -> bool {
    if neg && !is_zero {
        return false;
    }
    cmp_magnitude(mag, max) != Ordering::Greater
}

// =============================================================================
// Typed parse + canonical dispatch.
// =============================================================================

/// Parse `lex` in the lexical space of the numeric datatype `dt`,
/// applying the integer-family range checks. Returns the
/// [`DecimalValue`] (boolean is handled by [`parse_boolean`]), or
/// `None` if `lex` is outside `dt`'s lexical/value space. `dt` must be
/// `Decimal` or an integer-family datatype.
pub fn parse_typed_numeric(dt: XsdDatatypeConcept, lex: &str) -> Option<DecimalValue> {
    use XsdDatatypeConcept as D;
    match dt {
        D::Decimal => parse_decimal(lex),
        D::Integer
        | D::NonPositiveInteger
        | D::NegativeInteger
        | D::Long
        | D::Int
        | D::Short
        | D::Byte
        | D::NonNegativeInteger
        | D::UnsignedLong
        | D::UnsignedInt
        | D::UnsignedShort
        | D::UnsignedByte
        | D::PositiveInteger => parse_integer(lex).filter(|v| integer_in_range(dt, v)),
        _ => None,
    }
}

/// The canonical literal of `v` for the numeric datatype `dt`:
/// ·decimalCanonicalMap· for `Decimal`, ·integerCanonicalMap· for the
/// integer family. Returns `None` if `dt` is not a numeric datatype or
/// `v` is fractional under an integer datatype.
pub fn canonical_numeric(dt: XsdDatatypeConcept, v: &DecimalValue) -> Option<String> {
    use XsdDatatypeConcept as D;
    match dt {
        D::Decimal => Some(v.canonical_decimal()),
        D::Integer
        | D::NonPositiveInteger
        | D::NegativeInteger
        | D::Long
        | D::Int
        | D::Short
        | D::Byte
        | D::NonNegativeInteger
        | D::UnsignedLong
        | D::UnsignedInt
        | D::UnsignedShort
        | D::UnsignedByte
        | D::PositiveInteger => v.canonical_integer(),
        _ => None,
    }
}

// =============================================================================
// Axioms — the lexical / value / canonical laws (Peterson et al. 2012).
// =============================================================================

/// Axiom: the boolean canonical mapping (§3.3.2.2) yields only
/// `"true"` / `"false"`, both re-parse to their value, and the lexical
/// space is exactly `{true, false, 1, 0}`.
pub struct BooleanCanonicalIsTrueFalse;

impl Axiom for BooleanCanonicalIsTrueFalse {
    fn verify(&self) -> Verdict {
        let ok = canonical_boolean(true) == "true"
            && canonical_boolean(false) == "false"
            && parse_boolean(canonical_boolean(true)) == Some(true)
            && parse_boolean(canonical_boolean(false)) == Some(false)
            && parse_boolean("1") == Some(true)
            && parse_boolean("0") == Some(false)
            && parse_boolean("True").is_none()
            && parse_boolean("").is_none()
            && parse_boolean("2").is_none();
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BooleanCanonicalIsTrueFalse",
        "the boolean lexical space is exactly {true,false,1,0}; the canonical mapping yields only \"true\"/\"false\" and re-parses to its value",
        "W3C XSD 1.1 Part 2 §3.3.2 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(BooleanCanonicalIsTrueFalse, "W3C XSD 1.1 Part 2 §3.3.2");

/// Axiom: the decimal canonical mapping is a fixpoint of re-parse —
/// `parse_decimal(canonical_decimal(v)) == Some(v)` and
/// `canonical_decimal` is idempotent through a re-parse — over a set
/// of sample literals spanning sign, integer-valued, and fractional
/// cases (§3.3.3.1).
pub struct DecimalCanonicalIsFixpoint;

impl Axiom for DecimalCanonicalIsFixpoint {
    fn verify(&self) -> Verdict {
        // (lexical input, expected canonical form).
        let cases = [
            ("0", "0"),
            ("+0", "0"),
            ("-0", "0"),
            ("00", "0"),
            ("1", "1"),
            ("+1", "1"),
            ("-5", "-5"),
            ("100", "100"),
            ("007", "7"),
            ("1.0", "1"),    // integer-valued decimal → no point (XSD 1.1)
            ("1.50", "1.5"), // trailing zero stripped
            (".5", "0.5"),   // missing integer part → "0"
            ("-0.250", "-0.25"),
            ("5.", "5"), // trailing dot, empty fraction → integer form
        ];
        for (lex, want) in cases {
            let Some(v) = parse_decimal(lex) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v.canonical_decimal() != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            // Fixpoint: re-parsing the canonical form gives the same
            // value, whose canonical form is identical.
            let Some(v2) = parse_decimal(want) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v2 != v || v2.canonical_decimal() != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DecimalCanonicalIsFixpoint",
        "parse_decimal then canonical_decimal is stable: the canonical literal re-parses to the same value and canonicalizes to itself; integer-valued decimals carry no decimal point",
        "W3C XSD 1.1 Part 2 §3.3.3.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(DecimalCanonicalIsFixpoint, "W3C XSD 1.1 Part 2 §3.3.3.1");

/// Axiom: the integer canonical mapping (§3.4.13.1) never contains a
/// decimal point, drops leading zeros and the `+` sign, and re-parses
/// to the same value.
pub struct IntegerCanonicalHasNoPoint;

impl Axiom for IntegerCanonicalHasNoPoint {
    fn verify(&self) -> Verdict {
        let cases = [("+007", "7"), ("-0", "0"), ("0", "0"), ("-128", "-128")];
        for (lex, want) in cases {
            let Some(v) = parse_integer(lex) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            let Some(canon) = v.canonical_integer() else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if canon != want || canon.contains('.') {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if parse_integer(&canon) != Some(v) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "IntegerCanonicalHasNoPoint",
        "the integer canonical mapping contains no decimal point, drops leading zeros and the + sign, and re-parses to the same value",
        "W3C XSD 1.1 Part 2 §3.4.13.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(IntegerCanonicalHasNoPoint, "W3C XSD 1.1 Part 2 §3.4.13.1");

/// Axiom: the bounded integer datatypes admit exactly their specified
/// ranges at the boundaries (W3C XSD 1.1 Part 2 §3.4.16–§3.4.25):
/// the min/max are in range and min−1 / max+1 are out.
pub struct BoundedIntegerRangesRespected;

impl Axiom for BoundedIntegerRangesRespected {
    fn verify(&self) -> Verdict {
        use XsdDatatypeConcept as D;
        // (datatype, in-range literal, out-of-range literal).
        let cases = [
            (D::Byte, "127", "128"),
            (D::Byte, "-128", "-129"),
            (D::Short, "32767", "32768"),
            (D::Short, "-32768", "-32769"),
            (D::Int, "2147483647", "2147483648"),
            (D::Int, "-2147483648", "-2147483649"),
            (D::Long, "9223372036854775807", "9223372036854775808"),
            (D::Long, "-9223372036854775808", "-9223372036854775809"),
            (D::UnsignedByte, "255", "256"),
            (D::UnsignedByte, "0", "-1"),
            (D::UnsignedShort, "65535", "65536"),
            (D::UnsignedInt, "4294967295", "4294967296"),
            (
                D::UnsignedLong,
                "18446744073709551615",
                "18446744073709551616",
            ),
            (D::NonNegativeInteger, "0", "-1"),
            (D::PositiveInteger, "1", "0"),
            (D::NonPositiveInteger, "0", "1"),
            (D::NegativeInteger, "-1", "0"),
        ];
        for (dt, inside, outside) in cases {
            if parse_typed_numeric(dt, inside).is_none() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if parse_typed_numeric(dt, outside).is_some() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "BoundedIntegerRangesRespected",
        "each bounded integer datatype admits its min and max and rejects min-1 / max+1; sign-bounded datatypes respect their sign constraint",
        "W3C XSD 1.1 Part 2 §3.4.16-§3.4.25 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    BoundedIntegerRangesRespected,
    "W3C XSD 1.1 Part 2 §3.4.16-§3.4.25"
);

/// Axiom: the decimal / integer lexical mappings reject literals
/// outside their grammars (§3.3.3.1 / §3.4.13.1) — empty strings, bare
/// signs/points, doubled points, non-digits, and (for integer) a
/// decimal point.
pub struct NumericLexicalRejectsMalformed;

impl Axiom for NumericLexicalRejectsMalformed {
    fn verify(&self) -> Verdict {
        let bad_decimal = [
            "", "+", "-", ".", "+.", "1.2.3", "1e5", "abc", " 1", "1 ", "+-1",
        ];
        let bad_integer = ["", "+", "1.0", "1.", ".5", "0x10", "abc", "1,000"];
        let ok = bad_decimal.iter().all(|s| parse_decimal(s).is_none())
            && bad_integer.iter().all(|s| parse_integer(s).is_none());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NumericLexicalRejectsMalformed",
        "the decimal and integer lexical mappings reject literals outside their grammars (empty, bare sign/point, doubled point, non-digits, exponent, embedded whitespace; integer also rejects a decimal point)",
        "W3C XSD 1.1 Part 2 §3.3.3.1, §3.4.13.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    NumericLexicalRejectsMalformed,
    "W3C XSD 1.1 Part 2 §3.3.3.1, §3.4.13.1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── boolean ──────────────────────────────────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn boolean_lexical_space() {
        assert_eq!(parse_boolean("true"), Some(true));
        assert_eq!(parse_boolean("1"), Some(true));
        assert_eq!(parse_boolean("false"), Some(false));
        assert_eq!(parse_boolean("0"), Some(false));
        assert_eq!(parse_boolean("TRUE"), None);
        assert_eq!(parse_boolean("yes"), None);
        assert_eq!(parse_boolean(""), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn boolean_canonical() {
        assert_eq!(canonical_boolean(true), "true");
        assert_eq!(canonical_boolean(false), "false");
    }

    // ── decimal ──────────────────────────────────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn decimal_parse_and_canonical() {
        let cases = [
            ("0", "0"),
            ("-0", "0"),
            ("+0", "0"),
            ("1.0", "1"),
            ("1.50", "1.5"),
            (".5", "0.5"),
            ("-0.250", "-0.25"),
            ("007", "7"),
            ("100", "100"),
            ("5.", "5"),
            ("-123.456", "-123.456"),
        ];
        for (lex, want) in cases {
            let v = parse_decimal(lex).unwrap_or_else(|| panic!("{lex} should parse"));
            assert_eq!(v.canonical_decimal(), want, "canonical of {lex}");
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn decimal_rejects_malformed() {
        for bad in ["", "+", "-", ".", "1.2.3", "1e3", "abc", "+.", " 1"] {
            assert!(parse_decimal(bad).is_none(), "{bad} should not parse");
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn decimal_value_equality_ignores_lexical_form() {
        assert_eq!(parse_decimal("1.0"), parse_decimal("1"));
        assert_eq!(parse_decimal("01.50"), parse_decimal("1.5"));
        assert_eq!(parse_decimal("-0"), parse_decimal("0"));
    }

    // ── integer family ───────────────────────────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn integer_parse_and_canonical() {
        let v = parse_integer("-007").unwrap();
        assert_eq!(v.canonical_integer().as_deref(), Some("-7"));
        assert!(!v.canonical_integer().unwrap().contains('.'));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn integer_rejects_decimal_point() {
        assert!(parse_integer("1.0").is_none());
        assert!(parse_integer("1.").is_none());
        assert!(parse_integer(".5").is_none());
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn bounded_integer_boundaries() {
        use XsdDatatypeConcept as D;
        assert!(parse_typed_numeric(D::Byte, "127").is_some());
        assert!(parse_typed_numeric(D::Byte, "-128").is_some());
        assert!(parse_typed_numeric(D::Byte, "128").is_none());
        assert!(parse_typed_numeric(D::Byte, "-129").is_none());
        assert!(parse_typed_numeric(D::UnsignedByte, "255").is_some());
        assert!(parse_typed_numeric(D::UnsignedByte, "256").is_none());
        assert!(parse_typed_numeric(D::UnsignedByte, "-1").is_none());
        // Arbitrary precision: a huge literal is a valid integer/decimal
        // but out of every bounded type's range.
        let huge = "123456789012345678901234567890";
        assert!(parse_typed_numeric(D::Integer, huge).is_some());
        assert!(parse_typed_numeric(D::Long, huge).is_none());
        assert!(parse_typed_numeric(D::UnsignedLong, huge).is_none());
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn sign_bounded_integers() {
        use XsdDatatypeConcept as D;
        assert!(parse_typed_numeric(D::NonNegativeInteger, "0").is_some());
        assert!(parse_typed_numeric(D::NonNegativeInteger, "-1").is_none());
        assert!(parse_typed_numeric(D::PositiveInteger, "1").is_some());
        assert!(parse_typed_numeric(D::PositiveInteger, "0").is_none());
        assert!(parse_typed_numeric(D::NonPositiveInteger, "0").is_some());
        assert!(parse_typed_numeric(D::NonPositiveInteger, "1").is_none());
        assert!(parse_typed_numeric(D::NegativeInteger, "-1").is_some());
        assert!(parse_typed_numeric(D::NegativeInteger, "0").is_none());
    }

    // ── axioms ───────────────────────────────────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_boolean_canonical() {
        assert!(BooleanCanonicalIsTrueFalse.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_decimal_fixpoint() {
        assert!(DecimalCanonicalIsFixpoint.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_integer_no_point() {
        assert!(IntegerCanonicalHasNoPoint.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn axiom_bounded_ranges() {
        assert!(BoundedIntegerRangesRespected.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axiom_lexical_rejects_malformed() {
        assert!(NumericLexicalRejectsMalformed.verify().is_ok());
    }

    // ── property-based ───────────────────────────────────────────────

    /// A digit-string generator producing valid decimal literals.
    fn arb_decimal_lexical() -> impl Strategy<Value = String> {
        (
            prop::bool::ANY,                // sign
            "[0-9]{1,8}",                   // integer part
            prop::option::of("[0-9]{1,8}"), // optional fraction
        )
            .prop_map(|(neg, int, frac)| {
                let sign = if neg { "-" } else { "" };
                match frac {
                    Some(f) => format!("{sign}{int}.{f}"),
                    None => format!("{sign}{int}"),
                }
            })
    }

    proptest! {
        /// Canonicalization is a fixpoint: re-parsing the canonical
        /// literal yields the same value and the same canonical form
        /// (Peterson et al. 2012 §3.3.3.1 — the canonical mapping is a
        /// section of the lexical mapping).
        #[test]
        fn prop_decimal_canonical_fixpoint(lex in arb_decimal_lexical()) {
            let v = parse_decimal(&lex).expect("generated literal is valid");
            let canon = v.canonical_decimal();
            let v2 = parse_decimal(&canon).expect("canonical literal re-parses");
            prop_assert_eq!(&v2, &v);
            prop_assert_eq!(v2.canonical_decimal(), canon);
        }

        /// The canonical decimal literal is itself in the lexical space.
        #[test]
        fn prop_decimal_canonical_in_lexical_space(lex in arb_decimal_lexical()) {
            let v = parse_decimal(&lex).expect("valid literal");
            prop_assert!(parse_decimal(&v.canonical_decimal()).is_some());
        }

        /// Integer canonicalization is a fixpoint and point-free.
        #[test]
        fn prop_integer_canonical_fixpoint(neg in prop::bool::ANY, digits in "[0-9]{1,20}") {
            let sign = if neg { "-" } else { "" };
            let lex = format!("{sign}{digits}");
            let v = parse_integer(&lex).expect("valid integer literal");
            let canon = v.canonical_integer().expect("integer value canonicalizes");
            prop_assert!(!canon.contains('.'));
            let reparsed = parse_integer(&canon);
            prop_assert_eq!(reparsed.as_ref(), Some(&v));
        }
    }

    pr4xis::register_praxis_value!(prop_decimal_canonical_fixpoint, Deterministic);
    pr4xis::register_praxis_value!(prop_decimal_canonical_in_lexical_space, Deterministic);
    pr4xis::register_praxis_value!(prop_integer_canonical_fixpoint, Deterministic);
}
