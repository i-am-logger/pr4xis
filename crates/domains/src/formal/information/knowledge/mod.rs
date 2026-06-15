pub mod catalog;
pub mod instance;
pub mod lemon_adjunction;
pub mod lemon_functor;
pub mod ontology;
pub mod vocabulary;

// Auto-registered via linkme distributed_slice — no central registry file.
pub use catalog::{LoadedRef, SourceAvailability, SourceStatus, source_catalog};
pub use instance::{SYSTEM_NAME, SelfModelInstance, is_self_referent, self_referents};
pub use ontology::*;
pub use pr4xis::ontology::describe_knowledge_base;
pub use vocabulary::{KnowledgeBase, runtime_ontology_vocabulary};

#[cfg(test)]
mod tests;
