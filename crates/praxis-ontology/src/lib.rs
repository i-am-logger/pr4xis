mod domain;
mod property;
mod rule;
pub mod validate;

/// Axiom — a domain-specific rule that an ontology must satisfy.
pub mod axiom {
    pub use crate::rule::Axiom;
}

/// Quality — an attribute or capability that inheres in an individual.
pub mod quality {
    pub use crate::property::Quality;
}

/// Ontology — what exists, how things relate, and what rules govern them.
pub mod ontology {
    pub use crate::domain::Ontology;
}

pub use axiom::Axiom;
pub use ontology::Ontology;
pub use quality::Quality;

// Re-export praxis-logic composition layer for consumers
pub use praxis_logic::{
    self as logic, AllOf, AnyOf, Compare, CompareOp, Evaluation, Implies, Measurable, Not,
    Proposition, Threshold,
};

// Re-export praxis-category for consumers that need the math layer
pub use praxis_category;

#[cfg(test)]
mod tests;
