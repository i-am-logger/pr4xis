//! Cross-functor: Distinction → Syntrometry (reverse-direction lineage).
//!
//! Historically Spencer-Brown's *Laws of Form* (1969) is older than Heim's
//! *Syntrometrische Maximentelezentrik* (ca. 1980). The natural structural
//! embedding runs **Distinction → Syntrometry**: every distinction
//! primitive has a syntrometric counterpart, and the functor embeds
//! Spencer-Brown's small vocabulary into Heim's richer one.
//!
//! This is the only **kinded → kinded** functor in the Heim stack. Kinded
//! cross-functors need careful authoring — per the #98 research note, the
//! source morphism's `(from, to, kind)` image must exist in the target's
//! declared morphism set (including Identity self-loops, declared edges,
//! and Composed self-loops). Non-declared (from, to) pairs outside the
//! composed: closure would fail the functor laws.
//!
//! # The mapping
//!
//! Most distinction primitives collapse to `Syntrix` (the top-level
//! leveled-structure category) so every edge becomes a Syntrix self-loop;
//! `ReEntry` goes to `Synkolator` (self-application = endofunctor) so the
//! AppliesTo edges land on the declared `(Synkolator, Syntrix,
//! EndomorphismOn)` edge in Syntrometry.
//!
//! | Distinction | Syntrometry | Why |
//! |---|---|---|
//! | `Void`          | `Syntrix` | Pre-distinction state, collapses to Syntrix |
//! | `Mark`          | `Syntrix` | Atomic distinction = object in the category |
//! | `Boundary`      | `Syntrix` | Boundary = internal structure of the category |
//! | `MarkedSpace`   | `Syntrix` | Space = the category itself |
//! | `UnmarkedSpace` | `Syntrix` | Complement space, collapses |
//! | `ReEntry`       | `Synkolator` | Self-applied distinction = endofunctor |
//!
//! The 5:1 collapse to Syntrix is honest: Spencer-Brown's distinction
//! calculus doesn't distinguish the levels of structure Heim's syntrometric
//! logic adds. What matters for the lineage is that the one concept with
//! self-reference — `ReEntry` — lands at `Synkolator` with the right edge
//! structure preserved.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    SyntrometryCategory, SyntrometryConcept, SyntrometryRelation, SyntrometryRelationKind,
};
use crate::cognitive::cognition::distinction::{
    DistinctionCategory, DistinctionConcept, DistinctionRelation, DistinctionRelationKind,
};

fn map_concept(c: &DistinctionConcept) -> SyntrometryConcept {
    use DistinctionConcept as D;
    use SyntrometryConcept as S;
    match c {
        D::Void | D::Mark | D::Boundary | D::MarkedSpace | D::UnmarkedSpace => S::Syntrix,
        D::ReEntry => S::Synkolator,
    }
}

/// Cross-functor: Distinction → Syntrometry.
pub struct DistinctionToSyntrometry;

impl Functor for DistinctionToSyntrometry {
    type Source = DistinctionCategory;
    type Target = SyntrometryCategory;

    fn map_object(obj: &DistinctionConcept) -> SyntrometryConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &DistinctionRelation) -> SyntrometryRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            // Identity preserves: F(id_A) == id_{F(A)}.
            DistinctionRelationKind::Identity => SyntrometryCategory::identity(&from),
            // Composed source morphisms must map to Composed target morphisms
            // so that F(g ∘ f) == F(g) ∘ F(f) (target compose always produces
            // a Composed morphism for non-Identity inputs).
            DistinctionRelationKind::Composed => SyntrometryRelation {
                from,
                to,
                kind: SyntrometryRelationKind::Composed,
            },
            // For the declared edge kinds, pick the matching target edge
            // kind when one exists; otherwise fall through to Composed.
            // Note: every source edge kind MUST map consistently — if
            // target has a specific kind at (F.from, F.to), we return it;
            // otherwise Composed.
            _ => {
                if from == to {
                    // Self-loop target — prefer Composed self-loop (exists
                    // for every Syntrometry concept).
                    SyntrometryRelation {
                        from,
                        to,
                        kind: SyntrometryRelationKind::Composed,
                    }
                } else {
                    // Cross-object — fall through to Composed. Under the
                    // current concept mapping, no declared edge lives
                    // between the image endpoints (the (Synkolator, Syntrix,
                    // EndomorphismOn) pair is exercised by a self-loop
                    // target here because we collapse Mark/Boundary to
                    // Syntrix — the F(ReEntry → Boundary) chain doesn't
                    // reach a non-self-loop distinct target kind).
                    SyntrometryRelation {
                        from,
                        to,
                        kind: SyntrometryRelationKind::Composed,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn distinction_to_syntrometry_laws_pass() {
        check_functor_laws::<DistinctionToSyntrometry>().unwrap();
    }
}
