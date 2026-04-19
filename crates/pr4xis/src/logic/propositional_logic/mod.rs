//! Propositional logic — substrate ontology grounding `logic/composition.rs`,
//! `logic/propositional.rs`, and `logic/truth_table.rs`. Names the concepts
//! (Proposition, Formula, Connective, Tautology, Contradiction, TruthTable,
//! and the classical theorems) per Boole (1854), Frege (1879), Russell &
//! Whitehead (1910–13), Post (1921), Sheffer (1913), Aristotle, Tarski
//! (1936), Kleene (1952).

pub mod ontology;

pub use ontology::{
    PropositionalLogicCategory, PropositionalLogicConcept, PropositionalLogicOntology,
    PropositionalTradition,
};
