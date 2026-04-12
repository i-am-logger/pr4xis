pub mod descriptor;
pub mod instance;
pub mod ontology;

pub use descriptor::{VocabularyDescriptor, describe_knowledge_base};
pub use instance::SelfModelInstance;
pub use ontology::*;

#[cfg(test)]
mod tests;
