//! Lexical / value / canonical mappings for the time-bearing XSD
//! datatypes (W3C XML Schema 1.1 Part 2 §3.3.7 / §3.3.8 / §3.4.28):
//! `dateTime`, `time`, and `dateTimeStamp`.
//!
//! These complete the seven-property model with the hour / minute /
//! second properties. Two subtleties are handled per spec:
//!
//! - **End-of-day** `24:00:00`(`.0+`)? is a valid lexical form that
//!   maps to `00:00:00`; for `dateTime` it rolls the date to the next
//!   day (§3.3.7.2). The canonical mapping never emits `24:…`.
//! - **No timezone normalization.** In XSD 1.1 the value space is the
//!   seven-property tuple with the `·timezoneOffset·` *retained*, so
//!   `…12:00:00-05:00` and `…17:00:00Z` are *distinct* values (§3.3.7
//!   "now denote unequal values"). The canonical mapping renders the
//!   offset as given (`Z` for zero) and never shifts to UTC.
//!
//! `dateTimeStamp` (§3.4.28) restricts `dateTime` with
//! `explicitTimezone = required`: a timezone must be present.
//!
//! Fractional seconds carry no trailing zeros
//! (`·secondCanonicalFragmentMap·`); `12.250` → `12.25`, `12.0` → `12`.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.7 dateTime, §3.3.8 time,
//!   §3.4.28 dateTimeStamp, Appendix E.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::common::{
    Timezone, canonical_timezone, days_in_month, parse_year_frag, split_trailing_timezone,
    two_digit, two_digit_value, year_fragment,
};

/// The hour / minute / second triple of the seven-property model. The
/// second is split into a whole part (`0`–`59`) and a fraction with no
/// trailing zeros (empty for a whole second).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub second_fraction: String,
}

/// An `xs:time` value: a [`TimeOfDay`] plus an optional timezone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeValue {
    pub time: TimeOfDay,
    pub tz: Timezone,
}

/// An `xs:dateTime` value: a date, a [`TimeOfDay`], and an optional
/// timezone (required for `xs:dateTimeStamp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeValue {
    pub year: i64,
    pub month: u8,
    pub day: u8,
    pub time: TimeOfDay,
    pub tz: Timezone,
}

/// Parse the time portion `hh ':' mm ':' ss ('.' frac)?` or the
/// end-of-day form `24:00:00` (`'.' '0'+`)?. Returns the [`TimeOfDay`]
/// (end-of-day yields `00:00:00`) and whether it was the end-of-day
/// form (so the caller can roll the date forward).
fn parse_time_of_day(s: &str) -> Option<(TimeOfDay, bool)> {
    // End-of-day: '24:00:00' optionally followed by '.' then all zeros.
    if s == "24:00:00"
        || s.strip_prefix("24:00:00.")
            .is_some_and(|f| !f.is_empty() && f.bytes().all(|b| b == b'0'))
    {
        return Some((
            TimeOfDay {
                hour: 0,
                minute: 0,
                second: 0,
                second_fraction: String::new(),
            },
            true,
        ));
    }
    let mut parts = s.splitn(3, ':');
    let hh = two_digit_value(parts.next()?)?;
    let mm = two_digit_value(parts.next()?)?;
    let sec_field = parts.next()?;
    if hh > 23 || mm > 59 {
        return None;
    }
    let (whole_s, frac) = match sec_field.split_once('.') {
        Some((w, f)) => {
            if f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            (w, f.trim_end_matches('0').to_string())
        }
        None => (sec_field, String::new()),
    };
    let second = two_digit_value(whole_s)?;
    if second > 59 {
        return None;
    }
    Some((
        TimeOfDay {
            hour: hh,
            minute: mm,
            second,
            second_fraction: frac,
        },
        false,
    ))
}

/// `·secondCanonicalFragmentMap·` (Appendix E): two-digit second, with
/// a `.`-prefixed fraction (no trailing zeros) only when present.
fn canonical_second(t: &TimeOfDay) -> String {
    if t.second_fraction.is_empty() {
        two_digit(t.second)
    } else {
        format!("{}.{}", two_digit(t.second), t.second_fraction)
    }
}

