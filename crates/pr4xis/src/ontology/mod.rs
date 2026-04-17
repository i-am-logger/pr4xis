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
pub use compose::{Concept, EdgeKind, Lexical, Metroplex, OntologyBuilder, Staging};
pub use domain::Ontology;
pub use meta::{Citation, ModulePath, OntologyMeta, OntologyName, Vocabulary};
pub use property::Quality;
#[cfg(not(target_arch = "wasm32"))]
pub use registry::VOCABULARIES;
pub use registry::describe_knowledge_base;

#[cfg(test)]
mod tests;
