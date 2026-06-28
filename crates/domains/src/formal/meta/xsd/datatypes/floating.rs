//! Lexical / value / canonical mappings for the `float` and `double`
//! XSD datatypes (W3C XML Schema 1.1 Part 2 §3.3.4 / §3.3.5).
//!
//! ## Value space
//!
//! `xs:float` is the IEEE 754-2008 binary32 value set, `xs:double` the
//! binary64 set (§3.3.4 / §3.3.5): the finite values, positive and
//! negative zero, positive and negative infinity, and not-a-number.
//! These map exactly onto Rust's [`f32`] / [`f64`].
//!
//! ## Lexical space
//!
//! `·floatRep·` / `·doubleRep·` (§3.3.4.1 / §3.3.5.1):
//! `noDecimalPtNumeral | decimalPtNumeral | scientificNotationNumeral
//! | numericalSpecialRep`, where `numericalSpecialRep` is exactly one
//! of `INF`, `+INF`, `-INF`, `NaN` (case-sensitive — unlike Rust's own
//! float parser, which also accepts `inf`/`infinity`/`nan`; this module
//! enforces the narrower XSD grammar).
//!
//! ## Canonical mapping
//!
//! `·floatCanonicalMap·` (Appendix E):
//! - `positiveInfinity` → `"INF"`, `negativeInfinity` → `"-INF"`,
//!   `notANumber` → `"NaN"` (`·specialRepCanonicalMap·`);
//! - `positiveZero` → `"0.0E0"`, `negativeZero` → `"-0.0E0"`;
//! - otherwise `·scientificCanonicalMap·`: a sign, a mantissa in
//!   `[1,10)` written with a mandatory decimal point and ≥1 fraction
//!   digit (`·unsignedDecimalPtCanonicalMap·`), `'E'`, then the
//!   integer exponent (`·noDecimalPtCanonicalMap·` — no leading zeros,
//!   no `+`). The spec's map chooses the *smallest* mantissa `c` with
//!   `|f| = c × 10^e`; Rust's `{:e}` formatting yields exactly that
//!   shortest round-tripping decimal, so e.g. `100.0` → `"1.0E2"`,
//!   `123.4567` → `"1.234567E2"` (the spec's own worked example).
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.4 float, §3.3.5 double,
//!   Appendix E (·floatCanonicalMap·, ·scientificCanonicalMap·,
//!   ·specialRepCanonicalMap·).
//! - **IEEE 754-2008**, IEEE Standard for Floating-Point Arithmetic.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

// =============================================================================
// Lexical grammar — shared by float (§3.3.4.1) and double (§3.3.5.1).
// =============================================================================

/// True if `s` matches `noDecimalPtNumeral | decimalPtNumeral`:
/// `('+'|'-')? ( digit+ ('.' digit*)? | '.' digit+ )`. This is the
/// mantissa grammar; the exponent uses [`valid_signed_int`].
fn valid_signed_decimal(s: &str) -> bool {
    let body = strip_sign(s);
    match body.split_once('.') {
        Some((int, frac)) => {
            let int_ok = int.bytes().all(|b| b.is_ascii_digit());
            let frac_ok = frac.bytes().all(|b| b.is_ascii_digit());
            // At least one digit on some side; both sides all-digits.
            int_ok && frac_ok && !(int.is_empty() && frac.is_empty())
        }
        None => !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit()),
    }
}

/// True if `s` matches `noDecimalPtNumeral`: `('+'|'-')? digit+` (the
/// exponent grammar).
fn valid_signed_int(s: &str) -> bool {
    let body = strip_sign(s);
    !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit())
}

/// Strip a single leading `+`/`-`.
fn strip_sign(s: &str) -> &str {
    s.strip_prefix(['+', '-']).unwrap_or(s)
}

/// Classify a (non-special) numeral literal against the float/double
/// numeral grammar (`noDecimalPtNumeral | decimalPtNumeral |
/// scientificNotationNumeral`). Returns the literal split into mantissa
/// and optional exponent if valid, else `None`.
fn valid_numeral(s: &str) -> Option<(&str, Option<&str>)> {
    let (mant, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (s, None),
    };
    if !valid_signed_decimal(mant) {
        return None;
    }
    if let Some(e) = exp
        && !valid_signed_int(e)
    {
        return None;
    }
    Some((mant, exp))
}

