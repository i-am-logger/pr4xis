pub mod argument;
pub mod authority;
pub mod decision;
pub mod element;
pub mod engine;
pub mod entity;
pub mod fact;
pub mod finding;
pub mod lifecycle;
pub mod ontology;
pub mod rule;
pub mod source;

// Re-export commonly used types
pub use argument::*;
pub use authority::*;
pub use decision::*;
pub use element::*;
pub use engine::*;
pub use entity::*;
pub use fact::*;
pub use finding::*;
pub use lifecycle::*;
pub use ontology::*;
pub use rule::*;
pub use source::*;

#[cfg(test)]
mod tests;
