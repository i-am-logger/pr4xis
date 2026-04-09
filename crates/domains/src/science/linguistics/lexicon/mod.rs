pub mod function_words;
pub mod olia;
pub mod ontology;
pub mod pos;
pub mod vocabulary;

pub use function_words::{lookup, lookup_all};
pub use pos::*;

#[cfg(test)]
mod tests;