/// Build a string Rust's `f32`/`f64` parser accepts, from an
/// XSD-validated numeral: ensure a digit on each side of any decimal
/// point (`"1."` → `"1.0"`, `".5"` → `"0.5"`). Value-preserving.
fn parseable(s: &str) -> String {
    let (neg, body) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let (mant, exp) = match body.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (body, None),
    };
    let mant = match mant.split_once('.') {
        Some((int, frac)) => {
            let int = if int.is_empty() { "0" } else { int };
            let frac = if frac.is_empty() { "0" } else { frac };
            format!("{int}.{frac}")
        }
        None => mant.to_string(),
    };
    let sign = if neg { "-" } else { "" };
    match exp {
        Some(e) => format!("{sign}{mant}e{e}"),
        None => format!("{sign}{mant}"),
    }
}

// =============================================================================
// Shared canonical scientific construction.
// =============================================================================

/// Convert Rust's `{:e}` formatting of a positive magnitude (e.g.
/// `"1e2"`, `"1.234567e2"`, `"5e-1"`) into the XSD
/// `scientificNotationNumeral` canonical form: a mantissa with a
/// mandatory decimal point and ≥1 fraction digit, `'E'`, and the
/// integer exponent. `neg` prepends the sign.
fn scientific_from_lower_exp(neg: bool, lower_exp: &str) -> String {
    let (mant, exp) = lower_exp
        .split_once('e')
        .expect("Rust {:e} always emits an exponent");
    let mant = if mant.contains('.') {
        mant.to_string()
    } else {
        format!("{mant}.0")
    };
    let sign = if neg { "-" } else { "" };
    format!("{sign}{mant}E{exp}")
}

// =============================================================================
// float — IEEE binary32 (§3.3.4).
// =============================================================================

/// Parse a `·floatRep·` (§3.3.4.1) into its [`f32`] value, or `None` if
/// `lex` is outside the lexical space. Special values must be exactly
/// `INF` / `+INF` / `-INF` / `NaN`.
pub fn parse_float(lex: &str) -> Option<f32> {
    match lex {
        "INF" | "+INF" => return Some(f32::INFINITY),
        "-INF" => return Some(f32::NEG_INFINITY),
        "NaN" => return Some(f32::NAN),
        _ => {}
    }
    valid_numeral(lex)?;
    parseable(lex).parse::<f32>().ok()
}

/// `·floatCanonicalMap·` (§3.3.4.2 / Appendix E) for an [`f32`] value.
pub fn canonical_float(v: f32) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-INF" } else { "INF" }.to_string();
    }
    if v == 0.0 {
        return if v.is_sign_negative() {
            "-0.0E0"
        } else {
            "0.0E0"
        }
        .to_string();
    }
    let neg = v < 0.0;
    scientific_from_lower_exp(neg, &format!("{:e}", v.abs()))
}

// =============================================================================
// double — IEEE binary64 (§3.3.5).
// =============================================================================

/// Parse a `·doubleRep·` (§3.3.5.1) into its [`f64`] value, or `None`
/// if `lex` is outside the lexical space.
pub fn parse_double(lex: &str) -> Option<f64> {
    match lex {
        "INF" | "+INF" => return Some(f64::INFINITY),
        "-INF" => return Some(f64::NEG_INFINITY),
        "NaN" => return Some(f64::NAN),
        _ => {}
    }
    valid_numeral(lex)?;
    parseable(lex).parse::<f64>().ok()
}

/// `·doubleCanonicalMap·` (§3.3.5.2 / Appendix E) for an [`f64`] value.
pub fn canonical_double(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-INF" } else { "INF" }.to_string();
    }
    if v == 0.0 {
        return if v.is_sign_negative() {
            "-0.0E0"
        } else {
            "0.0E0"
        }
        .to_string();
    }
    let neg = v < 0.0;
    scientific_from_lower_exp(neg, &format!("{:e}", v.abs()))
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: the float/double canonical mapping renders the special
/// values exactly as `·specialRepCanonicalMap·` / `·floatCanonicalMap·`
/// prescribe — `INF` / `-INF` / `NaN` and `0.0E0` / `-0.0E0` — and each
/// re-parses to the same value (with NaN compared by `is_nan` and the
/// zeros by sign bit).
pub struct FloatSpecialValuesCanonical;

