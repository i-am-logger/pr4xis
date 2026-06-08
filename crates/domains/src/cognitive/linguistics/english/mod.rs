pub mod ontology;

pub use ontology::{Concept, ConceptId, English, LexicalReasoner, SenseId, english_loaded};

#[cfg(test)]
mod tests;
