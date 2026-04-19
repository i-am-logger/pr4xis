//! Situation calculus + STRIPS — substrate ontology grounding `engine/`.
//! Names Situation, Action, Fluent, Precondition, Plan, Trace, plus BDI
//! (Agent / Belief / Desire / Intention) per McCarthy (1963), Fikes &
//! Nilsson (1971), Reiter (2001), Bratman (1987).

pub mod ontology;

pub use ontology::{
    SituationCalculusCategory, SituationCalculusConcept, SituationCalculusOntology,
    SituationCalculusTradition,
};
