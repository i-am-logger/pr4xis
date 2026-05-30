//! Lexical / value / canonical mappings for the duration XSD
//! datatypes (W3C XML Schema 1.1 Part 2 §3.3.6 / §3.4.26 / §3.4.27):
//! `duration`, `yearMonthDuration`, `dayTimeDuration`.
//!
//! ## Value space
//!
//! A duration value is a pair (`·months·`, `·seconds·`) sharing one
//! sign (§3.3.6). [`DurationValue`] stores the *total* months and the
//! *total* seconds (whole + fraction): a literal's year/month/day/
//! hour/minute fields are folded into these two totals at parse time,
//! and the canonical mapping re-decomposes them — so `P50M` →
//! `P4Y2M`, `PT100S` → `PT1M40S` (`·durationCanonicalMap·` via
//! `·duYearMonthCanonicalFragmentMap·` / `·duDayTimeCanonicalFragmentMap·`).
//!
//! Totals are held as [`u128`]: components must fit `u128`
//! (≈3.4×10^38 — about 10^28 years, far beyond any real duration).
//! A literal whose totals overflow `u128` is outside this range and
//! parses to `None` (fails closed).
//!
//! `yearMonthDuration` (§3.4.26) restricts the lexical space to the
//! Y/M fragments; `dayTimeDuration` (§3.4.27) to the D and T
//! fragments. Their canonical zero is `P0M` / `PT0S` respectively;
//! plain `duration`'s canonical zero is `PT0S`.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.6 duration, §3.4.26
//!   yearMonthDuration, §3.4.27 dayTimeDuration, Appendix E.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

const SECONDS_PER_DAY: u128 = 86400;
const SECONDS_PER_HOUR: u128 = 3600;
const SECONDS_PER_MINUTE: u128 = 60;
const MONTHS_PER_YEAR: u128 = 12;

/// A point in the `xs:duration` value space (§3.3.6): total months and
/// total seconds, sharing one sign. The seconds fraction carries no
/// trailing zeros. Zero is non-negative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationValue {
    negative: bool,
    months: u128,
    seconds_whole: u128,
    seconds_fraction: String,
}

impl DurationValue {
    fn is_zero(&self) -> bool {
        self.months == 0 && self.seconds_whole == 0 && self.seconds_fraction.is_empty()
    }

    /// `·duYearMonthCanonicalFragmentMap·` (Appendix E): decompose
    /// total months into `nY` `nM`, omitting zero components.
    fn year_month_fragment(&self) -> String {
        let years = self.months / MONTHS_PER_YEAR;
        let months = self.months % MONTHS_PER_YEAR;
        let mut s = String::new();
        if years > 0 {
            s.push_str(&format!("{years}Y"));
        }
        if months > 0 {
            s.push_str(&format!("{months}M"));
        }
        s
    }

    /// `·duDayTimeCanonicalFragmentMap·` (Appendix E): decompose total
    /// seconds into `nD` then `T nH nM nS`, omitting zero components.
    fn day_time_fragment(&self) -> String {
        let days = self.seconds_whole / SECONDS_PER_DAY;
        let mut rem = self.seconds_whole % SECONDS_PER_DAY;
        let hours = rem / SECONDS_PER_HOUR;
        rem %= SECONDS_PER_HOUR;
        let minutes = rem / SECONDS_PER_MINUTE;
        let seconds = rem % SECONDS_PER_MINUTE;

        let mut s = String::new();
        if days > 0 {
            s.push_str(&format!("{days}D"));
        }
        let has_time = hours > 0 || minutes > 0 || seconds > 0 || !self.seconds_fraction.is_empty();
        if has_time {
            s.push('T');
            if hours > 0 {
                s.push_str(&format!("{hours}H"));
            }
            if minutes > 0 {
                s.push_str(&format!("{minutes}M"));
            }
            if seconds > 0 || !self.seconds_fraction.is_empty() {
                if self.seconds_fraction.is_empty() {
                    s.push_str(&format!("{seconds}S"));
                } else {
                    s.push_str(&format!("{seconds}.{}S", self.seconds_fraction));
                }
            }
        }
        s
    }

