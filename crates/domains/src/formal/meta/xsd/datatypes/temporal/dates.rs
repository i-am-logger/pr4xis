//! Lexical / value / canonical mappings for the date and Gregorian
//! XSD datatypes (W3C XML Schema 1.1 Part 2 §3.3.9-§3.3.14):
//! `date`, `gYearMonth`, `gYear`, `gMonthDay`, `gDay`, `gMonth`.
//!
//! Each is a projection of the seven-property model (§D.2.1) onto a
//! subset of {year, month, day} plus an optional timezone. Parsing
//! validates the fragment grammar and the Gregorian day-of-month rule;
//! the canonical mapping renders each present property through the
//! shared fragment maps in [`super::common`].
//!
//! The year is held as an [`i64`]; literals whose year magnitude
//! exceeds [`i64::MAX`] are outside this implementation's range (no
//! realistic calendar date approaches it).
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.9 date, §3.3.10 gYearMonth,
//!   §3.3.11 gYear, §3.3.12 gMonthDay, §3.3.13 gDay, §3.3.14 gMonth.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::common::{
    Timezone, canonical_timezone, days_in_month, parse_year_frag, split_trailing_timezone,
    two_digit, two_digit_value, year_fragment,
};

/// A value of the date / Gregorian family: the present subset of
/// {year, month, day} plus a timezone. Each datatype's parser fills
/// the fields it uses; the others stay `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateValue {
    pub year: Option<i64>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub tz: Timezone,
}

/// Validate a `monthFrag` (§3.3.7.1): two digits in `[01,12]`.
fn parse_month(s: &str) -> Option<u8> {
    let m = two_digit_value(s)?;
    (1..=12).contains(&m).then_some(m)
}

/// Validate a `dayFrag` (§3.3.7.1): two digits in `[01,31]`. The
/// per-month day bound is enforced by the caller (it needs the month).
fn parse_day(s: &str) -> Option<u8> {
    let d = two_digit_value(s)?;
    (1..=31).contains(&d).then_some(d)
}

// =============================================================================
// date — §3.3.9: yearFrag '-' monthFrag '-' dayFrag timezoneFrag?
// =============================================================================

/// Parse a `·dateLexicalRep·` (§3.3.9.1).
pub fn parse_date(lex: &str) -> Option<DateValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    // Split into year / month / day. The year may carry a leading '-',
    // so split month and day off the right.
    let (ym, day_s) = body.rsplit_once('-')?;
    let (year_s, month_s) = ym.rsplit_once('-')?;
    let year = parse_year_frag(year_s)?;
    let month = parse_month(month_s)?;
    let day = parse_day(day_s)?;
    if day > days_in_month(month, Some(year)) {
        return None;
    }
    Some(DateValue {
        year: Some(year),
        month: Some(month),
        day: Some(day),
        tz,
    })
}

/// `·dateCanonicalMap·` (§3.3.9.2 / Appendix E).
pub fn canonical_date(v: &DateValue) -> Option<String> {
    let (y, m, d) = (v.year?, v.month?, v.day?);
    Some(format!(
        "{}-{}-{}{}",
        year_fragment(y),
        two_digit(m),
        two_digit(d),
        canonical_timezone(v.tz)
    ))
}

// =============================================================================
// gYearMonth — §3.3.10: yearFrag '-' monthFrag timezoneFrag?
// =============================================================================

/// Parse a `·gYearMonthLexicalRep·` (§3.3.10.1).
pub fn parse_g_year_month(lex: &str) -> Option<DateValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let (year_s, month_s) = body.rsplit_once('-')?;
    let year = parse_year_frag(year_s)?;
    let month = parse_month(month_s)?;
    Some(DateValue {
        year: Some(year),
        month: Some(month),
        day: None,
        tz,
    })
}

/// `·gYearMonthCanonicalMap·` (§3.3.10.2).
pub fn canonical_g_year_month(v: &DateValue) -> Option<String> {
    let (y, m) = (v.year?, v.month?);
    Some(format!(
        "{}-{}{}",
        year_fragment(y),
        two_digit(m),
        canonical_timezone(v.tz)
    ))
}

