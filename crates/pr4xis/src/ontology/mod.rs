#[macro_use]
pub mod macros;
mod domain;
pub mod interpretation;
pub mod meta;
mod property;
pub mod provenance;
pub mod reasoning;
pub mod registry;
pub mod staging;
pub mod validate;

pub use crate::logic::Axiom;
pub use domain::Ontology;
pub use meta::{
    Citation, ConceptName, Definition, Grade, Label, LanguageCode, Lexical, ModulePath, Morphism,
    MorphismKind, OntologyName, Provenance, SynkolationLevel, Vocabulary, Year,
};
pub use property::{Quality, QualityKind};
#[cfg(not(target_arch = "wasm32"))]
pub use registry::{
    ADJUNCTION_CONSTRUCTORS, ADJUNCTIONS, AXIOM_CONSTRUCTORS, AXIOMS, FUNCTOR_CONSTRUCTORS,
    FUNCTORS, NATURAL_TRANSFORMATION_CONSTRUCTORS, NATURAL_TRANSFORMATIONS, VOCABULARIES,
};
pub use registry::{
    BoxedAxiom, adjunction_by_name, axiom_by_name, axiom_constructors, boxed_axiom,
    connection_constructors, describe_adjunctions, describe_all_arrows, describe_axioms,
    describe_functors, describe_knowledge_base, describe_natural_transformations, functor_by_name,
};
pub use staging::Staging;

#[cfg(test)]
mod tests;
