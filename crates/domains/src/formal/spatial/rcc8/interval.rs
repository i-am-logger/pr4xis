//! The realized RCC-8 classifier over closed 1-D intervals (Randell, Cui &
//! Cohn 1992 §2-3): the simplest faithful region representation for which
//! every one of the 8 base relations is exactly computable from four
//! boundary comparisons, used as the concrete substrate the RCC-8
//! taxonomy's grounding axiom checks against.

use core::cmp::Ordering;

/// A closed interval `[start, end]` on the real line — a 1-D "region"
/// (Randell, Cui & Cohn 1992's own worked 1-D and 2-D examples; a closed
/// interval has a well-defined boundary and interior, exactly what RCC-8's
/// connection predicate needs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub start: f64,
    pub end: f64,
}

impl Interval {
    pub fn new(start: f64, end: f64) -> Self {
        assert!(start <= end, "an interval's start must not exceed its end");
        Self { start, end }
    }
}

/// The 8 RCC-8 base relations (Randell, Cui & Cohn 1992 §3, Table 1) — a
/// jointly exhaustive, pairwise disjoint (JEPD) partition of every possible
/// spatial relationship between two regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// DC — DisConnected: the regions share no point.
    DisConnected,
    /// EC — Externally Connected: the regions share only boundary points.
    ExternallyConnected,
    /// PO — Partially Overlapping: the regions share interior points but
    /// neither contains the other.
    PartiallyOverlapping,
    /// TPP — Tangential Proper Part: `a` is a proper part of `b` and their
    /// boundaries touch.
    TangentialProperPart,
    /// NTPP — Non-Tangential Proper Part: `a` is a proper part of `b` and
    /// their boundaries do not touch (`a` is strictly interior to `b`).
    NonTangentialProperPart,
    /// TPPi — the inverse of TPP (`b` is a tangential proper part of `a`).
    TangentialProperPartInverse,
    /// NTPPi — the inverse of NTPP.
    NonTangentialProperPartInverse,
    /// EQ — Equal: the regions are identical.
    Equal,
}

/// Classify the RCC-8 relation `a` bears to `b` — the exact classification
/// for closed 1-D intervals (Randell, Cui & Cohn 1992 §3): four boundary
/// comparisons (`a.start` vs `b.start`, `a.end` vs `b.end`, plus the two
/// disjointness checks) determine the unique relation among the 8, per the
/// JEPD property.
pub fn classify(a: Interval, b: Interval) -> Relation {
    use Relation as R;

    if a.start == b.start && a.end == b.end {
        return R::Equal;
    }
    // Disjoint (no shared point at all) -> DC.
    if a.end < b.start || b.end < a.start {
        return R::DisConnected;
    }
    // Touching at exactly one boundary point, otherwise disjoint interiors
    // -> EC.
    if a.end == b.start || b.end == a.start {
        return R::ExternallyConnected;
    }
    let a_inside_b = b.start <= a.start && a.end <= b.end;
    let b_inside_a = a.start <= b.start && b.end <= a.end;
    if a_inside_b {
        let tangent = a.start == b.start || a.end == b.end;
        return if tangent {
            R::TangentialProperPart
        } else {
            R::NonTangentialProperPart
        };
    }
    if b_inside_a {
        let tangent = a.start == b.start || a.end == b.end;
        return if tangent {
            R::TangentialProperPartInverse
        } else {
            R::NonTangentialProperPartInverse
        };
    }
    // Neither disjoint, touching, nor one contained in the other: the
    // interiors genuinely overlap without full containment.
    R::PartiallyOverlapping
}

/// Is `a` connected to `b` at all (`C(a,b)` in RCC's own notation) — every
/// relation except DC.
pub fn connected(a: Interval, b: Interval) -> bool {
    classify(a, b) != Relation::DisConnected
}

/// A total order over interval START points — used only to make property
/// tests over generated interval pairs deterministic; not part of the RCC-8
/// calculus itself.
pub fn start_order(a: Interval, b: Interval) -> Ordering {
    a.start.partial_cmp(&b.start).unwrap_or(Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn disjoint_intervals_are_disconnected() {
        let a = Interval::new(0.0, 1.0);
        let b = Interval::new(2.0, 3.0);
        assert_eq!(classify(a, b), Relation::DisConnected);
        assert_eq!(classify(b, a), Relation::DisConnected);
        assert!(!connected(a, b));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn touching_intervals_are_externally_connected() {
        let a = Interval::new(0.0, 1.0);
        let b = Interval::new(1.0, 2.0);
        assert_eq!(classify(a, b), Relation::ExternallyConnected);
        assert_eq!(classify(b, a), Relation::ExternallyConnected);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn overlapping_intervals_are_partially_overlapping() {
        let a = Interval::new(0.0, 2.0);
        let b = Interval::new(1.0, 3.0);
        assert_eq!(classify(a, b), Relation::PartiallyOverlapping);
        assert_eq!(classify(b, a), Relation::PartiallyOverlapping);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_strictly_interior_subinterval_is_a_non_tangential_proper_part() {
        let inner = Interval::new(1.0, 2.0);
        let outer = Interval::new(0.0, 3.0);
        assert_eq!(classify(inner, outer), Relation::NonTangentialProperPart);
        assert_eq!(
            classify(outer, inner),
            Relation::NonTangentialProperPartInverse
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_boundary_touching_subinterval_is_a_tangential_proper_part() {
        let inner = Interval::new(0.0, 1.0);
        let outer = Interval::new(0.0, 3.0);
        assert_eq!(classify(inner, outer), Relation::TangentialProperPart);
        assert_eq!(
            classify(outer, inner),
            Relation::TangentialProperPartInverse
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn identical_intervals_are_equal() {
        let a = Interval::new(0.0, 1.0);
        let b = Interval::new(0.0, 1.0);
        assert_eq!(classify(a, b), Relation::Equal);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classification_is_symmetric_in_the_expected_inverse_pairs() {
        // DC, EC, PO, EQ are each their own inverse; TPP/NTPP invert to
        // TPPi/NTPPi and back.
        let a = Interval::new(0.5, 1.5);
        let b = Interval::new(0.0, 2.0);
        let ab = classify(a, b);
        let ba = classify(b, a);
        let expected_inverse = match ab {
            Relation::TangentialProperPart => Relation::TangentialProperPartInverse,
            Relation::NonTangentialProperPart => Relation::NonTangentialProperPartInverse,
            Relation::TangentialProperPartInverse => Relation::TangentialProperPart,
            Relation::NonTangentialProperPartInverse => Relation::NonTangentialProperPart,
            same => same,
        };
        assert_eq!(ba, expected_inverse);
    }
}
