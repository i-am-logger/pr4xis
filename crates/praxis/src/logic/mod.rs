pub mod axiom;
pub mod composition;
pub mod inference;
pub mod propositional;
pub mod truth_table;

pub use axiom::Axiom;
pub use inference::{
    Abduction, AbductionResult, Deduction, DeductionResult, Induction, InductionResult,
};
