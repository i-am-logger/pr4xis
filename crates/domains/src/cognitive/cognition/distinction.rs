use pr4xis::category::Entity;
use pr4xis::define_ontology;

// Distinction — the most fundamental act of cognition.
//
// "Draw a distinction." — G. Spencer-Brown, Laws of Form (1969)
//
// Before categories, before logic, before knowledge — there is distinction.
// The act of drawing a boundary that separates "this" from "not this."
// Everything in praxis — Entity, Opposition, Boundary, Bit — is a distinction.
//
// References:
// - Spencer-Brown, Laws of Form (1969)
// - Kauffman, Laws of Form — An Exploration (docs/papers/)
// - von Foerster, Observing Systems (1981) — distinction as basis of observation

/// The elements of distinction — what exists when a boundary is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum DistinctionElement {
    /// The unmarked state — the void before any distinction is made.
    Void,
    /// The mark — the act of indicating one side of a distinction.
    Mark,
    /// The boundary — the distinction itself, separating marked from unmarked.
    Boundary,
    /// The marked space — what is indicated (this side).
    MarkedSpace,
    /// The unmarked space — what is not indicated (the other side).
    UnmarkedSpace,
    /// Re-entry — the distinction applied to itself (self-reference).
    /// This creates time and self-awareness (Spencer-Brown Ch. 11).
    ReEntry,
}

define_ontology! {
    /// Distinction — the most fundamental cognitive act (Spencer-Brown 1969).
    pub DistinctionOntology for DistinctionCategory {
        concepts: DistinctionElement,
        relation: DistinctionRelation,
        kind: DistinctionRelationKind,
        kinds: [
            /// Mark creates Boundary (the act of distinguishing).
            Creates,
            /// Boundary separates MarkedSpace from UnmarkedSpace.
            Separates,
            /// ReEntry applies distinction to itself (self-reference).
            AppliesTo,
            /// Mark indicates MarkedSpace.
            Indicates,
        ],
        edges: [
            // The two axioms of Laws of Form:
            // 1. Mark creates Boundary (the law of calling)
            (Mark, Boundary, Creates),
            // 2. Boundary separates spaces (the law of crossing)
            (Boundary, MarkedSpace, Separates),
            (Boundary, UnmarkedSpace, Separates),
            // Mark indicates MarkedSpace
            (Mark, MarkedSpace, Indicates),
            // Void → Mark (distinction emerges from void)
            (Void, Mark, Creates),
            // ReEntry: distinction applied to itself
            (ReEntry, Mark, AppliesTo),
            (ReEntry, Boundary, AppliesTo),
        ],
        composed: [
            (Void, Boundary),
            (Void, MarkedSpace),
            (Mark, UnmarkedSpace),
            (ReEntry, MarkedSpace),
            (ReEntry, UnmarkedSpace),
        ],

        being: AbstractObject,
        source: "Spencer-Brown (1969)",
    }
}

/// Draw a distinction — the fundamental cognitive act.
/// Returns the two sides of the distinction.
pub fn draw_distinction<T: Clone + PartialEq + std::fmt::Debug>(marked: T, unmarked: T) -> (T, T) {
    assert_ne!(
        marked, unmarked,
        "a distinction requires two different things"
    );
    (marked, unmarked)
}