    /// `·durationCanonicalMap·` (§3.3.6.2 / Appendix E). Zero is
    /// `PT0S`.
    pub fn canonical_duration(&self) -> String {
        let core = format!("{}{}", self.year_month_fragment(), self.day_time_fragment());
        if core.is_empty() {
            return "PT0S".to_string();
        }
        let sign = if self.negative { "-" } else { "" };
        format!("{sign}P{core}")
    }

    /// `·yearMonthDurationCanonicalMap·` (§3.4.26.2). Zero is `P0M`.
    pub fn canonical_year_month_duration(&self) -> String {
        let frag = self.year_month_fragment();
        if frag.is_empty() {
            return "P0M".to_string();
        }
        let sign = if self.negative { "-" } else { "" };
        format!("{sign}P{frag}")
    }

    /// `·dayTimeDurationCanonicalMap·` (§3.4.27.2). Zero is `PT0S`.
    pub fn canonical_day_time_duration(&self) -> String {
        let frag = self.day_time_fragment();
        if frag.is_empty() {
            return "PT0S".to_string();
        }
        let sign = if self.negative { "-" } else { "" };
        format!("{sign}P{frag}")
    }
}

/// One parsed `value designator` token, e.g. `("12", 'Y')`.
type DurToken = (String, char);

/// Tokenize a duration section into `(number, designator)` pairs.
/// Returns `None` on a malformed run (a designator with no number, a
/// non-digit/non-letter character, or a trailing number with no unit).
fn tokenize(section: &str) -> Option<Vec<DurToken>> {
    let mut out = Vec::new();
    let mut num = String::new();
    for c in section.chars() {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
        } else if c.is_ascii_alphabetic() {
            if num.is_empty() {
                return None;
            }
            out.push((core::mem::take(&mut num), c));
        } else {
            return None;
        }
    }
    if !num.is_empty() || out.is_empty() {
        return None;
    }
    Some(out)
}

/// Parse `unsignedNoDecimalPtNumeral` (`digit+`) to `u128`.
fn parse_u128_int(s: &str) -> Option<u128> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<u128>().ok()
}

/// Parse the `S` field's `unsignedNoDecimalPtNumeral |
/// unsignedDecimalPtNumeral` into `(whole, fraction-no-trailing-zeros)`.
fn parse_seconds_field(s: &str) -> Option<(u128, String)> {
    match s.split_once('.') {
        Some((w, f)) => {
            if f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            // `digit+ '.' digit+` or `'.' digit+` — at least one digit
            // before or after the point.
            let whole = if w.is_empty() { 0 } else { parse_u128_int(w)? };
            Some((whole, f.trim_end_matches('0').to_string()))
        }
        None => Some((parse_u128_int(s)?, String::new())),
    }
}

