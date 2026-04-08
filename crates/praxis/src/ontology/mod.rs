mod domain;
mod property;
pub mod reasoning;
mod rule;
pub mod validate;

pub use domain::Ontology;
pub use property::Quality;
pub use rule::Axiom;

#[cfg(test)]
mod tests;
