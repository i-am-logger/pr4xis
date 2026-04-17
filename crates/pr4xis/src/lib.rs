pub mod category;
#[cfg(feature = "codegen")]
pub mod codegen;
pub mod codegen_data;
pub mod engine;
pub mod logic;
pub mod ontology;

pub use pr4xis_derive::ontology;

// Re-export linkme/paste so downstream macros can refer to them
// without requiring crates to add these dependencies directly.
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use paste;
