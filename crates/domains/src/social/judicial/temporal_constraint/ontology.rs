//! Temporal constraint ontology — concepts, is_a, axioms.
//!
//! See `mod.rs` for the literature inventory and the `Duration` value type.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "TemporalConstraint",
    source: "Pustejovsky, Castaño, Ingria, Saurí, Gaizauskas, Setzer, Katz (2003) TimeML: Robust Specification of Event and Temporal Expressions in Text, AAAI Spring Symposium on Reasoning about Time; ISO 24617-1:2012 SemAF Part 1: Time and events; ISO 8601:2019 Date and time representations; Federal Rules of Civil Procedure, Rule 6 (Computing Time); 5 U.S.C. § 5546",

    concepts: [
        // Root
        TemporalConstraint,

        // Granularity leaves
        Immediate,    // no duration; effective at the trigger
        Day,          // calendar day (ISO 8601)
        BusinessDay,  // FRCP Rule 6(a)(6) business day
        Week,         // 7 calendar days
        Month,        // ISO 8601 calendar month
        Year,         // ISO 8601 calendar year
    ],

    labels: {
        TemporalConstraint: ("en", "Temporal constraint",
            "Pustejovsky (2003): the granularity unit pinning down a temporal duration in a legal deadline or timing rule."),
        Immediate: ("en", "Immediate",
            "TimeML PT0S: no waiting period; the obligation becomes operative at the trigger event."),
        Day: ("en", "Day",
            "ISO 8601 calendar day. TimeML granularity D."),
        BusinessDay: ("en", "Business day",
            "FRCP Rule 6(a)(6): Monday-Friday excluding federal holidays (5 U.S.C. § 5546)."),
        Week: ("en", "Week",
            "ISO 8601 calendar week (7 days). TimeML granularity W."),
        Month: ("en", "Month",
            "ISO 8601 calendar month (variable-length, 28-31 days). TimeML granularity M."),
        Year: ("en", "Year",
            "ISO 8601 calendar year. TimeML granularity Y."),
    },

    is_a: [
        (Immediate, TemporalConstraint),
        (Day, TemporalConstraint),
        (BusinessDay, TemporalConstraint),
        (Week, TemporalConstraint),
        (Month, TemporalConstraint),
        (Year, TemporalConstraint),
    ],
}

// ---------------------------------------------------------------------------
// Quality: ApproximateDays — for ordering/comparison, the average number
// of calendar days a single unit represents. Used by axioms and by
// callers reasoning about deadline stringency.
// ---------------------------------------------------------------------------

/// Average days-per-unit for ordering purposes. BusinessDay uses a
/// fractional approximation (5 business days ≈ 7 calendar days) — a
/// caller doing exact computation should resolve against the actual
/// calendar.
///
/// Returns `None` for the abstract root and for `Immediate` (which has
/// zero-duration semantics and shouldn't be ordered with positive
/// durations).
#[derive(Debug, Clone)]
pub struct ApproximateDays;

impl Quality for ApproximateDays {
    type Individual = TemporalConstraintConcept;
    type Value = u32;

    fn get(&self, c: &TemporalConstraintConcept) -> Option<u32> {
        use TemporalConstraintConcept as T;
        match c {
            T::Day => Some(1),
            T::BusinessDay => Some(2), // 5 business days ≈ 7 calendar days, rounded up per-unit
            T::Week => Some(7),
            T::Month => Some(30),
            T::Year => Some(365),
            T::Immediate | T::TemporalConstraint => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn granularity_leaves() -> [TemporalConstraintConcept; 5] {
    [
        TemporalConstraintConcept::Day,
        TemporalConstraintConcept::BusinessDay,
        TemporalConstraintConcept::Week,
        TemporalConstraintConcept::Month,
        TemporalConstraintConcept::Year,
    ]
}

pub fn is_leaf(c: TemporalConstraintConcept) -> bool {
    !matches!(c, TemporalConstraintConcept::TemporalConstraint)
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for TemporalConstraintOntology {
    type Cat = TemporalConstraintCategory;
    type Qual = ApproximateDays;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(GranularityOrdering));
        axioms.push(Box::new(ImmediateIsZeroDuration));
        axioms.push(Box::new(BusinessDayBelowWeek));
        axioms
    }
}

/// Axiom: the granularity leaves admit a strict ordering by
/// `ApproximateDays`: Day < BusinessDay < Week < Month < Year. The
/// ordering reflects ISO 8601 + FRCP Rule 6 semantics — a single
/// business day spans on average ~1.4 calendar days; a single week is
/// 7; a single month is ≈30; a year ≈365.
pub struct GranularityOrdering;

impl Axiom for GranularityOrdering {
    fn verify(&self) -> Verdict {
        let q = ApproximateDays;
        let leaves = [
            TemporalConstraintConcept::Day,
            TemporalConstraintConcept::BusinessDay,
            TemporalConstraintConcept::Week,
            TemporalConstraintConcept::Month,
            TemporalConstraintConcept::Year,
        ];
        let mut prev: u32 = 0;
        for c in leaves {
            let Some(v) = q.get(&c) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            if v <= prev {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            prev = v;
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GranularityOrdering",
        "ApproximateDays gives a strict ordering Day < BusinessDay < Week < Month < Year",
        "ISO 8601:2019 Date and time representations; FRCP Rule 6(a)(6)"
    );
}

pr4xis::register_axiom!(GranularityOrdering, "ISO 8601:2019; FRCP Rule 6(a)(6)");

/// Axiom: `Immediate` has no positive duration — its `ApproximateDays`
/// is `None`, distinguishing it from any positive-count granularity.
///
/// TimeML PT0S grounds the zero-duration semantics.
pub struct ImmediateIsZeroDuration;

impl Axiom for ImmediateIsZeroDuration {
    fn verify(&self) -> Verdict {
        if ApproximateDays
            .get(&TemporalConstraintConcept::Immediate)
            .is_none()
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ImmediateIsZeroDuration",
        "Immediate has no positive ApproximateDays — distinct from positive-count units",
        "Pustejovsky (2003) TimeML §3.2 (PT0S zero-duration)"
    );
}

pr4xis::register_axiom!(ImmediateIsZeroDuration, "Pustejovsky (2003) TimeML §3.2");

/// Axiom: a single business day fits within a single week. Concretely,
/// `ApproximateDays(BusinessDay) < ApproximateDays(Week)`. This is the
/// FRCP Rule 6(a)(6) invariant: a five-business-day window spans
/// roughly seven calendar days, but a single business day is shorter
/// than a calendar week.
pub struct BusinessDayBelowWeek;

impl Axiom for BusinessDayBelowWeek {
    fn verify(&self) -> Verdict {
        let q = ApproximateDays;
        match (
            q.get(&TemporalConstraintConcept::BusinessDay),
            q.get(&TemporalConstraintConcept::Week),
        ) {
            (Some(b), Some(w)) if b < w => Ok(Box::new(SimpleProof::new(self.meta()))),
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "BusinessDayBelowWeek",
        "a single business day is shorter than a calendar week",
        "FRCP Rule 6(a)(6); 5 U.S.C. § 5546"
    );
}

pr4xis::register_axiom!(BusinessDayBelowWeek, "FRCP Rule 6(a)(6); 5 U.S.C. § 5546");
