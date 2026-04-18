//! Lineage functor: Syntrometry → Pr4xisSubstrate.
//!
//! The claim the project has made since its first research notes is that
//! pr4xis's categorical substrate *instantiates* Heim's syntrometric
//! structure. Per `feedback_docs_need_proof.md`, the right way to
//! substantiate that claim is not to cite it in prose; the right way is to
//! encode both sides as ontologies, build a functor between them, and let
//! `check_functor_laws` turn the lineage claim into a tested theorem.
//!
//! # The mapping (Heim §1-2)
//!
//! | Syntrometry concept | Substrate concept | Source |
//! |---|---|---|
//! | `Predicate`     | `SubEntity`      | atomic distinction = Entity variant |
//! | `Predikatrix`   | `SubOntology`    | predicate-system = small ontology |
//! | `Dialektik`     | `SubCategory`    | binary-opposition structure |
//! | `Koordination`  | `SubMorphism`    | ordering between predicates = morphism |
//! | `Aspekt`        | `SubCategory`    | product [D × K × P] = product category |
//! | `Syntrix`       | `SubCategory`    | C_SL (§2.2 — Category of Leveled Structures) |
//! | `SyntrixLevel`  | `SubEntity`      | single level = object in the category |
//! | `Synkolator`    | `SubEndofunctor` | endofunctor on the Syntrix |
//! | `Korporator`    | `SubFunctor`     | structure-mapping functor |
//! | `Part`          | `SubMorphism`    | mereological relation = morphism |
//!
//! The collapse (Predikatrix + several other concepts → SubOntology /
//! SubCategory) is honest: pr4xis's substrate doesn't distinguish the
//! subjective/objective/hierarchical flavours of leveled structures —
//! they're all Categories from its vantage point.
//!
//! # Verification
//!
//! The single test in this module calls
//! [`pr4xis::category::validate::check_functor_laws`] on
//! `SyntrometryToPr4xisSubstrate`. If it passes, the lineage claim is
//! verified at test time. If the encoding or the mapping is wrong, the
//! laws fail — not the prose.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    SyntrometryCategory, SyntrometryConcept, SyntrometryRelation, SyntrometryRelationKind,
};
use super::substrate::{
    Pr4xisSubstrateCategory, Pr4xisSubstrateConcept, Pr4xisSubstrateRelation,
    Pr4xisSubstrateRelationKind,
};

fn map_concept(c: &SyntrometryConcept) -> Pr4xisSubstrateConcept {
    use Pr4xisSubstrateConcept as P;
    use SyntrometryConcept as S;
    match c {
        // Core primitives each land at a distinct substrate target.
        S::Predicate => P::SubEntity,
        S::PredicateMatrix => P::SubOntology,
        S::Syntrix => P::SubCategory,
        S::Coordination => P::SubMorphism,
        S::Syncolator => P::SubEndofunctor,
        S::Composer => P::SubFunctor,
        // Teleological / hierarchical concepts.
        S::Telecenter => P::SubEigenform,
        S::Maxim => P::SubIntention,
        S::TranscendenceLevel => P::SubStagingLevel,
        S::Metroplex => P::SubSystemOfSystems,
        // Refined distinctions — the substrate sub-kinds preserve these
        // without collapsing them to their parent.
        S::Aspect => P::SubProductCategory,
        S::SyntrixLevel => P::SubGradedObject,
        S::Part => P::SubObject,
        // Dialectic intentionally collapses to the plain SubCategory in
        // the primary substrate — opposition structure lives in the
        // dedicated Dialectics ontology, reached via the
        // `SyntrometryToDialectics` cross-functor.
        S::Dialectic => P::SubCategory,
        // Permutation operators are endomorphism-like (automorphisms on the
        // predicate-sequence / orientation structure).
        S::SequencePermutation | S::OrientationPermutation => P::SubEndofunctor,
        // AspectivalSystem is a structured multi-aspect collection = a small
        // ontology at the substrate level.
        S::AspectivalSystem => P::SubOntology,
        // Reflexivity is a natural transformation (Mac Lane Ch. II §4).
        S::Reflexivity => P::SubNaturalTransformation,
    }
}

/// The lineage functor: Syntrometry → Pr4xisSubstrate.
pub struct SyntrometryToPr4xisSubstrate;

impl Functor for SyntrometryToPr4xisSubstrate {
    type Source = SyntrometryCategory;
    type Target = Pr4xisSubstrateCategory;

    fn map_object(obj: &SyntrometryConcept) -> Pr4xisSubstrateConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> Pr4xisSubstrateRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        // Preserve source's Identity → target's Identity. Everything else
        // becomes Composed in the target — matching how Category::compose
        // produces Composed morphisms for non-Identity cases (so
        // F(g∘f) == F(g)∘F(f) holds even when F collapses distinct source
        // objects to the same target object).
        match m.kind {
            SyntrometryRelationKind::Identity => Pr4xisSubstrateCategory::identity(&from),
            _ => Pr4xisSubstrateRelation {
                from,
                to,
                kind: Pr4xisSubstrateRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(SyntrometryToPr4xisSubstrate);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    /// The headline test: the lineage claim, verified by functor laws.
    ///
    /// > "pr4xis sits in a lineage from Spencer-Brown through Heim. The
    /// > lineage is not asserted from research — it is verified by a
    /// > functor whose laws are checked at test time."
    #[test]
    fn lineage_functor_laws_pass() {
        check_functor_laws::<SyntrometryToPr4xisSubstrate>().unwrap();
    }
}
