#[macro_use]
pub mod macros;
mod domain;
pub mod meta;
mod property;
pub mod reasoning;
pub mod upper;
pub mod validate;

pub use crate::logic::Axiom;
pub use domain::Ontology;
pub use meta::OntologyMeta;
pub use property::Quality;

#[cfg(test)]
mod tests;
