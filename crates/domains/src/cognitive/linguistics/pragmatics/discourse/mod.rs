mod legacy;
pub mod ontology;

pub use legacy::{Discourse, Turn};

#[cfg(test)]
mod tests;
