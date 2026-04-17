//! Cross-functor: Syntrometry → OntologyDiagnostics::MetaOntology.
//!
//! Phase 4 of the Heim-pr4xis lineage work. Demonstrates that Heim's
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
//! The collapses (SyntrixLevel → CategoryStructure, Korporator → Functor)
//! are honest: pr4xis's meta-ontology deliberately doesn't distinguish
//! endofunctor-on-a-Syntrix from a general functor at that level of
//! abstraction. Those distinctions live in Pr4xisSubstrate (Phases 1-3).

use pr4xis::category::{Category, Functor};

use super::ontology::{SyntrometryCategory, SyntrometryConcept, SyntrometryRelation};
use crate::formal::meta::ontology_diagnostics::ontology::{MetaCategory, MetaEntity, MetaRelation};

fn map_concept(c: &SyntrometryConcept) -> MetaEntity {
    use MetaEntity as M;
    use SyntrometryConcept as S;
    match c {
        S::Predicate => M::DomainOntology,
        S::Predikatrix => M::TaxonomyStructure,
        S::Dialektik => M::CausalStructure,
        S::Koordination => M::NaturalTransformation,
        S::Aspekt => M::QualityStructure,
        S::Syntrix | S::SyntrixLevel => M::CategoryStructure,
        S::Synkolator | S::Korporator => M::Functor,
        S::Part => M::UnitMorphism,
        S::Telecenter => M::CanonicalRepresentative,
        S::Maxime => M::PropertyTest,
        S::Transzendenzstufe => M::IntermediateDomain,
        S::Metroplex => M::Structure,
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
        if from == to {
            MetaCategory::identity(&from)
        } else {
            MetaRelation { from, to }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    /// The Phase 4 headline test: Heim's vocabulary aligns with pr4xis's
    /// own meta-ontology vocabulary via a strict Functor.
    #[test]
    fn meta_ontology_functor_laws_pass() {
        check_functor_laws::<SyntrometryToMetaOntology>().unwrap();
    }
}