/// Internal: parse a `·durationLexicalRep·`, returning the value plus
/// flags of which sections were present (for the sub-type lexical
/// restrictions).
fn parse_duration_inner(lex: &str) -> Option<(DurationValue, bool, bool)> {
    let (negative, rest) = match lex.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, lex),
    };
    let body = rest.strip_prefix('P')?;
    let (date_part, time_part) = match body.split_once('T') {
        Some((d, t)) => (d, Some(t)),
        None => (body, None),
    };

    // Date part: Y, M, D in strictly increasing order, each optional.
    let mut months: u128 = 0;
    let mut seconds_whole: u128 = 0;
    let mut seconds_fraction = String::new();
    let mut had_year_month = false;
    let mut had_day_time = false;

    if !date_part.is_empty() {
        let toks = tokenize(date_part)?;
        let mut last_order = -1i8;
        for (num, unit) in toks {
            let (order, kind) = match unit {
                'Y' => (0i8, 'Y'),
                'M' => (1, 'M'),
                'D' => (2, 'D'),
                _ => return None,
            };
            if order <= last_order {
                return None; // out of order or repeated
            }
            last_order = order;
            let v = parse_u128_int(&num)?;
            match kind {
                'Y' => {
                    months = months.checked_add(v.checked_mul(MONTHS_PER_YEAR)?)?;
                    had_year_month = true;
                }
                'M' => {
                    months = months.checked_add(v)?;
                    had_year_month = true;
                }
                'D' => {
                    seconds_whole = seconds_whole.checked_add(v.checked_mul(SECONDS_PER_DAY)?)?;
                    had_day_time = true;
                }
                _ => return None,
            }
        }
    }

    // Time part: H, M, S in strictly increasing order; required to be
    // non-empty when 'T' is present.
    if let Some(tp) = time_part {
        let toks = tokenize(tp)?;
        had_day_time = true;
        let mut last_order = -1i8;
        for (num, unit) in toks {
            let order = match unit {
                'H' => 0i8,
                'M' => 1,
                'S' => 2,
                _ => return None,
            };
            if order <= last_order {
                return None;
            }
            last_order = order;
            match unit {
                'H' => {
                    let v = parse_u128_int(&num)?;
                    seconds_whole = seconds_whole.checked_add(v.checked_mul(SECONDS_PER_HOUR)?)?;
                }
                'M' => {
                    let v = parse_u128_int(&num)?;
                    seconds_whole =
                        seconds_whole.checked_add(v.checked_mul(SECONDS_PER_MINUTE)?)?;
                }
                'S' => {
                    let (whole, frac) = parse_seconds_field(&num)?;
                    seconds_whole = seconds_whole.checked_add(whole)?;
                    seconds_fraction = frac;
                }
                _ => return None,
            }
        }
    }

    // At least one component overall (durationLexicalRep requires it).
    if !had_year_month && !had_day_time {
        return None;
    }
    let mut value = DurationValue {
        negative,
        months,
        seconds_whole,
        seconds_fraction,
    };
    // No negative zero in the value space.
    if value.is_zero() {
        value.negative = false;
    }
    Some((value, had_year_month, had_day_time))
}

/// Parse a `·durationLexicalRep·` (§3.3.6.1).
pub fn parse_duration(lex: &str) -> Option<DurationValue> {
    parse_duration_inner(lex).map(|(v, _, _)| v)
}

/// Parse a `·yearMonthDurationLexicalRep·` (§3.4.26.1): a `duration`
/// literal restricted to the Y/M fragments (no day/time part).
pub fn parse_year_month_duration(lex: &str) -> Option<DurationValue> {
    let (v, had_ym, had_dt) = parse_duration_inner(lex)?;
    (had_ym && !had_dt).then_some(v)
}

