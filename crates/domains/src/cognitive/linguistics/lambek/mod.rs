pub mod integration_tests;
pub mod montague;
pub mod pregroup;
pub mod reduce;
pub mod tokenize;
pub mod turing_benchmark;
pub mod types;

pub use reduce::{ReductionResult, TypedToken, reduce_sequence};
pub use types::LambekType;

#[cfg(test)]
mod tests;
