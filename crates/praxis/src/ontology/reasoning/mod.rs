pub mod analogy;
pub mod cached;
pub mod causation;
pub mod context;
pub mod equivalence;
pub(crate) mod graph;
pub mod mereology;
pub mod opposition;
pub mod taxonomy;

pub use analogy::Analogy;
pub use cached::{CachedEquivalence, CachedMereology, CachedOpposition, CachedTaxonomy};
pub use causation::{CausalCategory, CausalDef, Causes};
pub use context::ContextDef;
pub use equivalence::{EquivalenceCategory, EquivalenceDef, Equivalent};
pub use mereology::{HasA, MereologyCategory, MereologyDef};
pub use opposition::OppositionDef;
pub use taxonomy::{IsA, TaxonomyCategory, TaxonomyDef};

#[cfg(test)]
mod tests;
