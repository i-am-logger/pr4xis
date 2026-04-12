use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for DistinctionElement {
    fn variants() -> Vec<Self> {
        vec![
            Self::Void,
            Self::Mark,
            Self::Boundary,
            Self::MarkedSpace,
            Self::UnmarkedSpace,
            Self::ReEntry,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DistinctionRelation {
    pub from: DistinctionElement,
    pub to: DistinctionElement,
    pub kind: DistinctionRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistinctionRelationKind {
    Identity,
    /// Mark creates Boundary (the act of distinguishing).
    Creates,
    /// Boundary separates MarkedSpace from UnmarkedSpace.
    Separates,
    /// ReEntry applies distinction to itself (self-reference).
    AppliesTo,
    /// Mark indicates MarkedSpace.
    Indicates,
    Composed,
}

impl Relationship for DistinctionRelation {
    type Object = DistinctionElement;
    fn source(&self) -> DistinctionElement {
        self.from
    }
    fn target(&self) -> DistinctionElement {
        self.to
    }
}

pub struct DistinctionCategory;

impl Category for DistinctionCategory {
    type Object = DistinctionElement;
    type Morphism = DistinctionRelation;

    fn identity(obj: &DistinctionElement) -> DistinctionRelation {
        DistinctionRelation {
            from: *obj,
            to: *obj,
            kind: DistinctionRelationKind::Identity,
        }
    }

    fn compose(f: &DistinctionRelation, g: &DistinctionRelation) -> Option<DistinctionRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == DistinctionRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == DistinctionRelationKind::Identity {
            return Some(f.clone());
        }
        Some(DistinctionRelation {
            from: f.from,
            to: g.to,
            kind: DistinctionRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<DistinctionRelation> {
        use DistinctionElement::*;
        use DistinctionRelationKind::*;

        let mut m = Vec::new();

        for e in DistinctionElement::variants() {
            m.push(DistinctionRelation {
                from: e,
                to: e,
                kind: Identity,
            });
        }

        // The two axioms of Laws of Form:
        // 1. Mark creates Boundary (the law of calling)
        m.push(DistinctionRelation {
            from: Mark,
            to: Boundary,
            kind: Creates,
        });
        // 2. Boundary separates spaces (the law of crossing)
        m.push(DistinctionRelation {
            from: Boundary,
            to: MarkedSpace,
            kind: Separates,
        });
        m.push(DistinctionRelation {
            from: Boundary,
            to: UnmarkedSpace,
            kind: Separates,
        });
        // Mark indicates MarkedSpace
        m.push(DistinctionRelation {
            from: Mark,
            to: MarkedSpace,
            kind: Indicates,
        });
        // Void → Mark (distinction emerges from void)
        m.push(DistinctionRelation {
            from: Void,
            to: Mark,
            kind: Creates,
        });
        // ReEntry: distinction applied to itself
        m.push(DistinctionRelation {
            from: ReEntry,
            to: Mark,
            kind: AppliesTo,
        });
        m.push(DistinctionRelation {
            from: ReEntry,
            to: Boundary,
            kind: AppliesTo,
        });

        // Transitive
        m.push(DistinctionRelation {
            from: Void,
            to: Boundary,
            kind: Composed,
        });
        m.push(DistinctionRelation {
            from: Void,
            to: MarkedSpace,
            kind: Composed,
        });
        m.push(DistinctionRelation {
            from: Mark,
            to: UnmarkedSpace,
            kind: Composed,
        });
        m.push(DistinctionRelation {
            from: ReEntry,
            to: MarkedSpace,
            kind: Composed,
        });
        m.push(DistinctionRelation {
            from: ReEntry,
            to: UnmarkedSpace,
            kind: Composed,
        });

        // Self-composed
        for e in DistinctionElement::variants() {
            m.push(DistinctionRelation {
                from: e,
                to: e,
                kind: Composed,
            });
        }

        m
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