/// Parse a `·dayTimeDurationLexicalRep·` (§3.4.27.1): a `duration`
/// literal restricted to the D and T fragments (no year/month part).
pub fn parse_day_time_duration(lex: &str) -> Option<DurationValue> {
    let (v, had_ym, had_dt) = parse_duration_inner(lex)?;
    (had_dt && !had_ym).then_some(v)
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: the duration canonical mapping is a fixpoint of re-parse and
/// normalizes carries — `P50M` → `P4Y2M`, `PT100S` → `PT1M40S`,
/// `PT3600S` → `PT1H` — with the seconds fraction's trailing zeros
/// stripped (Peterson et al. 2012 §3.3.6, Appendix E).
pub struct DurationCanonicalIsFixpoint;

impl Axiom for DurationCanonicalIsFixpoint {
    fn verify(&self) -> Verdict {
        let cases = [
            ("P1Y2M3DT4H5M6S", "P1Y2M3DT4H5M6S"),
            ("P50M", "P4Y2M"),
            ("PT100S", "PT1M40S"),
            ("PT3600S", "PT1H"),
            ("PT0S", "PT0S"),
            ("P0Y0M0D", "PT0S"), // all-zero → PT0S
            ("-P1Y", "-P1Y"),
            ("PT1M30.500S", "PT1M30.5S"), // fraction trailing zeros
            ("P1DT24H", "P2D"),           // 1 day + 24h = 2 days
            ("P0D", "PT0S"),
        ];
        for (lex, want) in cases {
            let Some(v) = parse_duration(lex) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v.canonical_duration() != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            let Some(v2) = parse_duration(want) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v2 != v || v2.canonical_duration() != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DurationCanonicalIsFixpoint",
        "the duration canonical mapping is stable and normalizes carries (P50M->P4Y2M, PT100S->PT1M40S), strips seconds-fraction trailing zeros, and renders zero as PT0S",
        "W3C XSD 1.1 Part 2 §3.3.6, Appendix E (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(DurationCanonicalIsFixpoint, "W3C XSD 1.1 Part 2 §3.3.6");

/// Axiom: the duration sub-types restrict the lexical space —
/// `yearMonthDuration` (§3.4.26) admits only Y/M fragments,
/// `dayTimeDuration` (§3.4.27) only D/T fragments — and their
/// canonical zeros are `P0M` / `PT0S`.
pub struct DurationSubtypeRestrictions;

impl Axiom for DurationSubtypeRestrictions {
    fn verify(&self) -> Verdict {
        let ok = parse_year_month_duration("P1Y2M").is_some()
            && parse_year_month_duration("P1Y2M3D").is_none()   // has day part
            && parse_year_month_duration("PT1H").is_none()      // has time part
            && parse_day_time_duration("P3DT4H").is_some()
            && parse_day_time_duration("P1Y").is_none()         // has year part
            && parse_day_time_duration("P1M").is_none()         // month, not minute
            && parse_year_month_duration("P0M").map(|v| v.canonical_year_month_duration())
                == Some("P0M".to_string())
            && parse_day_time_duration("PT0S").map(|v| v.canonical_day_time_duration())
                == Some("PT0S".to_string());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DurationSubtypeRestrictions",
        "yearMonthDuration admits only Y/M fragments and dayTimeDuration only D/T fragments; their canonical zeros are P0M and PT0S",
        "W3C XSD 1.1 Part 2 §3.4.26, §3.4.27 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    DurationSubtypeRestrictions,
    "W3C XSD 1.1 Part 2 §3.4.26, §3.4.27"
);

/// Axiom: the duration lexical mapping rejects literals outside the
/// grammar — a missing `P`, an empty `P`/`PT`, units out of order or
/// repeated, a minute before `T`, and empty fractions (§3.3.6.1).
pub struct DurationLexicalRejectsMalformed;

impl Axiom for DurationLexicalRejectsMalformed {
    fn verify(&self) -> Verdict {
        let bad = [
            "", "P", "PT", "1Y", "P1H", "PT1Y", "P1M1Y", "P-1Y", "P1YT", "PT1S1M", "PT.S", "P1.5Y",
        ];
        let ok = bad.iter().all(|s| parse_duration(s).is_none());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DurationLexicalRejectsMalformed",
        "the duration lexical mapping rejects a missing P, an empty P/PT, an hour in the date part, units out of order/repeated, and a non-S decimal field",
        "W3C XSD 1.1 Part 2 §3.3.6.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    DurationLexicalRejectsMalformed,
    "W3C XSD 1.1 Part 2 §3.3.6.1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn full_duration_round_trips() {
        let v = parse_duration("P1Y2M3DT4H5M6.250S").unwrap();
        assert_eq!(v.canonical_duration(), "P1Y2M3DT4H5M6.25S");
    }

    #[test]
    fn carries_normalize() {
        assert_eq!(
            parse_duration("P50M").unwrap().canonical_duration(),
            "P4Y2M"
        );
        assert_eq!(
            parse_duration("PT100S").unwrap().canonical_duration(),
            "PT1M40S"
        );
        assert_eq!(
            parse_duration("P1DT24H").unwrap().canonical_duration(),
            "P2D"
        );
    }

    #[test]
    fn zero_durations() {
        assert_eq!(parse_duration("PT0S").unwrap().canonical_duration(), "PT0S");
        assert_eq!(parse_duration("P0Y").unwrap().canonical_duration(), "PT0S");
        assert_eq!(
            parse_year_month_duration("P0M")
                .unwrap()
                .canonical_year_month_duration(),
            "P0M"
        );
    }

    #[test]
    fn negative_duration() {
        assert_eq!(parse_duration("-P1Y").unwrap().canonical_duration(), "-P1Y");
        // Negative zero collapses to positive.
        assert_eq!(
            parse_duration("-PT0S").unwrap().canonical_duration(),
            "PT0S"
        );
    }

    #[test]
    fn month_vs_minute_disambiguation() {
        // 'M' before T is months; after T is minutes.
        assert_eq!(parse_duration("P1M").unwrap().canonical_duration(), "P1M");
        assert_eq!(parse_duration("PT1M").unwrap().canonical_duration(), "PT1M");
    }

    #[test]
    fn subtype_restrictions() {
        assert!(parse_year_month_duration("P1Y2M").is_some());
        assert!(parse_year_month_duration("P1D").is_none());
        assert!(parse_day_time_duration("P1DT2H").is_some());
        assert!(parse_day_time_duration("P1Y").is_none());
    }

    #[test]
    fn arbitrary_precision_within_u128() {
        // A very large month count normalizes correctly.
        let v = parse_duration("P1000000000000M").unwrap();
        // 1e12 months = 83333333333 years + 4 months.
        assert_eq!(v.canonical_duration(), "P83333333333Y4M");
    }

    #[test]
    fn axiom_fixpoint() {
        assert!(DurationCanonicalIsFixpoint.verify().is_ok());
    }

    #[test]
    fn axiom_subtype_restrictions() {
        assert!(DurationSubtypeRestrictions.verify().is_ok());
    }

    #[test]
    fn axiom_rejects_malformed() {
        assert!(DurationLexicalRejectsMalformed.verify().is_ok());
    }

    fn arb_duration_lexical() -> impl Strategy<Value = String> {
        (
            any::<bool>(),
            0u64..=1000,
            0u64..=1000,
            0u64..=1000,
            0u64..=1000,
            0u64..=1000,
            0u64..=1000,
        )
            .prop_filter("at least one component", |&(_, y, mo, d, h, mi, s)| {
                y + mo + d + h + mi + s > 0
            })
            .prop_map(|(neg, y, mo, d, h, mi, s)| {
                let sign = if neg { "-" } else { "" };
                let mut date = String::new();
                if y > 0 {
                    date.push_str(&format!("{y}Y"));
                }
                if mo > 0 {
                    date.push_str(&format!("{mo}M"));
                }
                if d > 0 {
                    date.push_str(&format!("{d}D"));
                }
                let mut time = String::new();
                if h > 0 {
                    time.push_str(&format!("{h}H"));
                }
                if mi > 0 {
                    time.push_str(&format!("{mi}M"));
                }
                if s > 0 {
                    time.push_str(&format!("{s}S"));
                }
                let t = if time.is_empty() {
                    String::new()
                } else {
                    format!("T{time}")
                };
                format!("{sign}P{date}{t}")
            })
    }

    proptest! {
        /// Duration canonicalization is a fixpoint of re-parse
        /// (Peterson et al. 2012 Appendix E).
        #[test]
        fn prop_duration_canonical_fixpoint(lex in arb_duration_lexical()) {
            let v = parse_duration(&lex).expect("generated duration is valid");
            let canon = v.canonical_duration();
            let v2 = parse_duration(&canon).expect("canonical re-parses");
            prop_assert_eq!(&v2, &v);
            prop_assert_eq!(v2.canonical_duration(), canon);
        }

        /// The canonical literal is itself in the lexical space.
        #[test]
        fn prop_duration_canonical_in_lexical_space(lex in arb_duration_lexical()) {
            let v = parse_duration(&lex).expect("valid");
            prop_assert!(parse_duration(&v.canonical_duration()).is_some());
        }
    }
}