// =============================================================================
// gYear — §3.3.11: yearFrag timezoneFrag?
// =============================================================================

/// Parse a `·gYearLexicalRep·` (§3.3.11.1).
pub fn parse_g_year(lex: &str) -> Option<DateValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let year = parse_year_frag(body)?;
    Some(DateValue {
        year: Some(year),
        month: None,
        day: None,
        tz,
    })
}

/// `·gYearCanonicalMap·` (§3.3.11.2).
pub fn canonical_g_year(v: &DateValue) -> Option<String> {
    let y = v.year?;
    Some(format!("{}{}", year_fragment(y), canonical_timezone(v.tz)))
}

// =============================================================================
// gMonthDay — §3.3.12: '--' monthFrag '-' dayFrag timezoneFrag?
// =============================================================================

/// Parse a `·gMonthDayLexicalRep·` (§3.3.12.1). With no year, February
/// admits day 29.
pub fn parse_g_month_day(lex: &str) -> Option<DateValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let rest = body.strip_prefix("--")?;
    let (month_s, day_s) = rest.split_once('-')?;
    let month = parse_month(month_s)?;
    let day = parse_day(day_s)?;
    if day > days_in_month(month, None) {
        return None;
    }
    Some(DateValue {
        year: None,
        month: Some(month),
        day: Some(day),
        tz,
    })
}

/// `·gMonthDayCanonicalMap·` (§3.3.12.2).
pub fn canonical_g_month_day(v: &DateValue) -> Option<String> {
    let (m, d) = (v.month?, v.day?);
    Some(format!(
        "--{}-{}{}",
        two_digit(m),
        two_digit(d),
        canonical_timezone(v.tz)
    ))
}

// =============================================================================
// gDay — §3.3.13: '---' dayFrag timezoneFrag?
// =============================================================================

/// Parse a `·gDayLexicalRep·` (§3.3.13.1).
pub fn parse_g_day(lex: &str) -> Option<DateValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let day_s = body.strip_prefix("---")?;
    let day = parse_day(day_s)?;
    Some(DateValue {
        year: None,
        month: None,
        day: Some(day),
        tz,
    })
}

/// `·gDayCanonicalMap·` (§3.3.13.2).
pub fn canonical_g_day(v: &DateValue) -> Option<String> {
    let d = v.day?;
    Some(format!("---{}{}", two_digit(d), canonical_timezone(v.tz)))
}

// =============================================================================
// gMonth — §3.3.14: '--' monthFrag timezoneFrag?
// =============================================================================

/// Parse a `·gMonthLexicalRep·` (§3.3.14.1).
pub fn parse_g_month(lex: &str) -> Option<DateValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let month_s = body.strip_prefix("--")?;
    let month = parse_month(month_s)?;
    Some(DateValue {
        year: None,
        month: Some(month),
        day: None,
        tz,
    })
}

