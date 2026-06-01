#[macro_use]
pub mod macros;
pub mod compose;
mod domain;
pub mod interpretation;
pub mod meta;
mod property;
pub mod provenance;
pub mod reasoning;
pub mod registry;
pub mod validate;

pub use crate::logic::Axiom;
pub use compose::Ontology as RuntimeOntology;
pub use compose::{EdgeKind, Metroplex, OntologyBuilder, RuntimeConcept, Staging};
pub use domain::Ontology;
pub use meta::{
    Citation, ConceptName, Definition, Grade, Label, LanguageCode, Lexical, ModulePath, Morphism,
    MorphismKind, OntologyName, Provenance, SynkolationLevel, Vocabulary, Year,
};
pub use property::{Quality, QualityKind};
#[cfg(not(target_arch = "wasm32"))]
pub use registry::{
    ADJUNCTIONS, AXIOM_CONSTRUCTORS, AXIOMS, FUNCTORS, NATURAL_TRANSFORMATIONS, VOCABULARIES,
};
pub use registry::{
    BoxedAxiom, axiom_by_name, axiom_constructors, boxed_axiom, describe_adjunctions,
    describe_all_arrows, describe_axioms, describe_functors, describe_knowledge_base,
    describe_natural_transformations,
};

#[cfg(test)]
mod tests;