/// Render the `hh:mm:ss(.fff)?` time fragment.
fn canonical_time_of_day(t: &TimeOfDay) -> String {
    format!(
        "{}:{}:{}",
        two_digit(t.hour),
        two_digit(t.minute),
        canonical_second(t)
    )
}

/// Advance a (year, month, day) one calendar day forward, rolling
/// month and year as needed (used by the dateTime end-of-day form).
fn next_day(year: i64, month: u8, day: u8) -> (i64, u8, u8) {
    if day < days_in_month(month, Some(year)) {
        (year, month, day + 1)
    } else if month < 12 {
        (year, month + 1, 1)
    } else {
        (year + 1, 1, 1)
    }
}

// =============================================================================
// time — §3.3.8.
// =============================================================================

/// Parse a `·timeLexicalRep·` (§3.3.8.1). End-of-day `24:00:00` maps to
/// `00:00:00`.
pub fn parse_time(lex: &str) -> Option<TimeValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let (time, _end_of_day) = parse_time_of_day(body)?;
    Some(TimeValue { time, tz })
}

/// `·timeCanonicalMap·` (§3.3.8.2).
pub fn canonical_time(v: &TimeValue) -> String {
    format!(
        "{}{}",
        canonical_time_of_day(&v.time),
        canonical_timezone(v.tz)
    )
}

// =============================================================================
// dateTime — §3.3.7.
// =============================================================================

/// Parse a `·dateTimeLexicalRep·` (§3.3.7.1):
/// `date 'T' time timezoneFrag?`. End-of-day rolls the date forward.
pub fn parse_date_time(lex: &str) -> Option<DateTimeValue> {
    let (body, tz) = split_trailing_timezone(lex)?;
    let (date_s, time_s) = body.split_once('T')?;
    // date part: yearFrag '-' monthFrag '-' dayFrag.
    let (ym, day_s) = date_s.rsplit_once('-')?;
    let (year_s, month_s) = ym.rsplit_once('-')?;
    let year = parse_year_frag(year_s)?;
    let month = two_digit_value(month_s).filter(|m| (1..=12).contains(m))?;
    let day = two_digit_value(day_s).filter(|d| (1..=31).contains(d))?;
    if day > days_in_month(month, Some(year)) {
        return None;
    }
    let (time, end_of_day) = parse_time_of_day(time_s)?;
    let (year, month, day) = if end_of_day {
        next_day(year, month, day)
    } else {
        (year, month, day)
    };
    Some(DateTimeValue {
        year,
        month,
        day,
        time,
        tz,
    })
}

/// `·dateTimeCanonicalMap·` (§3.3.7.2 / Appendix E).
pub fn canonical_date_time(v: &DateTimeValue) -> String {
    format!(
        "{}-{}-{}T{}{}",
        year_fragment(v.year),
        two_digit(v.month),
        two_digit(v.day),
        canonical_time_of_day(&v.time),
        canonical_timezone(v.tz)
    )
}

// =============================================================================
// dateTimeStamp — §3.4.28 (dateTime with explicitTimezone = required).
// =============================================================================

