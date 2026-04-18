#[macro_use]
pub mod macros;
pub mod compose;
mod domain;
pub mod meta;
mod property;
pub mod reasoning;
pub mod registry;
pub mod upper;
pub mod validate;

pub use crate::logic::Axiom;
pub use compose::Ontology as RuntimeOntology;
pub use compose::{EdgeKind, Metroplex, OntologyBuilder, RuntimeConcept, Staging};
pub use domain::Ontology;
pub use meta::{
    Citation, ConceptName, Definition, Grade, Label, LanguageCode, Lexical, ModulePath, Morphism,
    MorphismKind, OntologyName, RelationshipMeta, SynkolationLevel, Vocabulary, Year,
};
pub use property::{Quality, QualityKind};
#[cfg(not(target_arch = "wasm32"))]
pub use registry::{ADJUNCTIONS, AXIOMS, FUNCTORS, NATURAL_TRANSFORMATIONS, VOCABULARIES};
pub use registry::{
    describe_adjunctions, describe_all_arrows, describe_axioms, describe_functors,
    describe_knowledge_base, describe_natural_transformations,
};

#[cfg(test)]
mod tests;
