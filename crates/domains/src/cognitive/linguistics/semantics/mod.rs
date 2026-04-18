pub mod interpret;
pub mod meaning;
pub mod ontology;

pub use meaning::*;
pub use ontology::{MontagueCategory, MontagueConcept, MontagueOntology, MontagueRelation};

#[cfg(test)]
mod tests;
