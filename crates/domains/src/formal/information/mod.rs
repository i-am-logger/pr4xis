pub mod communication;
pub mod concurrency;
pub mod diagnostics;
pub mod dialogue;
pub mod events;
pub mod knowledge;
pub mod measurement;
pub mod ontology;
pub mod schema;
pub mod storage;

// provenance moved to core: `pr4xis::ontology::provenance`.
// It grounds the W3C PROV-O concepts that `RelationshipMeta` carries.

pub use ontology::*;

#[cfg(test)]
mod tests;
