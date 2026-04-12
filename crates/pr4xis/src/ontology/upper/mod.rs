/// DOLCE-aligned upper ontology for praxis.
///
/// DOLCE (Descriptive Ontology for Linguistic and Cognitive Engineering) provides
/// the philosophical foundation for classifying everything that exists into
/// fundamental categories. This module defines:
///
/// - [`Being`] — the top-level types of being (DOLCE-inspired)
/// - [`DolceCategory`] — Being as a category with ontological relations
/// - [`Classified`] — trait for domains to declare their ontological type
/// - [`PraxisToDolce`] — functor proving our type system maps to DOLCE
///
/// # Pattern: Ontology Evolution via Functor
///
/// This module demonstrates the praxis pattern for ontology evolution:
/// instead of rewriting the existing type system, we create a new ontology
/// (DOLCE-aligned) alongside it and prove the mapping via functor.
/// The functor preserves identity and composition, guaranteeing that
/// the transformation is structure-preserving.
///
/// This same pattern applies to any ontology migration: create the new
/// category, build a functor from old to new, test the laws, save.
pub mod being;
pub mod category;
pub mod classify;
pub mod functor;

pub use being::Being;
pub use category::{DolceCategory, OntologicalRelation};
pub use classify::Classified;
pub use functor::{PraxisMetaCategory, PraxisToDolce, PraxisType};

#[cfg(test)]
mod tests;
