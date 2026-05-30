pub mod catalog;
pub mod instance;
pub mod lemon_adjunction;
pub mod lemon_functor;
pub mod ontology;
pub mod vocabulary;

// Auto-registered via linkme distributed_slice — no central registry file.
pub use catalog::{LoadedRef, SourceAvailability, SourceStatus, source_catalog};
pub use instance::SelfModelInstance;
pub use ontology::*;
pub use pr4xis::ontology::describe_knowledge_base;
pub use vocabulary::KnowledgeBase;

#[cfg(test)]
mod tests;