/// Parse an `xs:dateTimeStamp` (§3.4.28): an `xs:dateTime` whose
/// timezone is *required*. Returns `None` if the timezone is absent.
pub fn parse_date_time_stamp(lex: &str) -> Option<DateTimeValue> {
    let v = parse_date_time(lex)?;
    (v.tz != Timezone::Absent).then_some(v)
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: the dateTime/time canonical mappings are fixpoints of
/// re-parse — the canonical literal re-parses to the same value and
/// re-canonicalizes to itself — including fractional-second
/// trailing-zero stripping and end-of-day rollover (Peterson et al.
/// 2012 §3.3.7-§3.3.8).
pub struct DateTimeCanonicalIsFixpoint;

impl Axiom for DateTimeCanonicalIsFixpoint {
    fn verify(&self) -> Verdict {
        // dateTime cases: (lexical, expected canonical).
        let dt_cases = [
            ("2002-10-10T12:00:00", "2002-10-10T12:00:00"),
            ("2002-10-10T12:00:00Z", "2002-10-10T12:00:00Z"),
            ("2002-10-10T12:00:00+00:00", "2002-10-10T12:00:00Z"),
            ("2002-10-10T12:00:00-05:00", "2002-10-10T12:00:00-05:00"),
            ("2002-10-10T12:00:00.250", "2002-10-10T12:00:00.25"),
            ("2002-10-10T12:00:00.0", "2002-10-10T12:00:00"),
            // End-of-day rolls to the next day (and month/year).
            ("2002-10-10T24:00:00", "2002-10-11T00:00:00"),
            ("2002-12-31T24:00:00", "2003-01-01T00:00:00"),
        ];
        for (lex, want) in dt_cases {
            let Some(v) = parse_date_time(lex) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if canonical_date_time(&v) != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            let Some(v2) = parse_date_time(want) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v2 != v || canonical_date_time(&v2) != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // time cases.
        let t_cases = [
            ("12:30:00", "12:30:00"),
            ("24:00:00", "00:00:00"),
            ("12:30:45.6700", "12:30:45.67"),
            ("12:30:45Z", "12:30:45Z"),
        ];
        for (lex, want) in t_cases {
            let Some(v) = parse_time(lex) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if canonical_time(&v) != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            let Some(v2) = parse_time(want) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v2 != v || canonical_time(&v2) != want {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DateTimeCanonicalIsFixpoint",
        "the dateTime/time canonical mappings are stable: the canonical literal re-parses to the same value and re-canonicalizes to itself, with fractional-second trailing zeros stripped, end-of-day rolled forward, and +00:00 normalized to Z",
        "W3C XSD 1.1 Part 2 §3.3.7, §3.3.8, Appendix E (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    DateTimeCanonicalIsFixpoint,
    "W3C XSD 1.1 Part 2 §3.3.7, §3.3.8, Appendix E"
);

/// Axiom: `xs:dateTimeStamp` (§3.4.28) requires a timezone — a
/// dateTime literal without one is rejected, one with a timezone is
/// accepted.
pub struct DateTimeStampRequiresTimezone;

impl Axiom for DateTimeStampRequiresTimezone {
    fn verify(&self) -> Verdict {
        let ok = parse_date_time_stamp("2002-10-10T12:00:00Z").is_some()
            && parse_date_time_stamp("2002-10-10T12:00:00-05:00").is_some()
            && parse_date_time_stamp("2002-10-10T12:00:00").is_none();
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DateTimeStampRequiresTimezone",
        "dateTimeStamp restricts dateTime with explicitTimezone=required: a literal without a timezone is rejected, one with a timezone accepted",
        "W3C XSD 1.1 Part 2 §3.4.28 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(DateTimeStampRequiresTimezone, "W3C XSD 1.1 Part 2 §3.4.28");

/// Axiom: the dateTime/time lexical mappings reject literals outside
/// the grammar — out-of-range hours/minutes/seconds, a `24:` form
/// other than `24:00:00`, a missing `T`, and empty fractions
/// (§3.3.7.1 / §3.3.8.1).
pub struct DateTimeLexicalRejectsMalformed;

impl Axiom for DateTimeLexicalRejectsMalformed {
    fn verify(&self) -> Verdict {
        let ok = parse_date_time("2002-10-10T25:00:00").is_none()   // hour 25
            && parse_date_time("2002-10-10T12:60:00").is_none()     // minute 60
            && parse_date_time("2002-10-10T12:00:60").is_none()     // second 60
            && parse_date_time("2002-10-10T24:00:01").is_none()     // bad end-of-day
            && parse_date_time("2002-10-10 12:00:00").is_none()     // missing 'T'
            && parse_date_time("2002-10-10T12:00:00.").is_none()    // empty fraction
            && parse_time("12:00").is_none()                        // missing seconds
            && parse_time("24:00:01").is_none(); // bad end-of-day
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DateTimeLexicalRejectsMalformed",
        "the dateTime/time lexical mappings reject out-of-range fields, a 24: form other than 24:00:00, a missing T separator, and empty fractions",
        "W3C XSD 1.1 Part 2 §3.3.7.1, §3.3.8.1 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    DateTimeLexicalRejectsMalformed,
    "W3C XSD 1.1 Part 2 §3.3.7.1, §3.3.8.1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable, Deterministic)]
    #[test]
    fn date_time_parse_and_canonical() {
        let v = parse_date_time("2002-10-10T12:30:45.250-05:00").unwrap();
        assert_eq!(v.year, 2002);
        assert_eq!(v.time.hour, 12);
        assert_eq!(v.time.second, 45);
        assert_eq!(v.time.second_fraction, "25");
        assert_eq!(v.tz, Timezone::Offset(-300));
        assert_eq!(canonical_date_time(&v), "2002-10-10T12:30:45.25-05:00");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn end_of_day_rolls_forward() {
        assert_eq!(
            canonical_date_time(&parse_date_time("2002-12-31T24:00:00").unwrap()),
            "2003-01-01T00:00:00"
        );
        // time has no date to roll: 24:00:00 -> 00:00:00.
        assert_eq!(canonical_time(&parse_time("24:00:00").unwrap()), "00:00:00");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn fractional_seconds_strip_trailing_zeros() {
        assert_eq!(
            canonical_time(&parse_time("12:00:00.5000").unwrap()),
            "12:00:00.5"
        );
        assert_eq!(
            canonical_time(&parse_time("12:00:00.0").unwrap()),
            "12:00:00"
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn utc_offset_normalizes_to_z() {
        assert_eq!(
            canonical_date_time(&parse_date_time("2002-10-10T12:00:00+00:00").unwrap()),
            "2002-10-10T12:00:00Z"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn date_time_stamp_requires_tz() {
        assert!(parse_date_time_stamp("2002-10-10T12:00:00Z").is_some());
        assert!(parse_date_time_stamp("2002-10-10T12:00:00").is_none());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_fixpoint() {
        assert!(DateTimeCanonicalIsFixpoint.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axiom_stamp_requires_tz() {
        assert!(DateTimeStampRequiresTimezone.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axiom_rejects_malformed() {
        assert!(DateTimeLexicalRejectsMalformed.verify().is_ok());
    }

    fn arb_date_time_lexical() -> impl Strategy<Value = String> {
        (
            1i64..=9999,
            1u8..=12,
            1u8..=28,
            0u8..=23,
            0u8..=59,
            0u8..=59,
            prop::option::of(1u32..=999999),
            -840i16..=840,
            any::<bool>(),
        )
            .prop_map(|(y, mo, d, h, mi, s, frac, off, has_tz)| {
                let sec = match frac {
                    Some(f) => format!("{s:02}.{f:06}"),
                    None => format!("{s:02}"),
                };
                let tz = if !has_tz {
                    String::new()
                } else if off == 0 {
                    "Z".to_string()
                } else {
                    let sign = if off < 0 { '-' } else { '+' };
                    let mag = off.unsigned_abs();
                    format!("{sign}{:02}:{:02}", mag / 60, mag % 60)
                };
                format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{sec}{tz}")
            })
    }

    proptest! {
        /// dateTime canonicalization is a fixpoint of re-parse
        /// (Peterson et al. 2012 Appendix E).
        #[test]
        fn prop_date_time_canonical_fixpoint(lex in arb_date_time_lexical()) {
            let v = parse_date_time(&lex).expect("generated dateTime is valid");
            let canon = canonical_date_time(&v);
            let v2 = parse_date_time(&canon).expect("canonical re-parses");
            prop_assert_eq!(&v2, &v);
            prop_assert_eq!(canonical_date_time(&v2), canon);
        }

        /// The canonical literal is itself in the lexical space.
        #[test]
        fn prop_date_time_canonical_in_lexical_space(lex in arb_date_time_lexical()) {
            let v = parse_date_time(&lex).expect("valid");
            prop_assert!(parse_date_time(&canonical_date_time(&v)).is_some());
        }
    }

    pr4xis::register_praxis_value!(prop_date_time_canonical_fixpoint, Deterministic);
    pr4xis::register_praxis_value!(prop_date_time_canonical_in_lexical_space, Deterministic);
}
