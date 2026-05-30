//! Shared fragments for the XSD date/time datatype family (W3C XML
//! Schema 1.1 Part 2 §3.3.7-§3.3.14, Appendix E).
//!
//! The date/time datatypes share a *seven-property model* (§D.2.1):
//! year, month, day, hour, minute, second, and timezoneOffset. This
//! module provides the fragments common to every member — the
//! [`Timezone`] value, its lexical/canonical mapping
//! (`·timezoneCanonicalFragmentMap·`), two-digit and four-digit-year
//! fragment renderers (`·unsTwoDigitCanonicalFragmentMap·`,
//! `·yearCanonicalFragmentMap·`), and the Gregorian day-of-month
//! validity rule.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.3.7-§3.3.14, §D.2, Appendix E.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString};

/// A timezone offset (W3C XSD 1.1 Part 2 §D.2.1 `·timezoneOffset·`):
/// either absent, or an integer number of minutes in `[-840, +840]`
/// (`±14:00`). UTC is `Offset(0)`, rendered canonically as `Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timezone {
    /// The `·timezoneOffset·` property is absent.
    Absent,
    /// Present, with an offset of this many minutes from UTC.
    Offset(i16),
}

/// The minimum / maximum admissible timezone offset in minutes
/// (`±14:00`, W3C XSD 1.1 Part 2 §3.2.7.3).
const TZ_LIMIT: i16 = 14 * 60;

/// Parse a `timezoneFrag` (§3.3.7.1): `Z | ('+'|'-') hh ':' mm`, with
/// the offset constrained to `[-14:00, +14:00]` and `mm` in `[0,59]`.
/// Returns `Some(Timezone::Offset(..))`, or `None` if malformed.
pub fn parse_timezone_frag(frag: &str) -> Option<Timezone> {
    if frag == "Z" {
        return Some(Timezone::Offset(0));
    }
    let bytes = frag.as_bytes();
    // ('+'|'-') hh ':' mm  — exactly 6 ASCII bytes.
    if bytes.len() != 6 {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    if bytes[3] != b':' {
        return None;
    }
    let hh = two_digit_value(&frag[1..3])?;
    let mm = two_digit_value(&frag[4..6])?;
    if mm > 59 {
        return None;
    }
    let offset = sign * (i16::from(hh) * 60 + i16::from(mm));
    if !(-TZ_LIMIT..=TZ_LIMIT).contains(&offset) {
        return None;
    }
    Some(Timezone::Offset(offset))
}

/// `·timezoneCanonicalFragmentMap·` (Appendix E): `Z` when the offset
/// is zero, a signed `hh:mm` otherwise; the empty string when absent.
pub fn canonical_timezone(tz: Timezone) -> String {
    match tz {
        Timezone::Absent => String::new(),
        Timezone::Offset(0) => "Z".to_string(),
        Timezone::Offset(t) => {
            let sign = if t < 0 { '-' } else { '+' };
            let mag = t.unsigned_abs();
            format!("{sign}{:02}:{:02}", mag / 60, mag % 60)
        }
    }
}

/// Split an optional trailing `timezoneFrag` off a date/time literal.
/// Returns the body (without the timezone) and the parsed
/// [`Timezone`]. Returns `None` only when a timezone-shaped suffix is
/// present but malformed (out of range / bad minutes).
pub fn split_trailing_timezone(s: &str) -> Option<(&str, Timezone)> {
    if let Some(body) = s.strip_suffix('Z') {
        return Some((body, Timezone::Offset(0)));
    }
    // A signed `hh:mm` suffix is the only source of ':' in the
    // date-family grammars, so a trailing `[+-]dd:dd` is unambiguous.
    if s.len() >= 6 {
        let cand = &s[s.len() - 6..];
        let b = cand.as_bytes();
        let shaped = (b[0] == b'+' || b[0] == b'-')
            && b[1].is_ascii_digit()
            && b[2].is_ascii_digit()
            && b[3] == b':'
            && b[4].is_ascii_digit()
            && b[5].is_ascii_digit();
        if shaped {
            let tz = parse_timezone_frag(cand)?;
            return Some((&s[..s.len() - 6], tz));
        }
    }
    Some((s, Timezone::Absent))
}

/// Parse a two-digit fragment (`'00'`…`'99'`) to its value, or `None`
/// if it is not exactly two ASCII digits.
pub fn two_digit_value(s: &str) -> Option<u8> {
    let b = s.as_bytes();
    if b.len() == 2 && b[0].is_ascii_digit() && b[1].is_ascii_digit() {
        Some((b[0] - b'0') * 10 + (b[1] - b'0'))
    } else {
        None
    }
}

/// `·unsTwoDigitCanonicalFragmentMap·` (Appendix E): a nonnegative
/// integer `< 100` as a zero-padded two-digit numeral.
pub fn two_digit(i: u8) -> String {
    format!("{i:02}")
}

/// `·yearCanonicalFragmentMap·` (Appendix E): an always-≥4-digit
/// numeral. `|year| < 10000` is rendered as four digits with a sign
/// for negatives (`·fourDigitCanonicalFragmentMap·`); larger
/// magnitudes use as many digits as needed (no leading zeros).
pub fn year_fragment(year: i64) -> String {
    let mag = year.unsigned_abs();
    let sign = if year < 0 { "-" } else { "" };
    if mag < 10000 {
        format!("{sign}{mag:04}")
    } else {
        format!("{sign}{mag}")
    }
}

/// Parse a `yearFrag` (§3.3.7.1): `'-'? (([1-9] digit digit digit+) |
/// ('0' digit digit digit))` — an optional sign then ≥4 digits, with
/// no leading zeros unless the magnitude is `< 1000` (exactly four
/// digits). Returns the signed year value.
pub fn parse_year_frag(frag: &str) -> Option<i64> {
    let (neg, digits) = match frag.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, frag),
    };
    if digits.len() < 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // No leading zeros unless exactly four digits (the '0' digit digit
    // digit branch), which permits years 0000-0999.
    if digits.as_bytes()[0] == b'0' && digits.len() != 4 {
        return None;
    }
    let value: i64 = digits.parse().ok()?;
    Some(if neg { -value } else { value })
}

/// True if `year` is a leap year in the proleptic Gregorian calendar
/// (W3C XSD 1.1 Part 2 §D.2.1): divisible by 4, except centuries not
/// divisible by 400.
pub fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

/// The number of days in `month` (`1`…`12`). February has 29 days when
/// `year` is a leap year, or when `year` is absent (as in `gMonthDay`,
/// which must admit `--02-29`; W3C XSD 1.1 Part 2 §3.3.12.1).
pub fn days_in_month(month: u8, year: Option<i64>) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => match year {
            Some(y) if !is_leap_year(y) => 28,
            _ => 29,
        },
        _ => 0,
    }
}
