pub mod ontology;

pub use ontology::{Concept, ConceptId, English, LexicalReasoner, SenseId};

#[cfg(test)]
mod tests;
