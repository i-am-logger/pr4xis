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
//! `ReEntry` goes to `Syncolator` (self-application = endofunctor) so the
//! AppliesTo edges land on the declared `(Syncolator, Syntrix,
//! EndomorphismOn)` edge in Syntrometry.
//!
//! | Distinction | Syntrometry | Why |
//! |---|---|---|
//! | `Void`          | `Syntrix` | Pre-distinction state, collapses to Syntrix |
//! | `Mark`          | `Syntrix` | Atomic distinction = object in the category |
//! | `Boundary`      | `Syntrix` | Boundary = internal structure of the category |
//! | `MarkedSpace`   | `Syntrix` | Space = the category itself |
//! | `UnmarkedSpace` | `Syntrix` | Complement space, collapses |
//! | `ReEntry`       | `Syncolator` | Self-applied distinction = endofunctor |
//!
//! The 5:1 collapse to Syntrix is honest: Spencer-Brown's distinction
//! calculus doesn't distinguish the levels of structure Heim's syntrometric
//! logic adds. What matters for the lineage is that the one concept with
//! self-reference — `ReEntry` — lands at `Syncolator` with the right edge
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
        D::ReEntry => S::Syncolator,
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
            // Every other kind maps to Composed in the target — matching how
            // the target's compose produces Composed morphisms for non-Identity
            // inputs (so F(g∘f) == F(g)∘F(f) holds even when F collapses
            // distinct source objects to the same target object).
            _ => SyntrometryRelation {
                from,
                to,
                kind: SyntrometryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(
    DistinctionToSyntrometry,
    "Historically Spencer-Brown's *Laws of Form* (1969) is older than Heim's"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn distinction_to_syntrometry_laws_pass() {
        check_functor_laws::<DistinctionToSyntrometry>().unwrap();
    }
}
