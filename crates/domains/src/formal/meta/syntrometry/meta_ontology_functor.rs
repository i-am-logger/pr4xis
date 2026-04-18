//! Cross-functor: Syntrometry → OntologyDiagnostics::MetaOntology.
//!
//! Demonstrates that Heim's
//! syntrometric vocabulary and pr4xis's meta-ontology vocabulary describe
//! the same primitives using different words — an explicit structural
//! alignment between the two, proven by a strict Functor whose laws check
//! at test time.
//!
//! # The mapping
//!
//! | Syntrometry | MetaEntity | Why |
//! |---|---|---|
//! | `Predicate`     | `DomainOntology`     | The atomic unit of a domain vocabulary |
//! | `Predikatrix`   | `TaxonomyStructure`  | A structured set of predicates = taxonomy |
//! | `Dialektik`     | `CausalStructure`    | Binary opposition ≈ minimal causal structure |
//! | `Koordination`  | `NaturalTransformation` | Ordering between parallel operators |
//! | `Aspekt`        | `QualityStructure`   | Subjective view = quality slice |
//! | `Syntrix`       | `CategoryStructure`  | §2.2 — Syntrix IS a category |
//! | `SyntrixLevel`  | `CategoryStructure`  | A level is a sub-category (collapse — intentional) |
//! | `Synkolator`    | `Functor`            | Endofunctor on a Syntrix |
//! | `Korporator`    | `Functor`            | General functor (collapse) |
//! | `Part`          | `UnitMorphism`       | Mereology embeds into the morphism structure |
//! | `Telecenter`    | `CanonicalRepresentative` | The round-trip attractor |
//! | `Maxime`        | `PropertyTest`       | Extremal selection = invariant check |
//! | `Transzendenzstufe` | `IntermediateDomain` | A grade between two abstract levels |
//! | `Metroplex`     | `Structure`          | The top-level abstract container |
//!
//! The functor has six intentional collapses out of 18 concepts — pairs
//! that share a diagnostic role land at the same `MetaEntity` bucket:
//!
//! - `Synkolator` / `Korporator` / `SequencePermutation` /
//!   `OrientationPermutation` → `Functor`
//! - `Syntrix` / `SyntrixLevel` → `CategoryStructure`
//! - `Predicate` / `Aspektivsystem` → `DomainOntology`
//! - `Koordination` / `Reflexivity` → `NaturalTransformation`
//!
//! These are honest: pr4xis's meta-ontology deliberately doesn't
//! distinguish endofunctor-on-a-Syntrix from a general functor at that
//! level of abstraction. The finer distinctions live in Pr4xisSubstrate
//! (where e.g. `SubEndofunctor` ⊂ `SubFunctor`) and in the dedicated
//! Dialectics / Kripke ontologies.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    SyntrometryCategory, SyntrometryConcept, SyntrometryRelation, SyntrometryRelationKind,
};
use crate::formal::meta::ontology_diagnostics::ontology::{
    MetaCategory, MetaCategoryRelationKind, MetaEntity, MetaRelation,
};

fn map_concept(c: &SyntrometryConcept) -> MetaEntity {
    use MetaEntity as M;
    use SyntrometryConcept as S;
    match c {
        S::Predicate => M::DomainOntology,
        S::PredicateMatrix => M::TaxonomyStructure,
        S::Dialectic => M::CausalStructure,
        S::Coordination => M::NaturalTransformation,
        S::Aspect => M::QualityStructure,
        S::Syntrix | S::SyntrixLevel => M::CategoryStructure,
        S::Syncolator | S::Composer => M::Functor,
        S::Part => M::UnitMorphism,
        S::Telecenter => M::CanonicalRepresentative,
        S::Maxim => M::PropertyTest,
        S::TranscendenceLevel => M::IntermediateDomain,
        S::Metroplex => M::Structure,
        S::SequencePermutation | S::OrientationPermutation => M::Functor,
        S::AspectivalSystem => M::DomainOntology,
        S::Reflexivity => M::NaturalTransformation,
    }
}

/// Cross-domain functor: Syntrometry → MetaOntology.
pub struct SyntrometryToMetaOntology;

impl Functor for SyntrometryToMetaOntology {
    type Source = SyntrometryCategory;
    type Target = MetaCategory;

    fn map_object(obj: &SyntrometryConcept) -> MetaEntity {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> MetaRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        // Preserve source's Identity → target's identity; everything else
        // becomes a non-identity arrow in the dense target.
        match m.kind {
            SyntrometryRelationKind::Identity => MetaCategory::identity(&from),
            _ => MetaRelation {
                from,
                to,
                kind: MetaCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(SyntrometryToMetaOntology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    /// Heim's vocabulary aligns with pr4xis's
    /// own meta-ontology vocabulary via a strict Functor.
    #[test]
    fn meta_ontology_functor_laws_pass() {
        check_functor_laws::<SyntrometryToMetaOntology>().unwrap();
    }
}
