pub mod montague;
pub mod reduce;
pub mod tokenize;
pub mod types;

pub use reduce::{ReductionResult, TypedToken, reduce_sequence};
pub use types::LambekType;

#[cfg(test)]
mod tests;
