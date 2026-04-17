//! Minimal pr4xis-core substrate ontology — the target category for the
//! syntrometric lineage functor.
//!
//! Encodes just enough of pr4xis's categorical primitives (Entity, Morphism,
//! Category, Functor, Endofunctor, Ontology) to be a valid functor target
//! for [`super::lineage_functor`]. Dense category — no kinded morphism
//! structure — so the kinded Syntrometry source can map into it without
//! the kind-alignment problems documented in the #98 research note.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Pr4xisSubstrate",
    source: "pr4xis/src/category/*.rs — Entity, Relationship, Category, Functor, Endofunctor, Ontology trait definitions",
    being: AbstractObject,

    concepts: [
        SubEntity,
        SubMorphism,
        SubCategory,
        SubFunctor,
        SubEndofunctor,
        SubOntology,
    ],

    labels: {
        SubEntity: ("en", "Entity", "The pr4xis::category::Entity trait — an enum of the objects of a category."),
        SubMorphism: ("en", "Morphism", "The pr4xis::category::Relationship trait — an arrow between two Entities."),
        SubCategory: ("en", "Category", "The pr4xis::category::Category trait — an Entity type plus a Morphism type satisfying identity + composition laws."),
        SubFunctor: ("en", "Functor", "The pr4xis::category::Functor trait — a structure-preserving map Source → Target."),
        SubEndofunctor: ("en", "Endofunctor", "The pr4xis::category::Endofunctor trait — a Functor specialised to Source = Target."),
        SubOntology: ("en", "Ontology", "The pr4xis::ontology::Ontology trait — a Category + reasoning systems (taxonomy, mereology, causation, opposition) + axioms."),
    },

    is_a: [
        // True subsumption only: every Endofunctor is a Functor specialised
        // to Source = Target (Mac Lane Ch. II §1).
        (SubEndofunctor, SubFunctor),
    ],
}

/// Quality: which pr4xis crate-section each substrate primitive lives in.
#[derive(Debug, Clone)]
pub struct SubstrateLocation;

impl Quality for SubstrateLocation {
    type Individual = Pr4xisSubstrateConcept;
    type Value = &'static str;

    fn get(&self, c: &Pr4xisSubstrateConcept) -> Option<&'static str> {
        use Pr4xisSubstrateConcept as P;
        Some(match c {
            P::SubEntity => "pr4xis::category::entity",
            P::SubMorphism => "pr4xis::category::relationship",
            P::SubCategory => "pr4xis::category::category",
            P::SubFunctor => "pr4xis::category::functor",
            P::SubEndofunctor => "pr4xis::category::endofunctor",
            P::SubOntology => "pr4xis::ontology",
        })
    }
}

impl Ontology for Pr4xisSubstrateOntology {
    type Cat = Pr4xisSubstrateCategory;
    type Qual = SubstrateLocation;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Pr4xisSubstrateOntology::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<Pr4xisSubstrateCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        Pr4xisSubstrateOntology::validate().unwrap();
    }
}