/// `·gMonthCanonicalMap·` (§3.3.14.2).
pub fn canonical_g_month(v: &DateValue) -> Option<String> {
    let m = v.month?;
    Some(format!("--{}{}", two_digit(m), canonical_timezone(v.tz)))
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: every date-family canonical mapping is a fixpoint of
/// re-parse — the canonical literal re-parses to the same value and
/// re-canonicalizes to itself — across representative literals of all
/// six datatypes (Peterson et al. 2012 §3.3.9-§3.3.14).
pub struct DateCanonicalIsFixpoint;

impl Axiom for DateCanonicalIsFixpoint {
    fn verify(&self) -> Verdict {
        // (parser, canonicalizer, lexical input, expected canonical).
        type P = fn(&str) -> Option<DateValue>;
        type C = fn(&DateValue) -> Option<String>;
        let cases: &[(P, C, &str, &str)] = &[
            (parse_date, canonical_date, "2002-10-10", "2002-10-10"),
            (parse_date, canonical_date, "2002-10-10Z", "2002-10-10Z"),
            (
                parse_date,
                canonical_date,
                "2002-10-10+00:00",
                "2002-10-10Z",
            ),
            (
                parse_date,
                canonical_date,
                "2002-10-10-05:00",
                "2002-10-10-05:00",
            ),
            (parse_date, canonical_date, "0052-02-29", "0052-02-29"),
            (parse_date, canonical_date, "-0045-03-15", "-0045-03-15"),
            (
                parse_g_year_month,
                canonical_g_year_month,
                "2002-10",
                "2002-10",
            ),
            (parse_g_year, canonical_g_year, "2002", "2002"),
            (parse_g_year, canonical_g_year, "12000Z", "12000Z"),
            (
                parse_g_month_day,
                canonical_g_month_day,
                "--02-29",
                "--02-29",
            ),
            (parse_g_day, canonical_g_day, "---15", "---15"),
            (parse_g_month, canonical_g_month, "--10", "--10"),
        ];
        for (parse, canon, lex, want) in cases {
            let Some(v) = parse(lex) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            let Some(c) = canon(&v) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if &c != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            // Fixpoint: re-parse the canonical literal.
            let Some(v2) = parse(&c) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v2 != v || canon(&v2).as_deref() != Some(want) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DateCanonicalIsFixpoint",
        "every date/gregorian canonical mapping is stable: the canonical literal re-parses to the same value and re-canonicalizes to itself; +00:00 normalizes to Z",
        "W3C XSD 1.1 Part 2 §3.3.9-§3.3.14, Appendix E (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(DateCanonicalIsFixpoint, "W3C XSD 1.1 Part 2 §3.3.9-§3.3.14");

/// Axiom: the Gregorian day-of-month rule (§3.3.9.1 "Day-of-month
/// Values") is enforced — leap-day validity depends on the year for
/// `date`, but `gMonthDay` (no year) admits `--02-29`; impossible
/// days (Feb 30, Apr 31) are rejected.
pub struct DayOfMonthValidity;

impl Axiom for DayOfMonthValidity {
    fn verify(&self) -> Verdict {
        let ok = parse_date("2004-02-29").is_some()       // 2004 is a leap year
            && parse_date("2003-02-29").is_none()         // 2003 is not
            && parse_date("1900-02-29").is_none()         // century, not /400
            && parse_date("2000-02-29").is_some()         // /400 leap year
            && parse_date("2002-04-31").is_none()         // April has 30 days
            && parse_date("2002-13-01").is_none()         // no month 13
            && parse_g_month_day("--02-29").is_some()     // no year: 29 allowed
            && parse_g_month_day("--02-30").is_none()     // 30 never valid in Feb
            && parse_g_month_day("--04-31").is_none(); // April has 30 days
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DayOfMonthValidity",
        "the Gregorian day-of-month rule is enforced: leap-day validity depends on the year for date, gMonthDay admits --02-29, and impossible days (Feb 30, Apr 31) are rejected",
        "W3C XSD 1.1 Part 2 §3.3.9.1, §3.3.12.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(DayOfMonthValidity, "W3C XSD 1.1 Part 2 §3.3.9.1, §3.3.12.1");

/// Axiom: the date-family lexical mappings reject literals outside
/// their grammars — wrong separators, missing leading markers
/// (`--` / `---`), out-of-range fields, and malformed timezones
/// (§3.3.9.1-§3.3.14.1).
pub struct DateLexicalRejectsMalformed;

impl Axiom for DateLexicalRejectsMalformed {
    fn verify(&self) -> Verdict {
        let ok = parse_date("2002-10").is_none()         // missing day
            && parse_date("2002/10/10").is_none()        // wrong separator
            && parse_date("2002-10-10+15:00").is_none()  // tz out of range
            && parse_date("20-10-10").is_none()          // year < 4 digits
            && parse_g_month_day("-02-29").is_none()     // needs '--'
            && parse_g_day("--15").is_none()             // gDay needs '---'
            && parse_g_month("10").is_none()             // gMonth needs '--'
            && parse_g_year("2002-").is_none(); // trailing junk
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DateLexicalRejectsMalformed",
        "the date/gregorian lexical mappings reject wrong separators, missing leading markers, out-of-range fields, and malformed timezones",
        "W3C XSD 1.1 Part 2 §3.3.9.1-§3.3.14.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    DateLexicalRejectsMalformed,
    "W3C XSD 1.1 Part 2 §3.3.9.1-§3.3.14.1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn date_parse_and_canonical() {
        let v = parse_date("2002-10-10-05:00").unwrap();
        assert_eq!(v.year, Some(2002));
        assert_eq!(v.month, Some(10));
        assert_eq!(v.day, Some(10));
        assert_eq!(v.tz, Timezone::Offset(-300));
        assert_eq!(canonical_date(&v).as_deref(), Some("2002-10-10-05:00"));
    }

    #[test]
    fn utc_offset_normalizes_to_z() {
        let a = parse_date("2002-10-10+00:00").unwrap();
        let b = parse_date("2002-10-10Z").unwrap();
        assert_eq!(a, b);
        assert_eq!(canonical_date(&a).as_deref(), Some("2002-10-10Z"));
    }

    #[test]
    fn negative_and_large_years() {
        assert_eq!(
            canonical_g_year(&parse_g_year("-0045").unwrap()).as_deref(),
            Some("-0045")
        );
        assert_eq!(
            canonical_g_year(&parse_g_year("12000").unwrap()).as_deref(),
            Some("12000")
        );
        assert_eq!(
            canonical_g_year(&parse_g_year("0052").unwrap()).as_deref(),
            Some("0052")
        );
    }

    #[test]
    fn gregorian_subtypes() {
        assert!(parse_g_year_month("2002-10").is_some());
        assert!(parse_g_month_day("--02-29").is_some());
        assert!(parse_g_day("---31").is_some());
        assert!(parse_g_month("--12").is_some());
        assert_eq!(
            canonical_g_month_day(&parse_g_month_day("--02-29").unwrap()).as_deref(),
            Some("--02-29")
        );
    }

    #[test]
    fn axiom_date_fixpoint() {
        assert!(DateCanonicalIsFixpoint.verify().is_ok());
    }

    #[test]
    fn axiom_day_of_month() {
        assert!(DayOfMonthValidity.verify().is_ok());
    }

    #[test]
    fn axiom_rejects_malformed() {
        assert!(DateLexicalRejectsMalformed.verify().is_ok());
    }

    /// A generator of valid `date` literals (day ≤ 28 is valid in every
    /// month) with an optional timezone.
    fn arb_date_lexical() -> impl Strategy<Value = String> {
        (
            1i64..=9999,
            1u8..=12,
            1u8..=28,
            -840i16..=840,
            any::<bool>(),
        )
            .prop_map(|(y, m, d, off, has_tz)| {
                let tz = if !has_tz {
                    String::new()
                } else if off == 0 {
                    "Z".to_string()
                } else {
                    let sign = if off < 0 { '-' } else { '+' };
                    let mag = off.unsigned_abs();
                    format!("{sign}{:02}:{:02}", mag / 60, mag % 60)
                };
                format!("{y:04}-{m:02}-{d:02}{tz}")
            })
    }

    proptest! {
        /// Date canonicalization is a fixpoint of re-parse (Peterson et
        /// al. 2012 Appendix E).
        #[test]
        fn prop_date_canonical_fixpoint(lex in arb_date_lexical()) {
            let v = parse_date(&lex).expect("generated date is valid");
            let canon = canonical_date(&v).expect("date canonicalizes");
            let v2 = parse_date(&canon).expect("canonical re-parses");
            prop_assert_eq!(v2, v);
            let recanon = canonical_date(&v2);
            prop_assert_eq!(recanon.as_deref(), Some(canon.as_str()));
        }

        /// The canonical literal is itself in the lexical space.
        #[test]
        fn prop_date_canonical_in_lexical_space(lex in arb_date_lexical()) {
            let v = parse_date(&lex).expect("valid");
            let canon = canonical_date(&v).expect("canonicalizes");
            prop_assert!(parse_date(&canon).is_some());
        }
    }
}
