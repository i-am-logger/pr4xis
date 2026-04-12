use crate::formal::math::temporal::interval::Interval;

/// Allen's 13 interval relations.
///
/// Allen (1983) defined 13 jointly exhaustive and pairwise disjoint
/// binary relations between temporal intervals. Every pair of intervals
/// satisfies exactly one of these relations.
///
/// Source: Allen, J.F. "Maintaining Knowledge about Temporal Intervals"
///         Communications of the ACM, 26(11):832-843, 1983.
///
/// Formal treatment: Grüninger, M. & Li, Z. "The Time Ontology of
///         Allen's Interval Algebra" TIME 2017, LIPIcs Vol. 90.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllenRelation {
    /// X takes place before Y (gap between them).
    Before,
    /// Y takes place before X.
    After,
    /// X meets Y (X.end = Y.begin, no gap).
    Meets,
    /// Y meets X.
    MetBy,
    /// X overlaps with Y (partial overlap, X starts first).
    Overlaps,
    /// Y overlaps with X.
    OverlappedBy,
    /// X starts at the same time as Y but ends before Y.
    Starts,
    /// Y starts at the same time as X but ends before X.
    StartedBy,
    /// X is during Y (X is contained within Y).
    During,
    /// Y is during X (Y is contained within X).
    Contains,
    /// X finishes at the same time as Y but starts after Y.
    Finishes,
    /// Y finishes at the same time as X but starts after X.
    FinishedBy,
    /// X and Y are equal (same begin and end).
    Equal,
}

impl AllenRelation {
    /// The inverse relation: if R(X,Y) then R^{-1}(Y,X).
    pub fn inverse(&self) -> Self {
        match self {
            Self::Before => Self::After,
            Self::After => Self::Before,
            Self::Meets => Self::MetBy,
            Self::MetBy => Self::Meets,
            Self::Overlaps => Self::OverlappedBy,
            Self::OverlappedBy => Self::Overlaps,
            Self::Starts => Self::StartedBy,
            Self::StartedBy => Self::Starts,
            Self::During => Self::Contains,
            Self::Contains => Self::During,
            Self::Finishes => Self::FinishedBy,
            Self::FinishedBy => Self::Finishes,
            Self::Equal => Self::Equal,
        }
    }
}

/// Determine the Allen relation between two intervals.
/// Uses tolerance for floating-point comparison.
pub fn relate(x: &Interval, y: &Interval, tol: f64) -> AllenRelation {
    let xb = x.begin.seconds;
    let xe = x.end.seconds;
    let yb = y.begin.seconds;
    let ye = y.end.seconds;

    let eq = |a: f64, b: f64| (a - b).abs() < tol;

    if eq(xb, yb) && eq(xe, ye) {
        AllenRelation::Equal
    } else if eq(xb, yb) && xe < ye {
        AllenRelation::Starts
    } else if eq(xb, yb) && xe > ye {
        AllenRelation::StartedBy
    } else if eq(xe, ye) && xb > yb {
        AllenRelation::Finishes
    } else if eq(xe, ye) && xb < yb {
        AllenRelation::FinishedBy
    } else if xe < yb - tol {
        AllenRelation::Before
    } else if xb > ye + tol {
        AllenRelation::After
    } else if eq(xe, yb) {
        AllenRelation::Meets
    } else if eq(xb, ye) {
        AllenRelation::MetBy
    } else if xb < yb && xe > yb && xe < ye {
        AllenRelation::Overlaps
    } else if xb > yb && xb < ye && xe > ye {
        AllenRelation::OverlappedBy
    } else if xb > yb && xe < ye {
        AllenRelation::During
    } else {
        // xb < yb && xe > ye
        AllenRelation::Contains
    }
}
