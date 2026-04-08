pub mod analogy;
pub mod causation;
pub mod mereology;
pub mod taxonomy;

pub use analogy::Analogy;
pub use causation::{CausalCategory, CausalDef, Causes};
pub use mereology::{HasA, MereologyCategory, MereologyDef};
pub use taxonomy::{IsA, TaxonomyCategory, TaxonomyDef};

#[cfg(test)]
mod tests;
