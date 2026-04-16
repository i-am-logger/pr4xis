pub mod descriptor;
pub mod instance;
pub mod lemon_adjunction;
pub mod lemon_functor;
pub mod ontology;
pub mod vocabulary;

pub use descriptor::describe_knowledge_base;
pub use instance::SelfModelInstance;
pub use ontology::*;
pub use pr4xis::ontology::OntologyDescriptor;
pub use vocabulary::{KnowledgeBase, Vocabulary};

#[cfg(test)]
mod tests;
