//! Temporal constraint — the granularity unit + duration count that
//! pins down a legal deadline.
//!
//! Statutes pin deadlines as "within 180 days of X", "30 days after Y",
//! "promptly", "immediately". Praxis types both the *granularity* of
//! the count (Day / Week / Month / Year, or `Immediate` for no
//! duration) and binds it together with a numeric count as a
//! `Duration` value.
//!
//! # Concept partition
//!
//! ```text
//! TemporalConstraint
//!   ├── Immediate    — no duration; effective at the trigger
//!   ├── Day          — calendar days (ISO 8601)
//!   ├── BusinessDay  — Monday–Friday excluding federal holidays
//!   ├── Week         — calendar weeks (7 days)
//!   ├── Month        — calendar months (ISO 8601; variable-length)
//!   └── Year         — calendar years (ISO 8601)
//! ```
//!
//! `Immediate` is the partition element with no associated count.
//! The other five granularities pair with a `u32` count inside a
//! [`Duration`] typed wrapper.
//!
//! # Why BusinessDay is its own concept
//!
//! Many statutory deadlines run on *business days* rather than calendar
//! days — particularly in administrative procedure. FRCP 6(a)(6) and 5
//! U.S.C. § 5546(b) both define business days as excluding weekends
//! and federal holidays. The mathematical conversion (5 business days =
//! 7+ calendar days, depending on the weekday of the start) is non-
//! trivial enough to warrant a distinct ontology concept.
//!
//! # Literature
//!
//! - **Pustejovsky, Castaño, Ingria, Saurí, Gaizauskas, Setzer, Katz
//!   (2003)** "TimeML: Robust Specification of Event and Temporal
//!   Expressions in Text", in *Proceedings of the AAAI 2003 Spring
//!   Symposium on Reasoning about Time*. TIMEX3 expressions encode
//!   duration as `value` + `granularity` (e.g. "P180D" = 180 days).
//! - **ISO 24617-1:2012** *Language resource management — Semantic
//!   annotation framework — Part 1: Time and events* — international
//!   standard for TimeML annotation.
//! - **ISO 8601:2019** *Date and time — Representations for
//!   information interchange* — defines Day/Week/Month/Year
//!   granularity semantics.
//! - **Federal Rules of Civil Procedure, Rule 6** (Computing Time) —
//!   the FRCP's rules for computing deadlines, distinguishing calendar
//!   days from business days.
//! - **5 U.S.C. § 5546** — federal-holiday definition; basis for the
//!   BusinessDay exclusion set.

pub mod ontology;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
use alloc::string::String;

use self::ontology::TemporalConstraintConcept;

/// A typed duration: pairs a TemporalConstraint granularity with a
/// numeric count. `Immediate` is represented by `unit =
/// TemporalConstraintConcept::Immediate` and `count = 0` (the count is
/// ignored for `Immediate`).
///
/// This is the praxis-typed replacement for the historical bare
/// `DeadlineDuration::Days(u32) | Months(u32) | Immediate` enum: the
/// granularity is now a literature-grounded concept and the count is
/// carried as a typed sub-field rather than an untyped enum payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Duration {
    /// The granularity unit (Day / Week / Month / Year / BusinessDay / Immediate).
    pub unit: TemporalConstraintConcept,
    /// The count of `unit`s. Ignored when `unit` is `Immediate`.
    pub count: u32,
}

impl Duration {
    /// Construct an immediate duration (no count).
    pub fn immediate() -> Self {
        Self {
            unit: TemporalConstraintConcept::Immediate,
            count: 0,
        }
    }

    /// Construct a duration in calendar days.
    pub fn days(count: u32) -> Self {
        Self {
            unit: TemporalConstraintConcept::Day,
            count,
        }
    }

    /// Construct a duration in business days (FRCP Rule 6(a)(6)).
    pub fn business_days(count: u32) -> Self {
        Self {
            unit: TemporalConstraintConcept::BusinessDay,
            count,
        }
    }

    /// Construct a duration in calendar weeks.
    pub fn weeks(count: u32) -> Self {
        Self {
            unit: TemporalConstraintConcept::Week,
            count,
        }
    }

    /// Construct a duration in calendar months.
    pub fn months(count: u32) -> Self {
        Self {
            unit: TemporalConstraintConcept::Month,
            count,
        }
    }

    /// Construct a duration in calendar years.
    pub fn years(count: u32) -> Self {
        Self {
            unit: TemporalConstraintConcept::Year,
            count,
        }
    }

    /// True iff this is the Immediate variant (no waiting period).
    pub fn is_immediate(&self) -> bool {
        self.unit == TemporalConstraintConcept::Immediate
    }

    /// Render as a TIMEX3 / ISO 8601 duration string (Pustejovsky 2003).
    /// `P180D`, `P30D`, `P2M`, `P1Y`, etc. `Immediate` renders as `PT0S`
    /// (zero seconds).
    pub fn to_timex3(&self) -> String {
        use TemporalConstraintConcept as T;
        match self.unit {
            T::Immediate => "PT0S".into(),
            T::Day => format!("P{}D", self.count),
            T::BusinessDay => format!("P{}D[business]", self.count),
            T::Week => format!("P{}W", self.count),
            T::Month => format!("P{}M", self.count),
            T::Year => format!("P{}Y", self.count),
            // Abstract root has no representation.
            T::TemporalConstraint => "P0D".into(),
        }
    }
}
