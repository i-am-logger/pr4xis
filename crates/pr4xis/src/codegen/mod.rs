mod builder;
mod generate;
pub mod statute;
pub mod usc_corpus;
pub mod uslm;
pub mod uslm_schema;
pub mod wordnet;
pub mod xhtml_schema;
pub mod xml_grammar;
pub mod xml_schemas;

pub use builder::{EntityDef, GenerateConfig, OntologyBuilder};
pub use generate::generate_rust;

// Re-export CodegenData from the always-available module.
pub use crate::codegen_data::CodegenData;
