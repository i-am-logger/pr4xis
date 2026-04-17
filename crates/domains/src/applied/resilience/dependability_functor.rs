//! Trivial functor Resilience → Dependability.
//!
//! Every resilience pattern (CircuitBreaker, Retry, Supervisor, Microreboot, …)
//! factors through `Dependability::FaultTolerance` — they are all instances of
//! fault-tolerance means. The natural structure-preserving map sends every
//! resilience concept to `FaultTolerance` and every resilience morphism to the
//! identity morphism on `FaultTolerance`.
//!
//! This is the **terminal functor** into the one-object subcategory
//! `{FaultTolerance, id}` of `DependabilityCategory`. It satisfies the functor
//! laws trivially:
//!
//! - `F(id_A) = id_FT` for every `A` — both sides are `id_FT`.
//! - `F(g ∘ f) = F(g) ∘ F(f)` — both sides are `id_FT ∘ id_FT = id_FT`.
//!
//! This fact is the empirical verification for the claim in
//! `docs/research/kinded-functor-failures.md` that case 3 of the three
//! previously deferred cross-ontology functors does not need any framework
//! extension — the trivial functor already satisfies the laws, and that *is*
//! the ontologically correct statement: "every resilience pattern is a
//! fault-tolerance means."
//!
//! Non-trivial sub-structure (e.g., mapping `CircuitBreaker` to a
//! `StabilityMeans` sub-kind of `FaultTolerance`) would require enriching
//! Dependability's means hierarchy, which is a content decision separate from
//! the functor laws. See the research doc for the recommendation.

use pr4xis::category::{Category, Functor};

use crate::applied::dependability::ontology::{
    DependabilityCategory, DependabilityConcept, DependabilityRelation,
};
use crate::applied::resilience::ontology::{
    ResilienceCategory, ResilienceConcept, ResilienceRelation,
};

/// The terminal functor `Resilience → {FaultTolerance}` subcategory of Dependability.
pub struct ResilienceToFaultTolerance;

impl Functor for ResilienceToFaultTolerance {
    type Source = ResilienceCategory;
    type Target = DependabilityCategory;

    fn map_object(_: &ResilienceConcept) -> DependabilityConcept {
        DependabilityConcept::FaultTolerance
    }

    fn map_morphism(_: &ResilienceRelation) -> DependabilityRelation {
        // Delegate identity construction to the category so the functor stays
        // correct if the generated relation type ever gains extra fields.
        DependabilityCategory::identity(&DependabilityConcept::FaultTolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn trivial_functor_satisfies_laws() {
        check_functor_laws::<ResilienceToFaultTolerance>().unwrap();
    }
}
