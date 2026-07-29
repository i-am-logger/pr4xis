pub mod category_projection;
pub mod integration_tests;
pub mod montague;
pub mod montague_functor;
pub mod notation_parser;
pub mod ontology;
pub mod operators;
pub mod pregroup;
pub mod quote_glyphs;
pub mod reduce;
pub mod supertag_costs;
pub mod tokenize;
pub mod turing_benchmark;
pub mod types;

pub use montague_functor::LambekToMontague;
pub use ontology::{LambekCategory, LambekConcept, LambekOntology, LambekRelation};
pub use reduce::{ExpressionUse, ReductionResult, TypedToken};
pub use tokenize::tokenize_ontological;
pub use types::LambekType;

#[cfg(test)]
mod tests;
