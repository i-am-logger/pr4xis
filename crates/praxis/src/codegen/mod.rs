mod builder;
mod generate;
pub mod wordnet;

pub use builder::{EntityDef, GenerateConfig, OntologyBuilder};
pub use generate::generate_rust;

// Re-export CodegenData from the always-available module.
pub use crate::codegen_data::CodegenData;