impl Axiom for FloatSpecialValuesCanonical {
    fn verify(&self) -> Verdict {
        let ok = canonical_double(f64::INFINITY) == "INF"
            && canonical_double(f64::NEG_INFINITY) == "-INF"
            && canonical_double(f64::NAN) == "NaN"
            && canonical_double(0.0) == "0.0E0"
            && canonical_double(-0.0) == "-0.0E0"
            && canonical_float(f32::INFINITY) == "INF"
            && canonical_float(f32::NEG_INFINITY) == "-INF"
            && canonical_float(f32::NAN) == "NaN"
            && canonical_float(0.0) == "0.0E0"
            && canonical_float(-0.0) == "-0.0E0"
            // Special lexical reps round-trip; negative zero keeps sign.
            && parse_double("INF") == Some(f64::INFINITY)
            && parse_double("-INF") == Some(f64::NEG_INFINITY)
            && parse_double("NaN").map(f64::is_nan) == Some(true)
            && parse_double("-0.0E0").map(f64::to_bits) == Some((-0.0f64).to_bits());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FloatSpecialValuesCanonical",
        "the float/double canonical mapping renders INF/-INF/NaN and ±0 as INF/-INF/NaN and 0.0E0/-0.0E0, and the special lexical reps re-parse to the same value",
        "W3C XSD 1.1 Part 2 §3.3.4, §3.3.5, Appendix E (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    FloatSpecialValuesCanonical,
    "W3C XSD 1.1 Part 2 §3.3.4, §3.3.5, Appendix E"
);

/// Axiom: `·scientificCanonicalMap·` renders finite nonzero values in
/// scientific notation with a mantissa in `[1,10)` (mandatory `.` and
/// ≥1 fraction digit) and an integer exponent — checked against the
/// spec's worked example `123.4567` → `"1.234567E2"` and other fixed
/// cases.
pub struct FloatScientificCanonical;

impl Axiom for FloatScientificCanonical {
    fn verify(&self) -> Verdict {
        let cases = [
            (1.0_f64, "1.0E0"),
            (100.0, "1.0E2"),
            (1.5, "1.5E0"),
            (-2.5, "-2.5E0"),
            (0.5, "5.0E-1"),
            (123.4567, "1.234567E2"), // spec worked example
            (1000.0, "1.0E3"),
        ];
        for (v, want) in cases {
            if canonical_double(v) != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // float (binary32) renders the same clean magnitudes.
        if canonical_float(100.0) != "1.0E2" || canonical_float(1.5) != "1.5E0" {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FloatScientificCanonical",
        "the float/double canonical mapping renders finite nonzero values as a mantissa in [1,10) with a mandatory decimal point, E, and an integer exponent (e.g. 123.4567 -> 1.234567E2)",
        "W3C XSD 1.1 Part 2 §3.3.4.2, Appendix E ·scientificCanonicalMap· (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    FloatScientificCanonical,
    "W3C XSD 1.1 Part 2 §3.3.4.2, Appendix E"
);

/// Axiom: float/double canonicalization is a fixpoint of re-parse —
/// `parse(canonical(v))` recovers `v` bit-for-bit (NaN by `is_nan`) —
/// over a spread of finite, zero, and special values, and the
/// canonical literal is itself in the lexical space.
pub struct FloatCanonicalIsFixpoint;

impl Axiom for FloatCanonicalIsFixpoint {
    fn verify(&self) -> Verdict {
        let samples = [
            0.0_f64,
            -0.0,
            1.0,
            -1.0,
            123.4567,
            6.022e23,
            1.6e-19,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ];
        for v in samples {
            let canon = canonical_double(v);
            // Canonical literal must re-parse.
            let Some(back) = parse_double(&canon) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if back.to_bits() != v.to_bits() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            // And canonicalizing again is stable.
            if canonical_double(back) != canon {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // NaN round-trips by predicate.
        let nan_canon = canonical_double(f64::NAN);
        if parse_double(&nan_canon).map(f64::is_nan) != Some(true) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FloatCanonicalIsFixpoint",
        "parse then canonical is stable for float/double: the canonical literal re-parses bit-for-bit to the same value (NaN by predicate) and re-canonicalizes to itself",
        "W3C XSD 1.1 Part 2 §3.3.4.2, §3.3.5.2, Appendix E (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    FloatCanonicalIsFixpoint,
    "W3C XSD 1.1 Part 2 §3.3.4.2, §3.3.5.2, Appendix E"
);

/// Axiom: the float/double lexical mapping rejects literals outside the
/// XSD grammar — including the Rust-accepted but XSD-invalid spellings
/// `inf` / `Infinity` / `nan` / `+NaN`, underscores, hex, embedded
/// whitespace, and malformed exponents (§3.3.4.1 / §3.3.5.1).
pub struct FloatLexicalRejectsMalformed;

impl Axiom for FloatLexicalRejectsMalformed {
    fn verify(&self) -> Verdict {
        let bad = [
            "", "inf", "Infinity", "infinity", "nan", "+NaN", "-NaN", "1_000", "0x1p4", " 1", "1 ",
            "1e", "e3", "1.2.3", "1ee3", "1e2e3", "++1", ".", "+", "1e+", "1e2.5",
        ];
        let ok = bad
            .iter()
            .all(|s| parse_float(s).is_none() && parse_double(s).is_none());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FloatLexicalRejectsMalformed",
        "the float/double lexical mapping rejects literals outside the XSD grammar, including inf/Infinity/nan/+NaN, underscores, hex, embedded whitespace, and malformed exponents",
        "W3C XSD 1.1 Part 2 §3.3.4.1, §3.3.5.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    FloatLexicalRejectsMalformed,
    "W3C XSD 1.1 Part 2 §3.3.4.1, §3.3.5.1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn special_values_parse() {
        assert_eq!(parse_double("INF"), Some(f64::INFINITY));
        assert_eq!(parse_double("+INF"), Some(f64::INFINITY));
        assert_eq!(parse_double("-INF"), Some(f64::NEG_INFINITY));
        assert!(parse_double("NaN").unwrap().is_nan());
        // XSD is case-sensitive; Rust-accepted spellings are rejected.
        assert!(parse_double("inf").is_none());
        assert!(parse_double("Infinity").is_none());
        assert!(parse_double("nan").is_none());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn special_values_canonical() {
        assert_eq!(canonical_double(f64::INFINITY), "INF");
        assert_eq!(canonical_double(f64::NEG_INFINITY), "-INF");
        assert_eq!(canonical_double(f64::NAN), "NaN");
        assert_eq!(canonical_double(0.0), "0.0E0");
        assert_eq!(canonical_double(-0.0), "-0.0E0");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn scientific_canonical_examples() {
        assert_eq!(canonical_double(1.0), "1.0E0");
        assert_eq!(canonical_double(100.0), "1.0E2");
        assert_eq!(canonical_double(1.5), "1.5E0");
        assert_eq!(canonical_double(-2.5), "-2.5E0");
        assert_eq!(canonical_double(0.5), "5.0E-1");
        assert_eq!(canonical_double(123.4567), "1.234567E2");
        assert_eq!(canonical_float(100.0), "1.0E2");
        assert_eq!(canonical_float(0.5), "5.0E-1");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lexical_accepts_numeral_forms() {
        // noDecimalPt, decimalPt (incl. leading/trailing dot), scientific.
        for lex in [
            "1", "-1", "+1", "1.5", "1.", ".5", "1e3", "1.5E-2", ".5e+2", "0",
        ] {
            assert!(parse_double(lex).is_some(), "{lex} should parse");
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn lexical_rejects_malformed() {
        for bad in [
            "", "inf", "Infinity", "1_000", "0x1", " 1", "1e", "1.2.3", "1ee3", ".",
        ] {
            assert!(parse_double(bad).is_none(), "{bad} should not parse");
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn negative_zero_round_trips() {
        let c = canonical_double(-0.0);
        assert_eq!(c, "-0.0E0");
        assert_eq!(parse_double(&c).unwrap().to_bits(), (-0.0f64).to_bits());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_special_values() {
        assert!(FloatSpecialValuesCanonical.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_scientific() {
        assert!(FloatScientificCanonical.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_fixpoint() {
        assert!(FloatCanonicalIsFixpoint.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axiom_rejects_malformed() {
        assert!(FloatLexicalRejectsMalformed.verify().is_ok());
    }

    proptest! {
        /// Canonicalization is a bitwise fixpoint of re-parse for any
        /// finite double (Peterson et al. 2012 Appendix E: the
        /// canonical map is a section of the lexical map).
        #[test]
        fn prop_double_canonical_fixpoint(bits in any::<u64>()) {
            let v = f64::from_bits(bits);
            prop_assume!(v.is_finite());
            let canon = canonical_double(v);
            let back = parse_double(&canon).expect("canonical re-parses");
            prop_assert_eq!(back.to_bits(), v.to_bits());
            prop_assert_eq!(canonical_double(back), canon);
        }

        /// Same for any finite float (binary32).
        #[test]
        fn prop_float_canonical_fixpoint(bits in any::<u32>()) {
            let v = f32::from_bits(bits);
            prop_assume!(v.is_finite());
            let canon = canonical_float(v);
            let back = parse_float(&canon).expect("canonical re-parses");
            prop_assert_eq!(back.to_bits(), v.to_bits());
            prop_assert_eq!(canonical_float(back), canon);
        }

        /// The canonical literal is always itself in the lexical space.
        #[test]
        fn prop_canonical_in_lexical_space(bits in any::<u64>()) {
            let v = f64::from_bits(bits);
            let canon = canonical_double(v);
            prop_assert!(parse_double(&canon).is_some());
        }
    }

    pr4xis::register_praxis_value!(prop_double_canonical_fixpoint, Deterministic);
    pr4xis::register_praxis_value!(prop_float_canonical_fixpoint, Deterministic);
    pr4xis::register_praxis_value!(prop_canonical_in_lexical_space, Deterministic);
}
