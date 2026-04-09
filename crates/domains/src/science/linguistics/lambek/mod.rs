pub mod reduce;
pub mod tokenize;
pub mod types;

pub use reduce::{ReductionResult, TypedToken, reduce_sequence};
pub use types::LambekType;

// TODO: montague module — the syntax→semantics functor
// Must implement type-driven interpretation per Montague (1973):
// - Each Lambek type maps to a semantic type (NP→entity, S→proposition, N→predicate)
// - Forward application maps to function application: f(x)
// - Backward application maps to function application: f(x)
// - Semantic rules are DETERMINED by syntactic rules — no pattern matching on words
// - This is a homomorphism (functor) from syntax algebra to semantics algebra
// Research needed: Montague PTQ, DisCoCat (Coecke 2010), de Felice thesis (2022)

#[cfg(test)]
mod tests;
