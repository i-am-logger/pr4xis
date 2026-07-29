//! Peano arithmetic (1889) + the Numeral -> `calculator::Value` bridge
//! (Hurford 1975) -- turing-benchmark A3.

pub mod numeral;
pub mod ontology;

pub use numeral::value_of_numeral_word;
pub use ontology::{
    CalculatorAdditionSatisfiesPeanoRecursion, CalculatorMultiplicationSatisfiesPeanoRecursion,
    PeanoArithmeticCategory, PeanoArithmeticConcept, PeanoArithmeticOntology, RecursionRole,
    RecursionStep, ZeroAnnihilatesMultiplication, ZeroIsIdentityForAddition,
};
