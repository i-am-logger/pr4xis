pub mod descriptor;
pub mod instance;
pub mod lemon_adjunction;
pub mod lemon_functor;
pub mod ontology;

pub use descriptor::describe_knowledge_base;
pub use instance::SelfModelInstance;
pub use ontology::*;
pub use pr4xis::ontology::OntologyDescriptor;

#[cfg(test)]
mod tests;
