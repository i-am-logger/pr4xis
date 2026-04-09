mod builder;
mod generate;
pub mod wordnet;

pub use builder::{EntityDef, GenerateConfig, OntologyBuilder};
pub use generate::generate_rust;
