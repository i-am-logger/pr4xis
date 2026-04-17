pub mod category;
#[cfg(feature = "codegen")]
pub mod codegen;
pub mod codegen_data;
pub mod engine;
pub mod logic;
pub mod ontology;

pub use pr4xis_derive::ontology;
