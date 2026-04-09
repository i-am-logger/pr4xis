pub mod ontology;
pub mod pos;
pub mod vocabulary;

pub use pos::*;
pub use vocabulary::{lookup, lookup_all};

#[cfg(test)]
mod tests;
